use crate::utils;
use crate::{Context, Error};
use poise::CreateReply;
use serenity::all::CreateEmbed;

use dotenv::var;

use serenity::all::CreateEmbedFooter;
use tmdb_api::client::reqwest::ReqwestExecutor;
use tmdb_api::client::Client;
use tmdb_api::movie::details::MovieDetails;
use tmdb_api::movie::search::MovieSearch;
use tmdb_api::prelude::Command;

use chrono::Datelike;

use scraper::{Html, Selector};
use urlencoding::encode;

struct Book {
    title: Option<String>,
    author: Option<String>,
    rating: Option<f32>,
    thumbnail_url: Option<String>,
    description: Option<String>,
    page_count: Option<u16>,
    published_date: Option<String>,
    book_url: Option<String>,
}

async fn get_goodreads_url(book_title: &str) -> Result<String, Error> {
    // Parse book title to be used in the URL, i.e. replace spaces with '+' and special characters with their ASCII code
    let encoded_title = encode(book_title);
    let url = format!(
        "https://www.goodreads.com/search?utf8=✓&q={}&search_type=books&search[field]=on",
        encoded_title
    );

    let res = reqwest::get(url).await?;
    let text = res.text().await?;

    let document = Html::parse_document(&text);

    let table_selector = Selector::parse("table.tableList").unwrap();
    let table = document.select(&table_selector).next().unwrap();

    let a_selector = Selector::parse("a").unwrap();
    let a = table.select(&a_selector).next().unwrap();

    let href = a.value().attr("href").unwrap();

    let goodreads_url = format!("https://www.goodreads.com{}", href);

    Ok(goodreads_url)
}

async fn get_book(goodreads_url: &str) -> Result<Book, Error> {
    let res = reqwest::get(goodreads_url).await?;
    let text = res.text().await?;

    let document = Html::parse_document(&text);

    // Extract book title
    let title_selector = Selector::parse(".Text.Text__title1").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>());

    // Extract author
    let author_selector = Selector::parse(".ContributorLink__name").unwrap();
    let author = document
        .select(&author_selector)
        .next()
        .map(|el| el.text().collect::<String>());

    // Extract rating
    let rating_selector = Selector::parse(".RatingStatistics__rating").unwrap();
    let rating = document
        .select(&rating_selector)
        .next()
        .and_then(|el| el.text().collect::<String>().parse::<f32>().ok());

    // Extract thumbnail URL
    let thumbnail_selector = Selector::parse(".ResponsiveImage").unwrap();
    let thumbnail_url = document
        .select(&thumbnail_selector)
        .next()
        .and_then(|el| el.value().attr("src").map(|src| src.to_owned()));

    // Extract description
    let description_selector = Selector::parse(".Formatted").unwrap();
    let description = document
        .select(&description_selector)
        .next()
        .map(|el| el.text().collect::<String>());

    // Extract page count and published date
    let details_selector = Selector::parse(".FeaturedDetails").unwrap();
    let details_div = document.select(&details_selector).next();
    let (page_count, published_date) = if let Some(details) = details_div {
        let p_elements: Vec<_> = details.select(&Selector::parse("p").unwrap()).collect();
        let page_count = p_elements
            .first()
            .and_then(|el| el.text().collect::<String>().parse::<u16>().ok());
        let published_date = p_elements.get(1).map(|el| el.text().collect::<String>());
        (page_count, published_date)
    } else {
        (None, None)
    };

    Ok(Book {
        title,
        author,
        rating,
        thumbnail_url,
        description,
        page_count,
        published_date,
        book_url: Some(goodreads_url.to_owned()),
    })
}

#[poise::command(slash_command)]
pub async fn book(
    ctx: Context<'_>,
    #[description = "The title of the book"] book_title: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Fetch Goodreads URL
    let goodreads_url = get_goodreads_url(&book_title)
        .await
        .map_err(|_| "Error searching for book")?;

    // Fetch book details
    let book = get_book(&goodreads_url)
        .await
        .map_err(|_| "Error fetching book details")?;

    // Build and send the embed
    let embed = build_book_embed(&book).await?;
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

async fn build_book_embed(book: &Book) -> Result<CreateEmbed, Error> {
    let mut embed = CreateEmbed::default();

    // Title and URL
    if let Some(title) = &book.title {
        embed = embed.title(title);
    }
    if let Some(book_url) = &book.book_url {
        embed = embed.url(book_url);
    }

    // Thumbnail and primary color
    if let Some(thumbnail_url) = &book.thumbnail_url {
        embed = embed.thumbnail(thumbnail_url);

        if let Ok(primary_color) = utils::get_image_primary_color(thumbnail_url).await {
            embed = embed.color(primary_color);
        }
    }

    // Author
    if let Some(author) = &book.author {
        embed = embed.field("Author", author, true);
    }

    // Published date
    if let Some(published_date) = &book.published_date {
        embed = embed.field("Published", published_date, true);
    }

    // Description
    let description = book
        .description
        .as_deref()
        .map_or("No description available".to_owned(), truncate_description);
    embed = embed.field("Description", description, false);

    // Page count
    if let Some(page_count) = book.page_count {
        embed = embed.field("Page Count", page_count.to_string(), true);
    }

    // Rating
    if let Some(rating) = book.rating {
        embed = embed.field("Rating", format!("{:.1}/5", rating), true);
    }

    Ok(embed)
}

fn truncate_description(description: &str) -> String {
    if description.chars().count() > 500 {
        let truncated: String = description.chars().take(500).collect();
        format!("{}...", truncated)
    } else {
        description.to_owned()
    }
}

#[poise::command(slash_command)]
pub async fn kino(
    ctx: Context<'_>,
    #[description = "The title of the movie"] movie_title: String,
    #[description = "The year the movie was released"] year: Option<u16>,
) -> Result<(), Error> {
    ctx.defer().await?;
    dotenv::dotenv().ok();

    let Ok(tmdb_api_key) = var("TMDB_API_KEY") else {
        ctx.send(CreateReply::default().content("TMDB_API_KEY not set"))
            .await?;
        return Ok(());
    };

    let client = Client::<ReqwestExecutor>::new(tmdb_api_key);

    let Ok(result) = MovieSearch::new(movie_title)
        .with_year(year)
        .execute(&client)
        .await
    else {
        ctx.send(
            CreateReply::default()
                .content("Error searching for movie")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let Some(item) = result.results.first() else {
        ctx.send(
            CreateReply::default()
                .content("No results found")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let mut embed = CreateEmbed::default();

    if let Some(release_date) = &item.inner.release_date {
        let title_with_year = format!("{} ({})", item.inner.title, release_date.year());
        embed = embed.title(&title_with_year);
    } else {
        embed = embed.title(&item.inner.title);
    }

    embed = embed.field("Description", &item.inner.overview, false);

    if let Ok(details) = MovieDetails::new(item.inner.id).execute(&client).await {
        let budget = utils::format_currency(details.budget);
        embed = embed.field("Budget", format!("${}", budget), true);

        let revenue = utils::format_currency(details.revenue);
        embed = embed.field("Revenue", format!("${}", revenue), true);

        if let Some(runtime) = details.runtime {
            embed = embed.field("Runtime", format!("{} minutes", runtime), true);
        }

        if let Some(imdb_id) = details.imdb_id {
            let imdb_link = format!("https://www.imdb.com/title/{}", imdb_id);
            embed = embed.url(imdb_link);
        }
    }

    if let Some(poster_path) = &item.inner.poster_path {
        let poster_url = format!("https://image.tmdb.org/t/p/original{}", poster_path);
        embed = embed.image(&poster_url);

        if let Ok(primary_color) = utils::get_image_primary_color(&poster_url).await {
            embed = embed.color(primary_color);
        }
    }

    embed = embed.footer(CreateEmbedFooter::new("Data sourced from TMDb"));

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn image(
    ctx: Context<'_>,
    #[description = "The search term for the image"] search_term: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    dotenv::dotenv().ok();

    let Ok(api_key) = var("GOOGLE_API_KEY") else {
        ctx.send(CreateReply::default().content("GOOGLE_API_KEY not set"))
            .await?;
        return Ok(());
    };

    let Ok(cse_id) = var("GOOGLE_CSE_ID") else {
        ctx.send(CreateReply::default().content("GOOGLE_CSE_ID not set"))
            .await?;
        return Ok(());
    };

    let url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&searchType=image&cx={}&q={}",
        api_key, cse_id, search_term
    );

    let res = reqwest::get(&url)
        .await
        .map_err(|_| "Error fetching image")?;
    let text = res
        .text()
        .await
        .map_err(|_| "Error reading response body")?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| "Error parsing JSON response")?;

    if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
        if let Some(first_item) = items.first() {
            if let Some(link) = first_item.get("link").and_then(|l| l.as_str()) {
                ctx.send(CreateReply::default().content(link)).await?;
                return Ok(());
            }
        }
    }

    ctx.send(CreateReply::default().content("No image found"))
        .await?;
    Ok(())
}

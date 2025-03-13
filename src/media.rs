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

#[poise::command(slash_command)]
pub async fn kino(
    ctx: Context<'_>,
    #[description = "The title of the movie"] movie_title: String,
    #[description = "The year the movie was released"] year: Option<u16>,
) -> Result<(), Error> {
    ctx.defer().await?;
    dotenv::dotenv().ok();

    let Ok(tmdb_api_key) = var("TMDB_API_KEY") else {
        ctx.send(
            CreateReply::default()
                .content("TMDB_API_KEY not set")
        )
        .await?;
        return Ok(());
    };

    let client = Client::<ReqwestExecutor>::new(tmdb_api_key);

    let Ok(result) = MovieSearch::new(movie_title.into())
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

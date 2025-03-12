use crate::{Context, Error};
use chrono::Datelike;
use poise::{serenity_prelude::MessageId, CreateReply};
use serenity::all::CreateEmbed;

use dotenv::var;

use tmdb_api::client::reqwest::ReqwestExecutor;
use tmdb_api::client::Client;
use tmdb_api::movie::details::MovieDetails;
use tmdb_api::movie::search::MovieSearch;
use tmdb_api::prelude::Command;

use reqwest::Client as ReqwestClient;

extern crate color_thief;
extern crate image;

use color_thief::get_palette;
use color_thief::ColorFormat;
use image::load_from_memory;

//#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
#[poise::command(slash_command, guild_only)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What to say"] text_to_say: String,
    #[description = "The id of the message to reply to"] message_id: Option<MessageId>,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content("Alright boss")
            .ephemeral(true),
    )
    .await?;

    let Some(message_id) = message_id else {
        ctx.channel_id().say(ctx.http(), text_to_say).await?;
        return Ok(());
    };

    let message = match ctx.channel_id().message(ctx.http(), message_id).await {
        Ok(message) => message,
        Err(why) => {
            println!("Error getting message: {:?}", why);
            return Ok(());
        }
    };
    message.reply_ping(ctx.http(), text_to_say).await?;

    Ok(())
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
        ctx.send(
            CreateReply::default()
                .content("TMDB_API_KEY not set")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let client = Client::<ReqwestExecutor>::new(tmdb_api_key);
    let movie_search = MovieSearch::new(movie_title.into()).with_year(year);

    let Ok(result) = movie_search.execute(&client).await else {
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

    let details_result = MovieDetails::new(item.inner.id).execute(&client).await;

    if let Ok(details) = details_result {
        // Add commas to the budget
        let budget = details.budget.to_string().chars().rev().collect::<Vec<_>>();
        let budget = budget
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(",");
        let budget = budget.chars().rev().collect::<String>();
        embed = embed.field("Budget", format!("${}", budget), true);

        let revenue = details.revenue.to_string().chars().rev().collect::<Vec<_>>();
        let revenue = revenue
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(",");
        let revenue = revenue.chars().rev().collect::<String>();
        embed = embed.field("Revenue", format!("${}", revenue), true);
    }

    if let Some(poster_path) = &item.inner.poster_path {
        let poster_url = format!("https://image.tmdb.org/t/p/original{}", poster_path);
        embed = embed.image(&poster_url);

        // Fetch the image data
        let reqwest_client = ReqwestClient::new();
        let Ok(response) = reqwest_client.get(&poster_url).send().await else {
            println!("Failed to fetch image");
            ctx.send(
                CreateReply::default()
                    .content("Failed to fetch image")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        };

        let Ok(image_data) = response.bytes().await else {
            println!("Failed to read image data");
            ctx.send(
                CreateReply::default()
                    .content("Failed to read image data")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        };

        let Ok(image) = load_from_memory(&image_data) else {
            println!("Failed to load image");
            ctx.send(
                CreateReply::default()
                    .content("Failed to load image")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        };

        // Convert the image to raw pixels
        let pixels = image.to_rgba8();
        let pixels = pixels.as_raw();

        // Get the color palette
        let Ok(palette) = get_palette(pixels, ColorFormat::Rgba, 10, 2) else {
            println!("Failed to get color palette");
            ctx.send(
                CreateReply::default()
                    .content("Failed to get color palette")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        };

        if let Some(primary_color) = palette.first() {
            embed = embed.color((primary_color.r, primary_color.g, primary_color.b));
        }
    } else {
        embed = embed.description("No poster available");
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/*
/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Vote for something
///
/// Enter `~vote pumpkin` to vote for pumpkins
#[poise::command(prefix_command, slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
    ctx.say(response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// ~getvotes
/// ~getvotes pumpkin
/// ```

#[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        ctx.say(response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
            response += &format!("{}: {} votes", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        ctx.say(response).await?;
    };

    Ok(())
}
     */

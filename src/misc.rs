use crate::{Context, Error};

use chrono::Utc;
use csv::Reader;
use serenity::all::Member;
use std::fs::File;

use chrono_tz::Tz;

use poise::CreateReply;

async fn get_user_local_time(user_id: &str) -> Result<String, String> {
    // Open the CSV file
    let file =
        File::open("timezones.csv").map_err(|err| format!("Error opening CSV file: {:?}", err))?;
    let mut rdr = Reader::from_reader(file);

    // Find the timezone for the user
    let timezone_str = rdr
        .records()
        .filter_map(Result::ok)
        .find(|record| record.get(0) == Some(user_id))
        .and_then(|record| record.get(1).map(|s| s.to_owned()))
        .ok_or_else(|| "Timezone not found".to_owned())?;

    // Parse the timezone
    let tz: Tz = timezone_str
        .parse()
        .map_err(|_| format!("Invalid timezone: {}", timezone_str))?;

    // Get the current local time in the user's timezone
    let local_time = Utc::now().with_timezone(&tz);
    let formatted_time = local_time.format("%H:%M %d/%m").to_string();

    Ok(formatted_time)
}

#[poise::command(slash_command)]
pub async fn timezone(
    ctx: Context<'_>,
    #[description = "The user you wish to get the timezone of"] user: Member,
) -> Result<(), Error> {
    let user_id = user.user.id.to_string();

    match get_user_local_time(&user_id).await {
        Ok(local_time) => {
            ctx.send(CreateReply::default().content(format!(
                "The current time and date for {} is: {}",
                user.user.name, local_time
            )))
            .await?;
        }
        Err(err) => {
            ctx.send(CreateReply::default().content(format!(
                "Failed to get timezone for {}: {}",
                user.user.name, err
            )))
            .await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn timezones(ctx: Context<'_>) -> Result<(), Error> {
    let file = match File::open("timezones.csv") {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening CSV file: {:?}", err);
            ctx.send(CreateReply::default().content("Failed to open the timezones file."))
                .await?;
            return Ok(());
        }
    };

    let mut rdr = csv::Reader::from_reader(file);
    let mut response = String::new();

    // Iterate through all records in the CSV file
    for result in rdr.records() {
        let record = match result {
            Ok(record) => record,
            Err(err) => {
                println!("Error reading record: {:?}", err);
                continue;
            }
        };

        // Extract user ID
        let user_id = match record.get(0) {
            Some(id) => id.to_string(),
            None => continue,
        };

        // Fetch the user from Discord
        let user = match ctx
            .serenity_context()
            .http
            .get_user(serenity::all::UserId::new(user_id.parse::<u64>().unwrap()))
            .await
        {
            Ok(user) => user,
            Err(err) => {
                println!("Error fetching user for ID {}: {:?}", user_id, err);
                response.push_str(&format!("Failed to fetch user for ID {}.\n", user_id));
                continue;
            }
        };

        // Get the local time for the user
        match get_user_local_time(&user_id).await {
            Ok(local_time) => {
                response.push_str(&format!(
                    "{} : {}\n",
                    user.name, local_time
                ));
            }
            Err(err) => {
                response.push_str(&format!(
                    "Failed to get timezone for {}: {}\n",
                    user.name, err
                ));
            }
        }
    }

    // Send the response as a single message
    if response.is_empty() {
        response = "No timezones found or failed to process.".to_string();
    }

    ctx.send(CreateReply::default().content(response)).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn fix_twitter_link(
    ctx: Context<'_>,
    #[description = "The Twitter link to fix"] twitter_link: String,
) -> Result<(), Error> {
    let fixed_link = twitter_link
        .replace("https://x.com/", "https://vxtwitter.com/")
        .replace("https://twitter.com/", "https://vxtwitter.com/");

    // Remove tracking parameters
    let fixed_link = fixed_link
        .split('?')
        .next()
        .unwrap_or(&fixed_link)
        .to_string();
    ctx.send(CreateReply::default().content(fixed_link)).await?;

    Ok(())
}

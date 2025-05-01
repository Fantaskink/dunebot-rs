use crate::{Context, Error};

use serenity::all::Member;
use std::fs::File;

use chrono_tz::Tz;

use poise::CreateReply;

#[poise::command(slash_command)]
pub async fn timezone(
    ctx: Context<'_>,
    #[description = "The user you wish to get the timezone of"] user: Member,
) -> Result<(), Error> {
    let file = match File::open("timezones.csv") {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening CSV file: {:?}", err);
            return Ok(());
        }
    };

    let mut rdr = csv::Reader::from_reader(file);

    let user_id = user.user.id.to_string();

    // Get timezone for the user
    let timezone_str = rdr
        .records()
        .filter_map(Result::ok)
        .find(|record| record.get(0) == Some(&user_id))
        .and_then(|record| record.get(1).map(|s| s.to_string()))
        .unwrap_or("Timezone not found".to_string());

    let tz: Tz = timezone_str.parse().unwrap_or_else(|_| {
        println!("Invalid timezone: {}", timezone_str);
        "UTC".parse().expect("Failed to parse default timezone")
    });

    // Get current time in the user's timezone
    let local_time = chrono::Utc::now().with_timezone(&tz);
    // let tz = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
    // Only get hour, minute, day and month
    let tz = local_time.format("%H:%M %d/%m").to_string();

    ctx.send(CreateReply::default().content(format!(
        "The current time and date for {} is: {}",
        user.user.name, tz
    )))
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn fix_twitter_link(
    ctx: Context<'_>,
    #[description = "The Twitter link to fix"] twitter_link: String,
) -> Result<(), Error> {
    let fixed_link = twitter_link.replace("https://x.com/", "https://vxtwitter.com/");

    // Remove tracking parameters
    let fixed_link = fixed_link
        .split('?')
        .next()
        .unwrap_or(&fixed_link)
        .to_string();
    ctx.send(CreateReply::default().content(fixed_link)).await?;

    Ok(())
}

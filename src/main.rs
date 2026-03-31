#![warn(clippy::str_to_string)]

mod admin;
mod media;
mod misc;
mod utils;

use poise::serenity_prelude as serenity;
use serenity::all::FullEvent;

use dotenv::var;
use std::fs::File;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    //votes: Mutex<HashMap<String, u32>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

fn load_banned_words() -> Vec<String> {
    match File::open("banned_words.csv") {
        Ok(file) => {
            let mut rdr = csv::Reader::from_reader(file);
            let mut banned_words = Vec::new();
            
            for result in rdr.records() {
                if let Ok(record) = result {
                    if let Some(word) = record.get(0) {
                        banned_words.push(word.to_lowercase());
                    }
                }
            }
            
            banned_words
        }
        Err(err) => {
            println!("Error opening banned_words.csv: {:?}", err);
            Vec::new()
        }
    }
}

fn contains_banned_word(content: &str, banned_words: &[String]) -> bool {
    let content_lower = content.to_lowercase();
    banned_words.iter().any(|word| content_lower.contains(word))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            admin::say(),
            admin::start_birthday_reminders(),
            media::kino(),
            media::book(),
            media::image(),
            misc::timezone(),
            misc::timezones(),
            misc::fix_twitter_link(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, _framework, _data| {
            Box::pin(async move {
                match event {
                    FullEvent::Message { new_message } => {
                        let banned_words = load_banned_words();
                        
                        if contains_banned_word(&new_message.content, &banned_words) {
                            println!("Deleting message from {}: {}", new_message.author.name, new_message.content);
                            if let Err(why) = new_message.delete(&ctx.http).await {
                                println!("Error deleting message: {:?}", why);
                            }
                        }
                    }
                    _ => {
                        println!(
                            "Got an event in event handler: {:?}",
                            event.snake_case_name()
                        );
                    }
                }
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                   // votes: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    let token = var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

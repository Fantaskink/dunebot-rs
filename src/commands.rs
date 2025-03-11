use crate::{Context, Error};
use poise::{serenity_prelude::MessageId, CreateReply};

#[poise::command(slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What to say"] text_to_say: String,
    #[description = "The id of the message to reply to"] message_id: Option<MessageId>,
) -> Result<(), Error> {
    ctx.send(CreateReply::default().content("Alright boss").ephemeral(true))
        .await?;

    let Some(message_id) = message_id else {
        match ctx.guild_channel().await {
            Some(channel) => {
                channel.say(ctx.http(), text_to_say).await?;
            },
            None => {
                ctx.say("This command can only be used in a guild").await?;
                return Ok(());
            }
        };
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

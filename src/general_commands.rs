use crate::command_base::*;

use serenity::model::id::ChannelId;
use serenity::utils::{content_safe, ContentSafeOptions};
use std::fmt::Write;

#[group]
#[commands(say, commands, roll)]
pub struct General;

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (name, amount) in counter {
        writeln!(contents, "- {name}: {amount}")?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role mentions are replaced
// with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command("say")]
#[sub_commands(vallah)]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(x) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                // By default roles, users, and channel mentions are cleaned.
                ContentSafeOptions::default()
                    // We do not want to clean channal mentions as they do not ping users.
                    .clean_channel(false)
                    // If it's a guild channel, we want mentioned users to be displayed as their
                    // display name.
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let content = content_safe(&ctx.cache, x, &settings, &msg.mentions);

            msg.reply(&ctx.http, &content).await?;

            return Ok(());
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
            return Ok(());
        }
    };
}

// sub-command for say
#[command]
async fn vallah(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "Nee").await?;

    Ok(())
}

#[command]
async fn zitat(ctx: &Context, msg: &Message) -> CommandResult {
    let zitate_channel_id = ChannelId::new(1290616138308386816);

    let message = &msg.content;

    let f = message.find(' ');

    match f {
        Some(f) => {
            zitate_channel_id
                .say(&ctx.http, message.clone().split_off(f))
                .await?;
            msg.channel_id
                .say(&ctx.http, "Zitat posted in <#1290616138308386816>!")
                .await?
        }
        None => {
            msg.channel_id
                .say(&ctx.http, "You need to add a zitat!")
                .await?
        }
    };

    Ok(())
}

#[command]
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let roll = match args.single::<u32>() {
        Ok(roll) => roll,
        Err(_) => {
            msg.reply(ctx, "You need to provide a number!").await?;
            return Err("No number supplied".into());
        }
    };

    let result = rand::thread_rng().gen_range(1..=roll);

    msg.reply(ctx, format!("You rolled a {}!", result)).await?;
    Ok(())
}

use serenity::all::UserId;

use crate::command_base::*;

#[group]
#[prefixes("mod")]
#[commands(delete_msg, ban)]
pub struct Mod;

#[command("del")]
#[bucket = "mod"]
#[required_permissions(MANAGE_MESSAGES)]
async fn delete_msg(ctx: &Context, msg: &Message) -> CommandResult {
    msg.referenced_message.as_ref().unwrap().delete(&ctx.http).await?;
    msg.delete(&ctx.http).await?;
    Ok(())
}

#[command("ban")]
#[description = "Bans a user from the server."]
#[bucket = "mod"]
#[required_permissions(BAN_MEMBERS)]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = match args.single::<UserId>() {
        Ok(user) => user,
        Err(_) => {
            msg.reply(ctx, "You need to provide a user ID!").await?;
            return Ok(());
        }
    };

    let reason = match args.single::<String>() {
        Ok(reason) => reason,
        Err(_) => String::from("No reason provided."),
    };

    let guild_id = msg.guild_id.unwrap();
    let member = guild_id.member(&ctx.http, user).await?;

    if member.ban_with_reason(&ctx.http, 0, &reason).await.is_err() {
        msg.reply(ctx, "I cannot ban this user.").await?;
        return Ok(());
    }

    msg.channel_id
        .say(&ctx.http, format!("Banned {} for {}", user, reason))
        .await?;

    Ok(())
}
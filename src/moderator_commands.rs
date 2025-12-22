use serenity::all::UserId;

use crate::command_base::*;

#[group]
#[prefixes("mod")]
#[commands(delete_msg, ban, kick, warn, checkwarns)]
pub struct Mod;

#[command("del")]
#[bucket = "mod"]
#[required_permissions(MANAGE_MESSAGES)]
async fn delete_msg(ctx: &Context, msg: &Message) -> CommandResult {
    msg.referenced_message
        .as_ref()
        .unwrap()
        .delete(&ctx.http)
        .await?;
    msg.delete(&ctx.http).await?;
    Ok(())
}

#[command("ban")]
#[description = "Bans a user from the server."]
#[bucket = "mod"]
#[required_permissions(BAN_MEMBERS)]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usr = args.single::<String>().unwrap();
    if !usr.starts_with("<@") {
        msg.reply(ctx, "You need to provide a user mention!").await?;
        return Ok(());
    }
    let user = usr.split("<@").collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()
        .unwrap();

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
        .say(&ctx.http, format!("Banned <@{}> for {}", user, reason))
        .await?;

    Ok(())
}

#[command("kick")]
#[description = "Kicks a user from the server."]
#[bucket = "mod"]
#[required_permissions(KICK_MEMBERS)]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usr = args.single::<String>().unwrap();
    if !usr.starts_with("<@") {
        msg.reply(ctx, "You need to provide a user mention!").await?;
        return Ok(());
    }
    let user = usr.split("<@").collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()
        .unwrap();

    let reason = match args.single::<String>() {
        Ok(reason) => reason,
        Err(_) => String::from("No reason provided."),
    };

    let guild_id = msg.guild_id.unwrap();
    let member = guild_id.member(&ctx.http, user).await?;

    if member.kick_with_reason(&ctx.http, &reason).await.is_err() {
        msg.reply(ctx, "I cannot kick this user.").await?;
        return Ok(());
    }

    msg.channel_id
        .say(&ctx.http, format!("Kicked <@{}> for {}", user, reason))
        .await?;

    Ok(())
}

#[command("unban")]
#[description = "Unbans a user from the server."]
#[bucket = "mod"]
#[required_permissions(BAN_MEMBERS)]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usr = args.single::<String>().unwrap();
    if !usr.starts_with("<@") {
        msg.reply(ctx, "You need to provide a user mention!").await?;
        return Ok(());
    }
    let user = usr.split("<@").collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()
        .unwrap();

    let guild_id = msg.guild_id.unwrap();
    let member = guild_id.member(&ctx.http, user).await?;

    if member.unban(&ctx.http).await.is_err() {
        msg.reply(ctx, "I cannot unban this user.").await?;
        return Ok(());
    }

    msg.channel_id
        .say(&ctx.http, format!("Unbanned <@{}>", user))
        .await?;

    Ok(())
}

#[command("warn")]
#[description = "Warns a user."]
#[bucket = "mod"]
#[required_permissions(MANAGE_MESSAGES)]
async fn warn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usr = args.single::<String>().unwrap();
    if !usr.starts_with("<@") {
        msg.reply(ctx, "You need to provide a user mention!").await?;
        return Ok(());
    }
    let user = usr.split("<@").collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()
        .unwrap();

    let reason = args.rest();

    let username = match msg.guild(&ctx.cache).unwrap().members.get(&user) {
        Some(member) => member.user.name.clone(),
        None => String::from("Unknown_user"),
    };

    let time = msg.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
    let timed_reason = format!("[{}] {}", time, reason);

    let path = format!("warnings/{}txt", username);
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    use tokio::io::AsyncWriteExt;
    file.write_all(format!("{}\n", timed_reason).as_bytes()).await?;
    file.flush().await?;

    msg.channel_id
        .say(&ctx.http, format!("Warned <@{}> for {}", user, reason))
        .await?;

    Ok(())
}

#[command("checkwarns")]
#[description = "Checks the warns of a given user."]
#[bucket = "mod"]
#[required_permissions(MANAGE_MESSAGES)]
async fn checkwarns(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usr = args.single::<String>().unwrap();
    if !usr.starts_with("<@") {
        msg.reply(ctx, "You need to provide a user mention!").await?;
        return Ok(());
    }
    let user = usr.split("<@").collect::<Vec<&str>>()[1]
        .split(">")
        .collect::<Vec<&str>>()[0]
        .parse::<UserId>()
        .unwrap();
    let username = match msg.guild(&ctx.cache).unwrap().members.get(&user) {
        Some(member) => member.user.name.clone(),
        None => String::from("Unknown_user"),
    };
    let path = format!("warnings/{}txt", username);
    let content = tokio::fs::read_to_string(&path).await?;
    let warnings = content.lines().map(|line| line.to_string()).collect::<Vec<String>>();
    if warnings.is_empty() {
        msg.channel_id
            .say(&ctx.http, format!("<@{}> has no warnings.", user))
            .await?;
    } else {
        let warning_list = warnings.join("\n");
        msg.channel_id
            .say(
                &ctx.http,
                format!("Warnings for <@{}>:\n{}", user, warning_list),
            )
            .await?;
    }
    Ok(())
}
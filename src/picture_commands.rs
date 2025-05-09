use crate::command_base::*;

#[group]
#[prefixes("pic")]
#[commands(bird, pov, demi, bimbo, ösi)]
pub struct Pic;

async fn base_picture_command(
    ctx: &Context,
    msg: &Message,
    file_name: &str,
    file_count: u8,
) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=file_count);
    let path = format!("images/{}{}.jpg", file_name, rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("{}{}.jpg", file_name, rng)).await?;
    let _ = match msg
        .channel_id
        .send_message(
            &ctx.http,
            serenity::all::CreateMessage::new().add_file(attachment),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(why) => Err(serenity::all::standard::CommandError::from(why)),
    };
    Ok(())
}

#[command]
#[bucket = "pic"]
async fn bird(ctx: &Context, msg: &Message) -> CommandResult {
    base_picture_command(ctx, msg, "bird", 2).await
}

#[command]
#[bucket = "pic"]
async fn pov(ctx: &Context, msg: &Message) -> CommandResult {
    base_picture_command(ctx, msg, "pov", 3).await
}

#[command]
#[bucket = "pic"]
async fn demi(ctx: &Context, msg: &Message) -> CommandResult {
    base_picture_command(ctx, msg, "demi", 4).await
}

#[command]
#[bucket = "pic"]
async fn bimbo(ctx: &Context, msg: &Message) -> CommandResult {
    let demi_rng = rand::thread_rng().gen_range(0..=100);
    if demi_rng < 90 {
        base_picture_command(ctx, msg, "bimbo", 1).await
    } else {
        demi(ctx, msg, _args).await
    }
}

#[command]
#[bucket = "pic"]
async fn ösi(ctx: &Context, msg: &Message) -> CommandResult {
    base_picture_command(ctx, msg, "ösi", 4).await
}

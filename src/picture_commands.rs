use crate::command_base::*;

#[group]
#[prefixes("pic")]
#[commands(bird, pov, demi, bimbo)]
pub struct Pic;

#[command]
#[bucket = "pic"]
async fn bird(ctx: &Context, msg: &Message) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=2);
    let path = format!("images/bird{}.jpg", rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("bird{}.jpg", rng)).await?;
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
async fn pov(ctx: &Context, msg: &Message) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=3);
    let path = format!("images/pov{}.jpg", rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("pov{}.jpg", rng)).await?;
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
async fn demi(ctx: &Context, msg: &Message) -> CommandResult {
    let rng = rand::thread_rng().gen_range(1..=4);
    let path = format!("images/demi{}.jpg", rng);
    let f = &tokio::fs::File::open(path).await?;
    let attachment = serenity::all::CreateAttachment::file(f, format!("demi{}.jpg", rng)).await?;
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
async fn bimbo(ctx: &Context, msg: &Message) -> CommandResult {
    let demi_rng = rand::thread_rng().gen_range(0..=100);
    if demi_rng < 90 {
        let rng = rand::thread_rng().gen_range(1..=1);
        let path = format!("images/bimbo{}.jpg", rng);
        let f = &tokio::fs::File::open(path).await?;
        let attachment =
            serenity::all::CreateAttachment::file(f, format!("bimbo{}.jpg", rng)).await?;
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
    } else {
        demi(ctx, msg, _args).await
    }
}

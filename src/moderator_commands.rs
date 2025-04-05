use crate::command_base::*;

#[group]
#[prefixes("mod")]
#[commands(delete_msg)]
pub struct Mod;

#[command("del")]
#[bucket = "mod"]
async fn delete_msg(ctx: &Context, msg: &Message) -> CommandResult {
    msg.referenced_message.as_ref().unwrap().delete(&ctx.http).await?;
    msg.delete(&ctx.http).await?;
    Ok(())
}
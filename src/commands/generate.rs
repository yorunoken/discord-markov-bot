use serenity::all::CreateMessage;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::utils::generate_markov_message;

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    _command_name: &String,
    _command_alias: Option<&str>,
) -> Result<(), Error> {
    let guild_id = match msg.guild_id {
        Some(s) => s,
        _ => return Ok(()),
    };

    let builder = match generate_markov_message(
        guild_id,
        msg.channel_id,
        Some(args.join(" ").as_str()),
    )
    .await
    {
        Some(s) => s,
        None => {
            CreateMessage::new().content("Please wait until this channel has over 500 messages.")
        }
    };

    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}

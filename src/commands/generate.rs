use serenity::all::{
    CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption,
    EditInteractionResponse, ResolvedValue,
};
use serenity::prelude::*;
use serenity::Error;

use crate::utils::generate_markov_message;

pub async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    command.defer(&ctx.http).await?;

    let guild_id = match command.guild_id {
        Some(s) => s,
        _ => return Ok(()),
    };

    let options = &command.data.options();

    let word = options
        .iter()
        .find(|opt| opt.name == "word")
        .and_then(|opt| {
            if let ResolvedValue::String(s) = &opt.value {
                Some(*s)
            } else {
                None
            }
        });

    let builder = match generate_markov_message(guild_id, command.channel_id, word).await {
        Some(markov_message) => EditInteractionResponse::new().content(markov_message),
        None => EditInteractionResponse::new()
            .content("Please wait until this channel has over 500 messages."),
    };

    command.edit_response(&ctx.http, builder).await?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("generate")
        .description("Generates a markov message.")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "word",
            "What the sentence will start with",
        ))
}

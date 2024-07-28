use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::commands::ping;

type CommandFn = for<'a> fn(
    &'a Context,     // Command context, `ctx`
    &'a Message,     // Message variable, `msg`
    Vec<&'a str>,    // Arguments of the command, `args`
    &'a String,      // The command's name, `command_name`
    Option<&'a str>, // The command's name, `command_alias`
) -> BoxFuture<'a, Result<(), Error>>;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub exec: CommandFn,
}

pub fn get_prefix_commands() -> Vec<Command> {
    vec![Command {
        name: String::from("ping"),
        aliases: vec![String::from("p")],
        exec: |ctx, msg, args, command_name, command_alias| {
            Box::pin(ping::execute(ctx, msg, args, command_name, command_alias))
        },
    }]
}

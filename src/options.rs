use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::commands::ping;

type CommandFn = for<'a> fn(
    &'a Context,  // Command context, `ctx`
    &'a Message,  // Message variable, `msg`
    Vec<&'a str>, // Arguments of the command, `args`
    &'a String,   // The command's name, `command_name`
) -> BoxFuture<'a, Result<(), Error>>;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub exec: CommandFn,
}

pub fn get_prefix_commands() -> Vec<Command> {
    vec![Command {
        name: String::from("ping"),
        exec: |ctx, msg, args, command_name| Box::pin(ping::execute(ctx, msg, args, command_name)),
    }]
}

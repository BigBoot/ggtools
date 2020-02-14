use crate::commands::SendEmbed;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[help_available(false)]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say_embed(&ctx.http, "Pong!");

    Ok(())
}

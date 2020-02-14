pub mod admin;
pub mod meta;
pub mod server;

use crate::database::open_guild_db;
use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::check, Args, CheckResult, CommandOptions},
    http::raw::Http,
    model::prelude::*,
    prelude::*,
};

#[check]
#[name("is_trusted")]
pub fn is_trusted(ctx: &mut Context, msg: &Message, _args: &mut Args, _opts: &CommandOptions) -> CheckResult {
    if let Some(guild) = msg.guild(&ctx) {
        let guild = guild.read();

        let permissions = guild.member_permissions(&msg.author);

        if permissions.administrator() {
            return CheckResult::Success;
        }

        let user_db = open_guild_db(guild.id.into(), "users");
        if user_db.contains_key(msg.author.id.as_u64().to_be_bytes()).unwrap_or(false) {
            return CheckResult::Success;
        }

        if let Ok(member) = guild.member(&ctx, &msg.author) {
            let roles_db = open_guild_db(guild.id.into(), "roles");

            if member.roles.iter().any(|&role| roles_db.contains_key(role.as_u64().to_be_bytes()).unwrap_or(false)) {
                return CheckResult::Success;
            }
        }

        let admin_db = open_guild_db(guild.id.into(), "admins");
        if admin_db.contains_key(msg.author.id.as_u64().to_be_bytes()).unwrap_or(false) {
            return CheckResult::Success;
        }
    }
    else {
        return CheckResult::new_user("Error retrieving guild.");
    }

    return CheckResult::new_user("You're not authorized to use this command.");
}

#[check]
#[name("is_admin")]
pub fn is_admin(ctx: &mut Context, msg: &Message, _args: &mut Args, _opts: &CommandOptions) -> CheckResult {
    if let Some(guild) = msg.guild(&ctx) {
        let guild = guild.read();

        let permissions = guild.member_permissions(&msg.author);

        if permissions.administrator() {
            return CheckResult::Success;
        }

        let user_db = open_guild_db(guild.id.into(), "admins");
        if user_db.contains_key(msg.author.id.as_u64().to_be_bytes()).unwrap_or(false) {
            return CheckResult::Success;
        }
    }
    else {
        return CheckResult::new_user("Error retrieving guild.");
    }

    return CheckResult::new_user("You're not authorized to use this command.");
}

pub trait SendEmbed {
    fn say_embed(&self, http: impl AsRef<Http>, description: &str);
    fn send_embed<F>(&self, http: impl AsRef<Http>, f: F)
    where
        F: FnOnce(&mut CreateEmbed);
}

impl SendEmbed for ChannelId {
    fn say_embed(&self, http: impl AsRef<Http>, description: &str) {
        let _ = self.send_message(http, |m| {
            m.embed(|e| {
                e.description(description);
                e
            });
            m
        });
    }

    fn send_embed<F>(&self, http: impl AsRef<Http>, f: F)
    where
        F: FnOnce(&mut CreateEmbed),
    {
        let _ = self.send_message(http, |m| {
            m.embed(|e| {
                f(e);
                e
            });
            m
        });
    }
}

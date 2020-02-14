use crate::{commands::SendEmbed, database::open_guild_db};
use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
};

fn add_user_to_db(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
    cmd_name: &str,
    list: &str,
    db: &str,
) -> CommandResult {
    let usage = format!(r#"Usage: {} <username>"#, cmd_name);

    let username = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;

    let guild = msg.guild(&ctx).ok_or(CommandError(format!("Error retrieving guild.")))?;
    let guild = guild.read();

    let member = guild.member_named(&username).ok_or(CommandError(format!("No matching user found.")))?;
    let user = member.user.read();

    let db = open_guild_db(guild.id.into(), db);

    db.insert(user.id.as_u64().to_be_bytes(), &[]).map_err(|_| {
        CommandError(format!("Error adding {}#{} to the list of {} users.", &user.name, user.discriminator, list))
    })?;

    msg.channel_id
        .say_embed(&ctx, &format!("Added {}#{} to the list of {} users.", &user.name, user.discriminator, list));

    Ok(())
}

fn remove_user_from_db(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
    cmd_name: &str,
    list: &str,
    db: &str,
) -> CommandResult {
    let usage = format!(r#"Usage: {} <username>"#, cmd_name);

    let username = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;

    let guild = msg.guild(&ctx).ok_or(CommandError(format!("Error retrieving guild.")))?;
    let guild = guild.read();

    let member = guild.member_named(&username).ok_or(CommandError(format!("No matching user found.")))?;
    let user = member.user.read();

    let db = open_guild_db(guild.id.into(), db);

    if !db.contains_key(user.id.as_u64().to_be_bytes()).unwrap_or(false) {
        return Err(CommandError(format!(
            "User {}#{} is not in the list of {} users.",
            &user.name, user.discriminator, list
        )));
    }

    db.remove(user.id.as_u64().to_be_bytes()).map_err(|_| {
        CommandError(format!("Error removing {}#{} from the list of {} users.", user.name, user.discriminator, list))
    })?;

    msg.channel_id
        .say_embed(&ctx, &format!("Removed {}#{} from the list of {} users.", &user.name, user.discriminator, list));

    Ok(())
}

fn add_role_to_db(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
    cmd_name: &str,
    list: &str,
    db: &str,
) -> CommandResult {
    let usage = format!(r#"Usage: {} <role>"#, cmd_name);

    let rolename = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;

    let guild = msg.guild(&ctx).ok_or(CommandError(format!("Error retrieving guild.")))?;
    let guild = guild.read();

    let role = guild.role_by_name(&rolename).ok_or(CommandError(format!("No matching role found.")))?;

    let db = open_guild_db(guild.id.into(), db);

    db.insert(role.id.as_u64().to_be_bytes(), &[])
        .map_err(|_| CommandError(format!("Error adding {} to the list of {} roles.", &role.name, list)))?;

    msg.channel_id.say_embed(&ctx, &format!("Added {} to the list of {} roles.", &role.name, list));

    Ok(())
}

fn remove_role_from_db(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
    cmd_name: &str,
    list: &str,
    db: &str,
) -> CommandResult {
    let usage = format!(r#"Usage: {} <role>"#, cmd_name);

    let rolename = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;

    let guild = msg.guild(&ctx).ok_or(CommandError(format!("Error retrieving guild.")))?;
    let guild = guild.read();

    let role = guild.role_by_name(&rolename).ok_or(CommandError(format!("No matching role found.")))?;

    let db = open_guild_db(guild.id.into(), db);

    if !db.contains_key(role.id.as_u64().to_be_bytes()).unwrap_or(false) {
        return Err(CommandError(format!("Role {} is not in the list of {} role.", &role.name, list)));
    }

    db.remove(role.id.as_u64().to_be_bytes())
        .map_err(|_| CommandError(format!("Error removing {} from the list of {} roles.", role.name, list)))?;

    msg.channel_id.say_embed(&ctx, &format!("Removed {} from the list of {} roles.", &role.name, list));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn set_notification_channel(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: set_notification_channel <channel>"#);

    let channelname = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;

    let guild = msg.guild(&ctx).ok_or(CommandError(format!("Error retrieving guild.")))?;
    let guild = guild.read();

    let channel_id =
        guild.channel_id_from_name(&ctx, &channelname).ok_or(CommandError(format!("No matching channel found.")))?;

    crate::database::set_notification_channel_id(guild.id.into(), Some(channel_id.into()));

    msg.channel_id.say_embed(&ctx, &format!("Added {} as notification channel.", &channelname));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn disable_notifications(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;

    crate::database::set_notification_channel_id(guild_id.into(), None);

    msg.channel_id.say_embed(&ctx, &format!("Disabled notifications."));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn add_trusted_user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    add_user_to_db(ctx, msg, args, "add_trusted_user", "trusted", "users")
}

#[command]
#[only_in(guilds)]
fn remove_trusted_user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    remove_user_from_db(ctx, msg, args, "remove_trusted_user", "trusted", "users")
}

#[command]
#[only_in(guilds)]
fn add_admin_user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    add_user_to_db(ctx, msg, args, "add_admin_user", "admin", "admins")
}

#[command]
#[only_in(guilds)]
fn remove_admin_user(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    remove_user_from_db(ctx, msg, args, "remove_admin_user", "admin", "admins")
}

#[command]
#[only_in(guilds)]
fn add_trusted_role(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    add_role_to_db(ctx, msg, args, "add_trusted_role", "trusted", "roles")
}

#[command]
#[only_in(guilds)]
fn remove_trusted_role(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    remove_role_from_db(ctx, msg, args, "remove_trusted_role", "trusted", "roles")
}

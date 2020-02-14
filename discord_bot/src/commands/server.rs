use crate::{commands::SendEmbed, database::open_guild_db, ReqwestClientContainer};
use rgcp_common::models::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[only_in(guilds)]
fn add_server(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: add_server "<name>" "<url>" "<api_key>"#);

    let name = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;
    let url = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;
    let api_key = args.single_quoted::<String>().map_err(|_| CommandError(usage))?;

    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let data = ctx.data.read();
    let client = data.get::<ReqwestClientContainer>().ok_or(CommandError(format!("Error retrieving http client.")))?;

    let response = client
        .post(&format!("{}/api/version", &url))
        .header("x-api-key", &api_key)
        .send()
        .map_err(|_| CommandError(format!("Couldn't reach the server.")))?
        .json::<VersionResponse>()
        .map_err(|_| CommandError(format!("Server sent invalid response.")))?;

    if response.api_version != API_VERSION {
        return Err(CommandError(format!(
            "Invalid api version, expected: {} actual: {}.",
            API_VERSION, response.api_version
        )));
    }

    let db = open_guild_db(guild_id.into(), "servers");

    if !db.insert(name.as_bytes(), format!("{}|{}", &url, &api_key).as_bytes()).is_ok() {
        return Err(CommandError(format!("Error adding {} to the list of servers.", &name)));
    }

    msg.channel_id.say_embed(&ctx.http, &format!("Succesfully added {} to the list of servers.", &name));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn remove_server(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: remove_server "<name>""#);

    let name = args.single_quoted::<String>().map_err(|_| CommandError(usage))?;
    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");

    if !db.contains_key(name.as_bytes()).unwrap_or(false) {
        return Err(CommandError(format!("User {} is not in the list servers.", &name)));
    }

    if !db.remove(name.as_bytes()).is_ok() {
        return Err(CommandError(format!("Error removing {} from the list servers.", &name)));
    }

    msg.channel_id.say_embed(&ctx.http, &format!("Removed {} from the list servers.", &name));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn list_servers(ctx: &mut Context, msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");

    let servers = db
        .iter()
        .filter_map(|result| result.ok().and_then(|(key, _)| String::from_utf8(key.to_vec()).ok()))
        .collect::<Vec<String>>()
        .join("\n");

    msg.channel_id.say_embed(&ctx.http, &servers);

    Ok(())
}

#[command]
#[only_in(guilds)]
fn start(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: start "<name>" "[map]" [max_players] "[creature1]" "[creature2]" "[creature3]"#);

    let name = args.single_quoted::<String>().map_err(|_| CommandError(usage))?;
    let map = args.single_quoted::<String>().unwrap_or("lv_canyon".to_owned());
    let max_players = args.single::<usize>().unwrap_or(10);
    let creature1 = args.single_quoted::<String>().ok();
    let creature2 = args.single_quoted::<String>().ok();
    let creature3 = args.single_quoted::<String>().ok();

    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");
    let server = String::from_utf8(
        db.get(name.as_bytes())
            .map_err(|_| CommandError(format!("Error accessing server db.")))?
            .ok_or(CommandError(format!("Unknown server.")))?
            .to_vec(),
    )?;

    let split = server.splitn(2, '|').map(|x| x.to_owned()).collect::<Vec<String>>();
    let (url, api_key) = (
        split.get(0).ok_or(CommandError(format!("Error retrieving server url from db.")))?.to_owned(),
        split.get(1).ok_or(CommandError(format!("Error retrieving server api key from db.")))?.to_owned(),
    );

    let data = ctx.data.read();
    let client = data.get::<ReqwestClientContainer>().ok_or(CommandError(format!("Error retrieving http client.")))?;

    let response = client
        .post(&format!("{}/api/start", &url))
        .header("x-api-key", &api_key)
        .json(&StartRequest {
            map: map,
            max_players: Some(max_players),
            creature0: creature1,
            creature1: creature2,
            creature2: creature3,
        })
        .send()
        .map_err(|_| CommandError(format!("Couldn't reach the server.")))?
        .json::<StartResponse>()
        .map_err(|_| CommandError(format!("Server sent invalid response.")))?;

    if let Some(err_msg) = response.error {
        return Err(CommandError(err_msg));
    }

    if let Some(open_url) = response.open_url {
        msg.channel_id.say_embed(&ctx.http, &format!("open {}", open_url));
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
fn kill(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: kill <server> <instance_id>"#);

    let server = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;
    let instance_id = args.single::<usize>().map_err(|_| CommandError(usage))?;

    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");
    let server = String::from_utf8(
        db.get(server.as_bytes())
            .map_err(|_| CommandError(format!("Error accessing server db.")))?
            .ok_or(CommandError(format!("Unknown server.")))?
            .to_vec(),
    )?;

    let split = server.splitn(2, '|').map(|x| x.to_owned()).collect::<Vec<String>>();
    let (url, api_key) = (
        split.get(0).ok_or(CommandError(format!("Error retrieving server url from db.")))?.to_owned(),
        split.get(1).ok_or(CommandError(format!("Error retrieving server api key from db.")))?.to_owned(),
    );

    let data = ctx.data.read();
    let client = data.get::<ReqwestClientContainer>().ok_or(CommandError(format!("Error retrieving http client.")))?;

    let response = client
        .post(&format!("{}/api/kill", &url))
        .header("x-api-key", &api_key)
        .json(&KillRequest { id: instance_id })
        .send()
        .map_err(|_| CommandError(format!("Couldn't reach the server.")))?
        .json::<KillResponse>()
        .map_err(|_| CommandError(format!("Server sent invalid response.")))?;

    if let Some(err_msg) = response.error {
        return Err(CommandError(err_msg));
    }

    msg.channel_id.say_embed(&ctx.http, &format!("Success"));

    Ok(())
}

#[command]
#[only_in(guilds)]
fn players(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: players <server> [instance_id]"#);

    let server = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;
    let instance_id = args.single::<u64>().unwrap_or(0);

    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");
    let server = String::from_utf8(
        db.get(server.as_bytes())
            .map_err(|_| CommandError(format!("Error accessing server db.")))?
            .ok_or(CommandError(format!("Unknown server.")))?
            .to_vec(),
    )?;

    let split = server.splitn(2, '|').map(|x| x.to_owned()).collect::<Vec<String>>();
    let url = split.get(0).ok_or(CommandError(format!("Error retrieving server url from db.")))?.to_owned();

    let data = ctx.data.read();
    let client = data.get::<ReqwestClientContainer>().ok_or(CommandError(format!("Error retrieving http client.")))?;

    let response = client
        .get(&format!("{}/api/players?id={}", &url, instance_id))
        .send()
        .map_err(|_| CommandError(format!("Couldn't reach the server.")))?
        .json::<Vec<PlayersResponse>>()
        .map_err(|_| CommandError(format!("Server sent invalid response.")))?;

    msg.channel_id.say_embed(
        &ctx.http,
        &format!(
            "Players:\n{}",
            response
                .iter()
                .map(|player| format!(
                    "{} -> {}",
                    player.name,
                    player.hero.as_ref().unwrap_or(&format!("Selecting").to_owned())
                ))
                .collect::<Vec<String>>()
                .join("\n")
        ),
    );

    Ok(())
}

#[command]
#[only_in(guilds)]
fn admin_pw(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let usage = format!(r#"Usage: admin_pw <server> [instance_id]"#);

    let server = args.single_quoted::<String>().map_err(|_| CommandError(usage.clone()))?;
    let instance_id = args.single::<usize>().unwrap_or(0);

    let guild_id = msg.guild_id.ok_or(CommandError(format!("Error retrieving guild.")))?;
    let db = open_guild_db(guild_id.into(), "servers");
    let server = String::from_utf8(
        db.get(server.as_bytes())
            .map_err(|_| CommandError(format!("Error accessing server db.")))?
            .ok_or(CommandError(format!("Unknown server.")))?
            .to_vec(),
    )?;

    let split = server.splitn(2, '|').map(|x| x.to_owned()).collect::<Vec<String>>();
    let (url, api_key) = (
        split.get(0).ok_or(CommandError(format!("Error retrieving server url from db.")))?.to_owned(),
        split.get(1).ok_or(CommandError(format!("Error retrieving server api key from db.")))?.to_owned(),
    );

    let data = ctx.data.read();
    let client = data.get::<ReqwestClientContainer>().ok_or(CommandError(format!("Error retrieving http client.")))?;

    let response = client
        .post(&format!("{}/api/admin_pw", &url))
        .header("x-api-key", &api_key)
        .json(&AdminPWRequest { id: instance_id })
        .send()
        .map_err(|_| CommandError(format!("Couldn't reach the server.")))?
        .json::<AdminPWResponse>()
        .map_err(|_| CommandError(format!("Server sent invalid response.")))?;

    let admin_pw = response
        .admin_pw
        .ok_or(CommandError(format!("No admin password available, are you sure the instance is running?")))?;

    let dm = msg.author.direct_message(&ctx, |m| m.content(&format!("Admin password for this instance: {}", admin_pw)));

    match dm {
        Ok(_) => {
            let _ = msg.react(&ctx, '👌');
        },
        Err(_) => {
            let _ = msg.reply(&ctx, "Could not send a DM.");
        },
    };

    Ok(())
}

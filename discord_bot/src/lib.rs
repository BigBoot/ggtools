mod commands;
mod database;

use commands::{admin::*, meta::*, server::*, SendEmbed, IS_ADMIN_CHECK, IS_TRUSTED_CHECK};
use dotenv::dotenv;
use log::{error, info};
use rgcp_common::{
    config::Config,
    models::{EventsRequest, EventsResponse},
};
use serenity::{
    client::{bridge::gateway::ShardManager, Client, Context, EventHandler},
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        HelpOptions,
        StandardFramework,
    },
    model::{
        event::ResumedEvent,
        gateway::Ready,
        id::{ChannelId, GuildId},
        prelude::{Message, UserId},
    },
    prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{sleep, JoinHandle},
    time::Duration,
};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct ReqwestClientContainer;

impl TypeMapKey for ReqwestClientContainer {
    type Value = reqwest::Client;
}

struct Handler {
    worker_handle: RwLock<Option<JoinHandle<()>>>,
    stop: Arc<AtomicBool>,
}

impl Handler {
    pub fn new() -> Self {
        return Handler { worker_handle: RwLock::new(None), stop: Arc::new(AtomicBool::new(false)) };
    }

    fn background_loop(ctx: &Context, timestamps: &mut HashMap<String, u128>) {
        let guild_ids = database::get_guild_ids().iter().map(|id| GuildId(*id)).collect::<Vec<GuildId>>();
        let http_client = reqwest::Client::new();

        for guild_id in guild_ids {
            if let (Some(notification_channel_id), Some(_)) =
                (database::get_notification_channel_id(guild_id.into()), guild_id.to_guild_cached(ctx))
            {
                let server_db = database::open_guild_db(guild_id.into(), "servers");

                let servers = server_db
                    .iter()
                    .filter_map(|e| {
                        e.ok().map(|(key, value)| {
                            (
                                String::from_utf8_lossy(&key.to_vec()).into_owned(),
                                String::from_utf8_lossy(&value.to_vec()).into_owned(),
                            )
                        })
                    })
                    .map(|(key, value)| (key, value.splitn(2, '|').map(|x| x.to_owned()).collect::<Vec<String>>()))
                    .filter_map(|(key, value)| {
                        if value.len() == 2 {
                            Some((key, value))
                        }
                        else {
                            None
                        }
                    })
                    .map(|(key, value)| (key, value.get(0).unwrap().clone(), value.get(1).unwrap().clone()))
                    .collect::<Vec<(String, String, String)>>();

                for (name, url, api_key) in servers {
                    let key = format!("{}|{}|{}", guild_id, &url, &api_key);
                    let timestamp = timestamps.get(&key).unwrap_or(&0);

                    let response = http_client
                        .post(&format!("{}/api/events", &url))
                        .header("x-api-key", &api_key)
                        .json(&EventsRequest { timestamp: *timestamp })
                        .send()
                        .ok()
                        .and_then(|mut resp| resp.json::<EventsResponse>().ok());

                    if let Some(event_response) = response {
                        let _ = timestamps.insert(key, event_response.timestamp);

                        for event in event_response.events {
                            ChannelId(notification_channel_id).say_embed(
                                &ctx.http,
                                &format!("Server {}({}): {}", &name, event.instance_id, &event.description),
                            );
                        }
                    }
                }
            }
        }
    }
}

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let stop = self.stop.clone();
        let mut worker_handle = self.worker_handle.write();
        *worker_handle = Some(std::thread::spawn(move || {
            let mut timestamps: HashMap<String, u128> = HashMap::new();
            while !stop.load(Ordering::SeqCst) {
                sleep(Duration::from_millis(500));
                Self::background_loop(&ctx, &mut timestamps);
            }
        }));
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);

        if let Some(worker_handle) = self.worker_handle.write().take() {
            worker_handle.join().expect("Error while waiting for background loop to exit");
        }
    }
}


group!({
    name: "admin",
    options: {
        checks: [is_admin]
    },
    commands: [
        add_trusted_user, 
        remove_trusted_user, 
        add_trusted_role, 
        remove_trusted_role, 
        add_admin_user, 
        remove_admin_user, 
        add_server, 
        remove_server,
        admin_pw,
        set_notification_channel,
        disable_notifications,
        kill,
    ],
});

group!({
    name: "trusted",
    options: {
        checks: [is_trusted]
    },
    commands: [
        start,
        list_servers,
    ],
});

group!({
    name: "everyone",
    commands: [
        ping,
        players,
    ],
});

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let available_groups: Vec<&'static CommandGroup> = groups
        .iter()
        .map(|e| *e)
        .filter(|group| {
            group.options.checks.iter().all(|check| {
                let function = check.function;
                return function(context, &msg, &mut args.clone(), &CommandOptions::default()).is_success();
            })
        })
        .collect();

    help_commands::with_embeds(context, msg, args, help_options, &available_groups, owners)
}

pub async fn run(config: Config) {
    dotenv().ok();

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    // env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token =
        env::var("DISCORD_TOKEN").ok().or_else(|| config.discord_token.get().clone()).expect("Discord token not set.");

    let mut client = Client::new(&token, Handler::new()).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ReqwestClientContainer>(reqwest::Client::new());
    }

    // let owners = match client.cache_and_http.http.get_current_application_info() {
    //     Ok(info) => {
    //         let mut set = HashSet::new();
    //         set.insert(info.owner.id);

    //         set
    //     },
    //     Err(why) => panic!("Couldn't get application info: {:?}", why),
    // };

    let owners = HashSet::default();

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.owners(owners).prefix("~"))
            .group(&ADMIN_GROUP)
            .group(&TRUSTED_GROUP)
            .group(&EVERYONE_GROUP)
            .help(&HELP)
            .after(|ctx, msg, _, err| {
                if let Err(cmd_err) = err {
                    msg.channel_id.send_embed(&ctx.http, |e| {
                        e.description(cmd_err.0);
                        e.color(serenity::utils::Colour::RED);
                    });
                }
            }),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

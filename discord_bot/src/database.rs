use lazy_static::lazy_static;
use std::convert::TryInto;

lazy_static! {
    static ref DB: sled::Db = sled::Config::new()
        .path("discord_bot.sled".to_owned())
        .flush_every_ms(Some(1000))
        .snapshot_after_ops(100_000)
        .cache_capacity(10_000_000_000)
        .open()
        .unwrap();
}

pub fn open_guild_db(guild_id: u64, db: &str) -> sled::Tree {
    let key: Vec<u8> =
        guild_id.to_be_bytes().iter().map(|b| *b).chain(std::iter::once(b'/')).chain(db.bytes()).collect();
    return DB.open_tree(&key).unwrap();
}

pub fn get_guild_ids() -> Vec<u64> {
    return DB
        .tree_names()
        .iter()
        .filter_map(|bytes| bytes[0..8].try_into().ok())
        .map(|bytes| u64::from_be_bytes(bytes))
        .collect();
}

pub fn set_notification_channel_id(guild_id: u64, notification_channel_id: Option<u64>) {
    if let Some(channel_id) = notification_channel_id {
        DB.open_tree("notification_channels".as_bytes())
            .unwrap()
            .insert(&guild_id.to_be_bytes(), &channel_id.to_be_bytes())
            .unwrap();
    }
    else {
        DB.open_tree("notification_channels".as_bytes()).unwrap().remove(&guild_id.to_be_bytes()).unwrap();
    }
}

pub fn get_notification_channel_id(guild_id: u64) -> Option<u64> {
    return DB
        .open_tree("notification_channels".as_bytes())
        .ok()
        .and_then(|db| db.get(&guild_id.to_be_bytes()).ok().and_then(|r| r))
        .map(|x| x.to_vec())
        .and_then(|x| x[0..8].try_into().ok())
        .map(|x| u64::from_be_bytes(x));
}

use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 2;

pub const EVENT_PLAYER_JOIN: &'static str = "PLAYER_JOIN";
pub const EVENT_PLAYER_LOCK: &'static str = "PLAYER_LOCK";
pub const EVENT_GUARDIAN_ATTACK: &'static str = "GUARDIAN_ATTACK";
pub const EVENT_SERVER_READY: &'static str = "SERVER_READY";
pub const EVENT_MATCH_STARTING: &'static str = "MATCH_STARTING";
pub const EVENT_MATCH_FINISHED: &'static str = "MATCH_FINISHED";

pub type InstanceID = usize;
pub type Timestamp = u128;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: String,
    pub instance_id: InstanceID,
    pub description: String,
    pub data: Option<String>,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvenDataServerReady {
    pub open_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvenDataMatchStarting {
    pub open_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub name: String,
    pub hero: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionResponse {
    pub app_version: String,
    pub app_version_major: String,
    pub app_version_minor: String,
    pub app_version_patch: String,
    pub api_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartRequest {
    pub map: String,
    pub max_players: Option<usize>,
    pub creature0: Option<String>,
    pub creature1: Option<String>,
    pub creature2: Option<String>,
    pub game_mod: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StartResponse {
    pub error: Option<String>,
    pub open_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KillRequest {
    pub id: InstanceID,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KillResponse {
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct PlayersResponse {
    pub name: String,
    pub hero: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminPWRequest {
    pub id: InstanceID,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminPWResponse {
    pub admin_pw: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventsRequest {
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventsResponse {
    pub timestamp: u128,
    pub events: Vec<Event>,
}

use crate::{api_key::ApiKey, server_manager::ServerManager, AppInfo};
use rgcp_common::{config::Config, models::*};
use rocket::{get, post, request::State};
use rocket_contrib::json::Json;
use std::sync::Arc;

#[post("/api/version")]
pub fn version(_api_key: ApiKey) -> Json<VersionResponse> {
    return Json(VersionResponse {
        app_version: AppInfo::get().version_string.clone(),
        app_version_major: AppInfo::get().pkg_version_major.clone(),
        app_version_minor: AppInfo::get().pkg_version_minor.clone(),
        app_version_patch: AppInfo::get().pkg_version_patch.clone(),
        api_version: API_VERSION,
    });
}

#[get("/api/logs?<id>&<from_line>&<to_line>")]
pub fn logs(
    server_manager: State<Arc<ServerManager>>,
    id: InstanceID,
    from_line: Option<u64>,
    to_line: Option<u64>,
) -> Json<Vec<String>> {
    return Json(server_manager.get_logs(id, from_line.unwrap_or(0), to_line.unwrap_or(std::u64::MAX)));
}

#[get("/api/players?<id>")]
pub fn get_players(server_manager: State<Arc<ServerManager>>, id: InstanceID) -> Json<Vec<Player>> {
    return Json(server_manager.get_players(id));
}

#[post("/api/start", data = "<data>")]
pub fn start(
    server_manager: State<Arc<ServerManager>>,
    config: State<Config>,
    data: Json<StartRequest>,
    _api_key: ApiKey,
) -> Json<StartResponse> {
    let creatures = [
        data.creature0.as_ref().unwrap_or(&config.default_creatures.get()[0]).to_owned(),
        data.creature1.as_ref().unwrap_or(&config.default_creatures.get()[1]).to_owned(),
        data.creature2.as_ref().unwrap_or(&config.default_creatures.get()[2]).to_owned(),
    ];

    if let Some(instance_id) = server_manager.start_new_instance(&data.map, &creatures, data.max_players.unwrap_or(10))
    {
        return Json(StartResponse {
            error: None,
            open_url: Some(format!("{}:{}", *config.server_url, *config.server_port + instance_id as u16)),
        });
    }

    return Json(StartResponse { error: Some("no instances available".to_owned()), open_url: None });
}

#[post("/api/kill", data = "<data>")]
pub fn kill(
    server_manager: State<Arc<ServerManager>>,
    data: Json<KillRequest>,
    _api_key: ApiKey,
) -> Json<KillResponse> {
    return Json(KillResponse { error: server_manager.kill_instance(data.id) });
}

#[post("/api/admin_pw", data = "<data>")]
pub fn admin_pw(
    server_manager: State<Arc<ServerManager>>,
    data: Json<AdminPWRequest>,
    _api_key: ApiKey,
) -> Json<AdminPWResponse> {
    return Json(AdminPWResponse { admin_pw: server_manager.get_admin_pw(data.id) });
}

#[post("/api/events", data = "<data>")]
pub fn events(
    server_manager: State<Arc<ServerManager>>,
    data: Json<EventsRequest>,
    _api_key: ApiKey,
) -> Json<EventsResponse> {
    let events = server_manager.get_events(data.timestamp);
    let timestamp = events.last().map(|e| e.timestamp).unwrap_or(data.timestamp);
    return Json(EventsResponse { events: events, timestamp: timestamp });
}

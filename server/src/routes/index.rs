use crate::{server_manager::ServerManager, templates::TERA};
use rgcp_common::{
    config::{Config, Creature, Map},
    AppInfo,
};
use rocket::{
    get,
    post,
    request::{Form, FromForm, State},
    response::{content::Html, Redirect},
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct IndexContext {
    title: String,
    running_instances: usize,
    max_instances: usize,
    app_version: String,
    creatures: Vec<Creature>,
    maps: Vec<Map>,
    default_creatures: Vec<String>,
}

#[get("/")]
pub fn get(config: State<Config>, server_manager: State<Arc<ServerManager>>) -> Html<String> {
    let context = IndexContext {
        title: config.title.get().to_owned(),
        running_instances: server_manager.running_instances(),
        max_instances: *config.max_instances,
        app_version: AppInfo::get().version_string.to_owned(),
        creatures: config.creatures.get().clone(),
        maps: config.maps.get().clone(),
        default_creatures: config.default_creatures.get().clone(),
    };

    let html = TERA.render("index", &tera::Context::from_serialize(context).unwrap()).unwrap();
    Html(html)
}


#[derive(FromForm)]
pub struct StartForm {
    map: String,
    max_players: usize,
    creature0: String,
    creature1: String,
    creature2: String,
}

#[post("/start", data = "<form>")]
pub fn start(form: Form<StartForm>, server_manager: State<Arc<ServerManager>>) -> Redirect {
    if let Some(id) = server_manager.start_new_instance(
        &form.map,
        &[form.creature0.clone(), form.creature1.clone(), form.creature2.clone()],
        form.max_players,
    ) {
        return Redirect::to(format!("/instance?id={}", id));
    }
    return Redirect::to("/");
}

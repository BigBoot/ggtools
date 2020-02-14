use crate::templates::TERA;
use rgcp_common::{config::Config, models::InstanceID, AppInfo};
use rocket::{get, request::State, response::content::Html};
use serde::Serialize;

#[derive(Serialize)]
struct InstanceContext {
    title: String,
    app_version: String,
    instance_id: InstanceID,
    instance_port: u16,
    server_url: String,
}

#[get("/instance?<id>")]
pub fn instance(id: InstanceID, config: State<Config>) -> Html<String> {
    let context = InstanceContext {
        title: config.title.get().to_owned(),
        app_version: AppInfo::get().version_string.to_owned(),
        server_url: config.server_url.get().to_owned(),
        instance_id: id,
        instance_port: *config.server_port + id as u16,
    };

    let html = TERA.render("instance", &tera::Context::from_serialize(context).unwrap()).unwrap();
    Html(html)
}

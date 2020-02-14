pub mod api;
pub mod index;
pub mod instance;
pub mod static_files;

use rocket::{routes, Route};

pub fn get() -> Vec<Route> {
    return routes![
        index::get,
        index::start,
        static_files::static_file,
        api::version,
        api::logs,
        api::get_players,
        api::start,
        api::kill,
        api::admin_pw,
        api::events,
        instance::instance
    ];
}

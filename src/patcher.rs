#![windows_subsystem = "windows"]
mod app_info;

fn main() {
    app_info::init();

    let config = rgcp_common::config::Config::load();

    rgcp_patcher::run(config);
}

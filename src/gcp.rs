//#![windows_subsystem = "windows"]

mod app_info;

use futures::executor::block_on;

fn main() {
    app_info::init();

    let config = rgcp_common::config::Config::load();

    block_on(rgcp_server::run(config));
}

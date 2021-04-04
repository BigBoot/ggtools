#![feature(proc_macro_hygiene, decl_macro)]

mod api_key;
mod assets;
mod embed_file;
mod routes;
mod server_manager;
mod templates;
mod mods;

use crate::{assets::Assets, server_manager::ServerManager};
use rgcp_common::{config::Config, AppInfo};
use rocket::{config::Environment, fairing::AdHoc};
use std::sync::Arc;

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn setup_conf(_conf: &mut rocket::config::Config) {}

const PAKKO: &'static str = r#"
                                       ;++++###
                                     +++++#####'
                                   ;++++++#########
         `                               ```....,#@#      .+'++++
       '''+#                    ``  `::::'`      ``:: +++++++##++##
  '''#:'+###@;:               ````;:,,,,;'+`     ```,;+'''''########
  '####;@###;;;;',         ```@@ ::,,,,,;'+ ````` ``.,:;;''+++########
   @##+';;;'';;'''#    ``     ``:::,:,:;'+#`````````.:;'+'+++@#+++####
     '''''';:;+''##+; .`````````,;:''##+'+````#@'```,:;'+@@###+###@###
''''':;;;;:::;;###++++`````+'`````````````  `````````:::;'@@@@@@@##@@@
 ####;::::::;'++++++++````+##;''+,',,.,:,.```````````.,,,:;';;@@@@@@@@
     `:::::'++++++++++````;####@+@@@@'+','++#:````....,:::;''';;     @
       '''++++++++++++`````+#`.;+#@+;;;;++@@++.``.:,:::;;';''''''
        +++++++++++++',`````'#`##::::::,;;;:':```.,::::;''''''''''
         `++++++++'''''``````.'',.. `:`;: `+````.,,::::;'''''+''''''
           '''''''''''''```````  ;'''''; ``````.,:,::;;'''''++++'''''
             ''''''''''':`````````````````````,,::::;''''''++++++++'''
               `''''''';;'`````````````````.,,:::;;''''++++++;;;;;+++'
                  :';;;;;;,`````````````,,,,::::;''''+++++++;;;;;;;;;'
                   ,,,,,,,,.:``````.,,,,,:::::;;'''''+++++;;;;;;,.....
                    ,,,,,,...,;:,,,,:::::;''''''''''+++''''';;.......,
                     ,,,,,..```.::::;;;;;'''+++#'++'''''''''.....,,,,,
"#;

#[cfg(not(debug_assertions))]
fn setup_conf(conf: &mut rocket::config::Config) {
    conf.set_secret_key("4m1AMRPsjmvgPOALTda4VwGA3bNTWs68/whPsBW9PyY=").unwrap();
    conf.log_level = rocket::logger::LoggingLevel::Off;
}

pub async fn run(config: Config) {
    println!("{}", PAKKO);
    println!("Welcome to BigBoot's Gigantic Control Panel V{}", AppInfo::get().version_string);

    #[cfg(debug_assertions)]
    let env = Environment::Development;

    #[cfg(not(debug_assertions))]
    let env = Environment::Production;

    let mut rocket_conf = rocket::config::Config::build(env)
        .extra("template_dir", "assets/templates".to_owned())
        .extra("static_dir", "assets/static".to_owned())
        .address("0.0.0.0".to_owned())
        .port(*config.http_port.get())
        .finalize()
        .unwrap();

    setup_conf(&mut rocket_conf);

    let server_manager = ServerManager::new(config.clone()).unwrap();

    rocket::custom(rocket_conf)
        .mount("/", routes::get())
        .attach(AdHoc::on_attach("Assets Config", |rocket| {
            let assets_dir = rocket.config().get_str("static_dir").unwrap_or("assets/static").to_owned();
            Ok(rocket.manage(routes::static_files::StaticDir(assets_dir)))
        }))
        .manage(config)
        .manage(Arc::new(server_manager))
        .launch();
}

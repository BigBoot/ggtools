use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::{Captures, Regex};
use rgcp_common::{config::Config, models::*};
use serde::Serialize;
use std::{env::current_dir, fs, path::PathBuf, time::SystemTime};

trait DBKey {
    fn db_key(&self, prefix: &str) -> Vec<u8>;
}

impl DBKey for InstanceID {
    fn db_key(&self, prefix: &str) -> Vec<u8> {
        return prefix.as_bytes().iter().chain(self.to_be_bytes().iter()).map(|b| *b).collect();
    }
}

fn now() -> u128 {
    let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    return (duration.as_secs() as u128) * 1000 + duration.subsec_millis() as u128;
}

#[cfg(target_os = "windows")]
struct Handle(winapi::shared::ntdef::HANDLE);

#[cfg(not(target_os = "windows"))]
struct Handle();

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

#[cfg_attr(not(windows), allow(dead_code))]
pub struct ServerManager {
    config: Config,
    job_handle: Handle,
    db: sled::Db,
}

#[cfg_attr(not(windows), allow(dead_code))]
impl ServerManager {
    pub fn new(mut config: Config) -> Option<Self> {
        if config.gigantic_path.is_none() {
            if let Ok(current_dir) = current_dir() {
                if current_dir.join("RxGame-Win64-Test.exe").exists() {
                    config
                        .gigantic_path
                        .set(Some(current_dir.parent().unwrap().parent().unwrap().to_string_lossy().into_owned()));
                }
                else if current_dir.join("Binaries").join("Win64").join("RxGame-Win64-Test.exe").exists() {
                    config.gigantic_path.set(Some(current_dir.to_string_lossy().into_owned()));
                }
            }
        }

        if config.gigantic_path.is_none() {
            println!("Gigantic path is not set, please set \"gigantic_path\" in your config.json.");
            return None;
        }

        let server_manager = ServerManager {
            config: config,
            job_handle: ServerManager::init_job_handle(),
            db: sled::Config::new().cache_capacity(10_000_000_000).temporary(true).open().unwrap(),
        };

        server_manager.unlock_all_instances();

        if !server_manager.binary_path("RxGame-Win64-Test.exe").exists() {
            println!("RxGame-Win64-Test.exe not found, please make sure \"gigantic_path\" is set in your config.json.");
            return None;
        }

        return Some(server_manager);
    }

    pub fn running_instances(&self) -> usize {
        let max_instances = *self.config.max_instances.get();
        (0..max_instances).filter(|x| self.is_locked(*x)).count()
    }

    pub fn start_new_instance(&self, map: &str, creatures: &[String], max_players: usize) -> Option<InstanceID> {
        let id = self.try_get_instance()?;
        let port = *self.config.server_port + id as u16;
        self.generate_config_file(id, creatures, max_players);
        self.run_instance(id, map, port);

        return Some(id);
    }

    pub fn kill_instance(&self, id: InstanceID) -> Option<String> {
        if !self.is_locked(id) {
            return Some(format!("Invalid instance"));
        }

        if let Ok(db) = self.db.open_tree("kill") {
            return match db.insert(id.to_be_bytes(), &[]) {
                Ok(_) => None,
                _ => Some(format!("Couldn't kill server")),
            };
        }

        return Some(format!("Couldn't kill server"));
    }

    pub fn get_admin_pw(&self, id: InstanceID) -> Option<String> {
        if !self.is_locked(id) {
            return None;
        }

        if let Ok(db) = self.db.open_tree("admin_pws") {
            if let Ok(Some(content)) = db.get(id.to_be_bytes()) {
                return Some(String::from_utf8_lossy(&content).into_owned());
            }
        }

        return None;
    }

    pub fn get_logs(&self, id: InstanceID, from_line: u64, to_line: u64) -> Vec<String> {
        return self
            .db
            .open_tree(id.db_key("logs"))
            .unwrap()
            .range(from_line.to_be_bytes()..to_line.to_be_bytes())
            .map(|l| String::from_utf8_lossy(&l.unwrap().1).into_owned())
            .collect();
    }

    pub fn get_players(&self, id: InstanceID) -> Vec<Player> {
        return self
            .db
            .open_tree(id.db_key("players"))
            .unwrap()
            .iter()
            .map(|b| serde_cbor::from_slice(&b.unwrap().1).unwrap())
            .collect();
    }

    pub fn get_events(&self, last_timestamp: u128) -> Vec<Event> {
        let max_instances = *self.config.max_instances.get();
        return (0..max_instances)
            .filter_map(|id| self.db.open_tree(id.db_key("events")).ok())
            .flat_map(|tree| {
                tree.range((last_timestamp + 1).to_be_bytes()..)
                    .map(|b| serde_cbor::from_slice(&b.unwrap().1).unwrap())
                    .collect::<Vec<Event>>()
            })
            .collect();
    }

    #[cfg(target_os = "windows")]
    fn init_job_handle() -> Handle {
        use std::ptr::null_mut;
        use winapi::{
            ctypes::c_void,
            um::{
                jobapi2::{CreateJobObjectW, SetInformationJobObject},
                winnt::{
                    JobObjectExtendedLimitInformation,
                    JOBOBJECT_BASIC_LIMIT_INFORMATION,
                    JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
                    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                },
            },
        };

        let job_handle = unsafe { CreateJobObjectW(null_mut(), null_mut()) };

        let mut jobobject_extended_limit_information = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                LimitFlags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                ..JOBOBJECT_BASIC_LIMIT_INFORMATION::default()
            },
            ..JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default()
        };

        unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                &mut jobobject_extended_limit_information as *mut _ as *mut c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
        };

        return Handle(job_handle);
    }

    #[cfg(not(target_os = "windows"))]
    fn init_job_handle() -> Handle {
        Handle()
    }

    fn instance_path(&self, id: InstanceID) -> PathBuf {
        current_dir().unwrap().join("instances").join(id.to_string())
    }

    fn game_path(&self) -> PathBuf {
        PathBuf::from(self.config.gigantic_path.get().as_ref().unwrap())
    }

    fn binary_path(&self, binary: &str) -> PathBuf {
        self.game_path().join("Binaries").join("Win64").join(binary)
    }

    fn config_path(&self, config: &str) -> PathBuf {
        self.game_path().join("RxGame").join("Config").join(config)
    }

    fn log_path(id: InstanceID) -> PathBuf {
        dirs::document_dir()
            .unwrap()
            .join("My Games")
            .join("Gigantic")
            .join("RxGame")
            .join("Logs")
            .join(format!("gcp_{}.log", id))
    }

    fn create_event(db: &sled::Tree, id: &str, instance_id: InstanceID, description: String) {
        Self::create_event_ex(db, id, instance_id, description, &{});
    }

    fn create_event_ex(db: &sled::Tree, id: &str, instance_id: InstanceID, description: String, data: &impl Serialize) {
        let timestamp = now();
        let event = Event {
            id: id.to_owned(),
            instance_id: instance_id,
            description: description,
            data: serde_json::to_string(data).ok(),
            timestamp: timestamp,
        };

        db.insert(timestamp.to_be_bytes(), serde_cbor::to_vec(&event).unwrap()).unwrap();
    }

    fn is_locked(&self, id: InstanceID) -> bool {
        self.instance_path(id).exists()
    }

    fn find_free_instance(&self) -> Option<InstanceID> {
        let max_instances = *self.config.max_instances.get();
        (0..max_instances).find(|x| !self.is_locked(*x))
    }

    fn try_get_instance(&self) -> Option<InstanceID> {
        self.find_free_instance().and_then(|id| fs::create_dir_all(self.instance_path(id)).map(|_| id).ok())
    }

    fn unlock_instance(&self, id: InstanceID) {
        if self.is_locked(id) {
            fs::remove_dir_all(self.instance_path(id)).unwrap()
        }
    }

    fn unlock_all_instances(&self) {
        let max_instances = *self.config.max_instances.get();
        for id in 0..max_instances {
            self.unlock_instance(id);
        }
    }

    fn generate_config_file(&self, id: InstanceID, creatures: &[String], max_players: usize) {
        lazy_static! {
            static ref RE_CREATURE: Regex = Regex::new(r#"DefaultMinionLoadout\[(\d+)]="\w*""#).unwrap();
            static ref RE_MAX_PLAYERS: Regex = Regex::new(r#"MaxPlayers=\d*"#).unwrap();
            static ref RE_ADMIN_PASSWORD: Regex = Regex::new(r#"AdminPassword=\w*"#).unwrap();
        }

        let creature_details: Vec<rgcp_common::config::Creature> = creatures
            .iter()
            .filter_map(|id| {
                self.config
                    .creatures
                    .get()
                    .iter()
                    .find(|creature| creature.id == *id)
                    .and_then(|creature| Some(creature.clone()))
            })
            .collect();

        let config_path = self.config_path("DefaultGame.ini");

        let mut config = fs::read_to_string(config_path).unwrap();

        config = creature_details
            .iter()
            .enumerate()
            .fold(config, |config, (i, creature)| {
                RE_CREATURE
                    .replace_all(&config, |captures: &Captures| {
                        let n: usize = captures.get(1).unwrap().as_str().parse().unwrap();
                        match n {
                            n if n == i => format!("DefaultMinionLoadout[{}]={}", n, creature.baby),
                            n if n == i + 3 => format!("DefaultMinionLoadout[{}]={}", n, creature.adult),
                            _ => captures.get(0).unwrap().as_str().to_owned(),
                        }
                    })
                    .into_owned()
            })
            .to_owned();

        config = RE_MAX_PLAYERS
            .replace_all(&config, |_: &Captures| format!("MaxPlayers={}", max_players).to_owned())
            .into_owned();

        let admin_pw = thread_rng().sample_iter(&Alphanumeric).take(16).collect::<String>();

        let db = self.db.open_tree("admin_pws").unwrap();
        db.insert(id.to_be_bytes(), admin_pw.as_bytes()).unwrap();

        config = RE_ADMIN_PASSWORD
            .replace_all(&config, |_: &Captures| format!("AdminPassword={}", admin_pw).to_owned())
            .into_owned();

        fs::write(self.instance_path(id).join("DefaultGame.ini"), config).unwrap();
    }

    fn run_instance(&self, id: InstanceID, map: &str, port: u16) {
        if Self::log_path(id).exists() {
            std::fs::remove_file(Self::log_path(id)).unwrap();
        }

        let process_handle = self.start_process(&format!(
            "{} server {}?listen?port={} -dedicated -defgameini=\"{}\" -log=gcp_{}.log -forcelogflush",
            self.binary_path("RxGame-Win64-Test.exe").to_string_lossy(),
            map,
            port,
            self.instance_path(id).join("DefaultGame.ini").to_string_lossy(),
            id
        ));

        if let Some(process_handle) = process_handle {
            self.watch_process(process_handle, id);
        }
    }

    #[cfg(target_os = "windows")]
    fn watch_process(&self, handle: Handle, id: InstanceID) {
        use std::io::{BufRead, Seek};
        use winapi::um::{
            minwinbase::STILL_ACTIVE,
            processthreadsapi::{GetExitCodeProcess, TerminateProcess},
        };

        lazy_static! {
            static ref RE_PLAYER_JOINED: Regex = Regex::new(r#"DevNet: Join succeeded: (.*?) playerid="#).unwrap();
            static ref RE_PLAYER_LOCKED: Regex = Regex::new(r#"RxPlayerController::PlayerWaiting:FinishLockingCharacterSelection (.*?) LOCKED HeroProviderIndex:'\d+' RxGameContent\.RxPawn_(.*?)$"#).unwrap();
            static ref RE_PREROUND_BEGIN: Regex = Regex::new(r#"-------------> PREROUND BeginState <-----------------"#).unwrap();
            static ref RE_PREROUND_END: Regex = Regex::new(r#"Starting match..."#).unwrap();
        }

        let logs = self.db.open_tree(id.db_key("logs")).unwrap();
        let events = self.db.open_tree(id.db_key("events")).unwrap();
        let players = self.db.open_tree(id.db_key("players")).unwrap();
        let kill = self.db.open_tree("kill").unwrap();
        let admin_pws = self.db.open_tree("admin_dbs").unwrap();
        let instance_path = self.instance_path(id);

        let _ = kill.remove(id.to_be_bytes());
        let _ = events.clear();

        let port = *self.config.server_port;
        let url = self.config.server_url.get().clone();
        let open_url = if id == 0 && port == 7777 { url } else { format!("{}:{}", url, port) };
        Self::create_event_ex(
            &events,
            EVENT_SERVER_READY,
            id,
            format!("Server started, please connect using \"open {}\"", &open_url),
            &EvenDataServerReady { open_url: open_url },
        );


        std::thread::spawn(move || {
            let _cleanup = rgcp_common::utils::DropGuard::new(Some(instance_path), |instance_path| {
                let _ = logs.clear();
                let _ = players.clear();
                let _ = admin_pws.remove(id.to_be_bytes());
                let _ = kill.remove(id.to_be_bytes());

                if let Some(instance_path) = instance_path {
                    println!("Cleaning up Server I{}", id);
                    let _ = fs::remove_dir_all(instance_path);
                }
            });

            let mut exit_code = STILL_ACTIVE;
            let mut last_line: u64 = 0;
            let mut last_size: u64 = 0;
            while exit_code == STILL_ACTIVE {
                std::thread::sleep(std::time::Duration::from_millis(250));

                if kill.contains_key(id.to_be_bytes()).unwrap_or(false) {
                    unsafe {
                        TerminateProcess(handle.0, 1);
                    }
                }

                if let Ok(log_file) = std::fs::OpenOptions::new().read(true).open(Self::log_path(id)) {
                    if let Ok(metadata) = log_file.metadata() {
                        let size = metadata.len();

                        let mut reader = std::io::BufReader::new(log_file);

                        if size < last_size {
                            logs.clear().unwrap();
                            last_line = 0;
                            last_size = 0;
                        }

                        reader.seek(std::io::SeekFrom::Start(last_size)).unwrap();

                        reader.lines().map(|l| l.unwrap()).for_each(|line| {
                            println!("I{} {}: {}", id, last_line as u64, line);

                            if RE_PLAYER_JOINED.is_match(&line) {
                                let cap = RE_PLAYER_JOINED.captures(&line).unwrap();
                                let player = Player { name: cap.get(1).unwrap().as_str().to_owned(), hero: None };

                                println!("Player joined: {}", &player.name);

                                players
                                    .insert(
                                        cap.get(1).unwrap().as_str().as_bytes(),
                                        serde_cbor::to_vec(&player).unwrap(),
                                    )
                                    .unwrap();

                                Self::create_event(
                                    &events,
                                    EVENT_PLAYER_JOIN,
                                    id,
                                    format!("Player {} joined", &player.name),
                                );
                            }

                            if RE_PLAYER_LOCKED.is_match(&line) {
                                let cap = RE_PLAYER_LOCKED.captures(&line).unwrap();
                                let player = Player {
                                    name: cap.get(1).unwrap().as_str().to_owned(),
                                    hero: Some(cap.get(2).unwrap().as_str().to_owned()),
                                };

                                println!(
                                    "Player locked: {} -> {}",
                                    cap.get(1).unwrap().as_str().to_owned(),
                                    cap.get(2).unwrap().as_str().to_owned()
                                );

                                Self::create_event(
                                    &events,
                                    EVENT_PLAYER_LOCK,
                                    id,
                                    format!(
                                        "Player {} locked char {}",
                                        &player.name,
                                        &cap.get(2).unwrap().as_str().to_owned()
                                    ),
                                );

                                players
                                    .insert(
                                        cap.get(1).unwrap().as_str().as_bytes(),
                                        serde_cbor::to_vec(&player).unwrap(),
                                    )
                                    .unwrap();
                            }

                            if RE_PREROUND_END.is_match(&line) {
                                Self::create_event(&events, EVENT_MATCH_STARTING, id, format!("Match starting"));
                            }


                            logs.insert((last_line).to_be_bytes(), line.into_bytes()).unwrap();
                            last_line += 1;
                        });

                        last_size = size;
                    }
                }

                unsafe { GetExitCodeProcess(handle.0, &mut exit_code) };
            }

            println!("Server I{} finished with code: {}", id, exit_code);

            let _ = events.clear();
            Self::create_event(&events, EVENT_MATCH_FINISHED, id, format!("Match finished"));
        });
    }

    #[cfg(not(target_os = "windows"))]
    fn watch_process(&self, _handle: Handle, _id: InstanceID) {}

    #[cfg(target_os = "windows")]
    fn start_process(&self, cmd: &str) -> Option<Handle> {
        use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, ptr::null_mut};
        use winapi::um::{
            handleapi::CloseHandle,
            jobapi2::AssignProcessToJobObject,
            processthreadsapi::{CreateProcessW, TerminateProcess, PROCESS_INFORMATION, STARTUPINFOW},
            winbase::STARTF_USESHOWWINDOW,
        };

        const SW_HIDE: winapi::ctypes::c_ushort = 0;

        let mut startup_info = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE,
            ..STARTUPINFOW::default()
        };
        let mut process_information = PROCESS_INFORMATION::default();
        let mut cmd_wide: Vec<u16> = OsStr::new(cmd).encode_wide().chain(once(0)).collect();
        let ok = unsafe {
            CreateProcessW(
                null_mut(),
                cmd_wide.as_mut_ptr(),
                null_mut(),
                null_mut(),
                0,
                0,
                null_mut(),
                null_mut(),
                &mut startup_info,
                &mut process_information,
            ) != 0
        };
        if ok {
            unsafe {
                AssignProcessToJobObject(self.job_handle.0, process_information.hProcess);
            }
            return Some(Handle(process_information.hProcess));
        }
        else {
            unsafe {
                TerminateProcess(process_information.hProcess, 0);
                CloseHandle(process_information.hProcess);
                CloseHandle(process_information.hThread);
                process_information.hProcess = null_mut();
            }
            return None;
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn start_process(&self, _cmd: &str) -> Option<Handle> {
        Some(Handle())
    }
}

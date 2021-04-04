use crate::default_option::DefaultOption;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub baby: String,
    pub adult: String,
    pub family: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_http_port")]
    pub http_port: DefaultOption<u16>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_gigantic_path")]
    pub gigantic_path: DefaultOption<Option<String>>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_server_url")]
    pub server_url: DefaultOption<String>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_server_port")]
    pub server_port: DefaultOption<u16>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_max_instances")]
    pub max_instances: DefaultOption<usize>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_title")]
    pub title: DefaultOption<String>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_default_creatures")]
    pub default_creatures: DefaultOption<Vec<String>>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_maps")]
    pub maps: DefaultOption<Vec<Map>>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_creatures")]
    pub creatures: DefaultOption<Vec<Creature>>,

    #[serde(skip_serializing_if = "DefaultOption::is_default")]
    #[serde(default = "default_api_key")]
    pub api_key: DefaultOption<Option<String>>,
}

fn default_http_port() -> DefaultOption<u16> {
    return DefaultOption::with_default(80);
}

fn default_gigantic_path() -> DefaultOption<Option<String>> {
    return DefaultOption::with_default(None);
}

fn default_server_url() -> DefaultOption<String> {
    return DefaultOption::with_default("127.0.0.1".to_owned());
}

fn default_server_port() -> DefaultOption<u16> {
    return DefaultOption::with_default(7777);
}

fn default_max_instances() -> DefaultOption<usize> {
    return DefaultOption::with_default(1);
}

fn default_title() -> DefaultOption<String> {
    return DefaultOption::with_default("Gigantic Control Panel".to_owned());
}

fn default_maps() -> DefaultOption<Vec<Map>> {
    return DefaultOption::with_default(vec![
        Map { id: "lv_canyon".to_owned(), name: "Ghost Reef".to_owned() },
        Map { id: "lv_mistforge".to_owned(), name: "Sanctum Falls".to_owned() },
        Map { id: "lv_valley".to_owned(), name: "Sirens Strand".to_owned() },
        Map { id: "lv_wizardwoods".to_owned(), name: "Ember Grove (Unfinished/Prototype)".to_owned() },
        Map { id: "lv_canyonnight".to_owned(), name: "Ghost Reef (Unfinished/Prototype)".to_owned() },
        Map { id: "lv_skycityv2".to_owned(), name: "Sky City V2 (Unfinished/Prototype)".to_owned() },
        Map { id: "lv_skytuga".to_owned(), name: "Sky Tuga (Unfinished/Prototype)".to_owned() },
    ]);
}

fn default_creatures() -> DefaultOption<Vec<Creature>> {
    return DefaultOption::with_default(vec![
        Creature {
            id: "cerb".to_owned(),
            name: "Cerberus".to_owned(),
            baby: "CerberusBaby".to_owned(),
            adult: "CerberusAdult".to_owned(),
            family: "cerb".to_owned(),
        },
        Creature {
            id: "cerb_shadow".to_owned(),
            name: "Shadow Cerberus".to_owned(),
            baby: "CerberusBaby_Shadow".to_owned(),
            adult: "CerberusShadow".to_owned(),
            family: "cerb".to_owned(),
        },
        Creature {
            id: "cerb_stone".to_owned(),
            name: "Stone Cerberus".to_owned(),
            baby: "CerberusBaby_Tough".to_owned(),
            adult: "CerberusTough".to_owned(),
            family: "cerb".to_owned(),
        },
        Creature {
            id: "cyclops".to_owned(),
            name: "Mountain Cyclops".to_owned(),
            baby: "CyclopsBaby".to_owned(),
            adult: "CyclopsAdult".to_owned(),
            family: "cyclops".to_owned(),
        },
        Creature {
            id: "cyclops_yeti".to_owned(),
            name: "Yeti Cyclops".to_owned(),
            baby: "CyclopsBaby_Frost".to_owned(),
            adult: "CyclopsFrost".to_owned(),
            family: "cyclops".to_owned(),
        },
        Creature {
            id: "cyclops_riftborn".to_owned(),
            name: "Riftborn Cyclops".to_owned(),
            baby: "CyclopsBaby_Magic".to_owned(),
            adult: "CyclopsMagic".to_owned(),
            family: "cyclops".to_owned(),
        },
        Creature {
            id: "bloomer".to_owned(),
            name: "Summer Bloomer".to_owned(),
            baby: "EntBaby".to_owned(),
            adult: "EntAdult".to_owned(),
            family: "bloomer".to_owned(),
        },
        Creature {
            id: "bloomer_winter".to_owned(),
            name: "Winter Bloomer".to_owned(),
            baby: "EntBaby_Winter".to_owned(),
            adult: "EntWinter".to_owned(),
            family: "bloomer".to_owned(),
        },
        Creature {
            id: "bloomer_spring".to_owned(),
            name: "Spring Bloomer".to_owned(),
            baby: "EntBaby_Spring".to_owned(),
            adult: "EntSpring".to_owned(),
            family: "bloomer".to_owned(),
        },
        Creature {
            id: "bloomer_autumn".to_owned(),
            name: "Autumn Bloomer".to_owned(),
            baby: "EntBaby_Fall".to_owned(),
            adult: "EntFall".to_owned(),
            family: "bloomer".to_owned(),
        },
        Creature {
            id: "dragon".to_owned(),
            name: "Fire Dragon".to_owned(),
            baby: "DragonBaby".to_owned(),
            adult: "DragonAdult".to_owned(),
            family: "dragon".to_owned(),
        },
        Creature {
            id: "dragon_storm".to_owned(),
            name: "Storm Dragon".to_owned(),
            baby: "DragonBaby_Storm".to_owned(),
            adult: "DragonStorm".to_owned(),
            family: "dragon".to_owned(),
        },
        Creature {
            id: "obelisk".to_owned(),
            name: "Ancient Obelisk".to_owned(),
            baby: "ObeliskBaby".to_owned(),
            adult: "ObeliskAdult".to_owned(),
            family: "obelisk".to_owned(),
        },
        Creature {
            id: "infernal".to_owned(),
            name: "Crimson Infernal".to_owned(),
            baby: "DemonBaby".to_owned(),
            adult: "DemonAdult".to_owned(),
            family: "infernal".to_owned(),
        },
        Creature {
            id: "infernal_void".to_owned(),
            name: "Void Infernal".to_owned(),
            baby: "DemonBaby".to_owned(),
            adult: "DemonVoid".to_owned(),
            family: "infernal".to_owned(),
        },
        Creature {
            id: "infernal_savage".to_owned(),
            name: "Savage Infernal".to_owned(),
            baby: "DemonBaby".to_owned(),
            adult: "DemonOni".to_owned(),
            family: "infernal".to_owned(),
        },
    ]);
}

fn default_default_creatures() -> DefaultOption<Vec<String>> {
    return DefaultOption::with_default(vec!["bloomer".to_owned(), "cerb".to_owned(), "cyclops".to_owned()]);
}

fn default_api_key() -> DefaultOption<Option<String>> {
    return DefaultOption::with_default(None);
}

impl Default for Config {
    fn default() -> Self {
        return serde_json::from_str("{}").unwrap();
    }
}

impl Config {
    pub fn load() -> Self {
        return std::fs::read_to_string("config.json").map(|json| serde_json::from_str(&json).unwrap()).unwrap_or_else(
            |_| {
                println!("No config.json found, using default values...");
                return Config::default();
            },
        );
    }
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModMetadata {
    pub name: String,
    pub description: String,
    pub creatures: Option<Vec<String>>,
    pub map: Option<String>,
    pub number_of_players: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub id: String,
    pub meta: ModMetadata,
}


pub fn get_mods() -> Vec<Mod> {
    return glob::glob("mods/**/mod.toml")
        .expect("Failed to load mods")
        .filter_map(Result::ok)
        .filter_map(|file| file.parent().map(|path| path.to_owned()))
        .filter_map(|path| path.file_name().and_then(|name| name.to_owned().into_string().ok()))
        .filter_map(|name| get_mod(&name).ok())
        .collect();
}

pub fn get_mod(name: &str) -> Result<Mod> {
    let content = fs::read_to_string(PathBuf::new().join("mods").join(name).join("mod.toml"))?;
    toml::from_str(&content)
        .with_context(|| format!("Error parsing mods/{}/mod.toml", name))
        .map(|meta| Mod { id: name.to_owned(), meta })
}

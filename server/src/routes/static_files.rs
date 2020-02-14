use rocket::get;
use std::path::{Path, PathBuf};

use crate::{embed_file::EmbedFile, Assets};

pub struct StaticDir(pub String);

#[get("/static/<file..>")]
pub fn static_file(file: PathBuf) -> Option<EmbedFile> {
    return EmbedFile::open::<Assets>(&Path::new("static").join(file).to_string_lossy());
}

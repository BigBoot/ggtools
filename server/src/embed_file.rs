use rocket::{
    http::ContentType,
    request::Request,
    response::{self, Responder, Stream},
};
use rust_embed::RustEmbed;
use std::{borrow::Cow, io::Cursor};

#[derive(Debug)]
pub struct EmbedFile(Cow<'static, [u8]>, Option<ContentType>);

impl EmbedFile {
    pub fn open<'a, T: RustEmbed>(file: &str) -> Option<EmbedFile> {
        if let Some(content) = T::get(&file.replace("\\", "/")) {
            let content_type = std::path::Path::new(file)
                .extension()
                .and_then(|extension| ContentType::from_extension(&extension.to_string_lossy()));

            return Some(EmbedFile(content, content_type));
        }
        return None;
    }
}

impl<'r> Responder<'r> for EmbedFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let mut response = Stream::from(Cursor::new(self.0)).respond_to(req)?;
        if let Some(content_type) = self.1 {
            response.set_header(content_type);
        }

        return Ok(response);
    }
}

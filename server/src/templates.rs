use crate::assets::Assets;
use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TERA: Tera = {
        let mut tera = Tera::default();

        let mut templates: Vec<(String, String)> = Vec::new();
        for file in Assets::iter() {
            if file.starts_with("templates/") && file.ends_with(".html.tera") {
                let filename = file[10..file.len() - 10].to_owned();
                let bytes = Assets::get(&file).expect(&format!("Cannot get template {} from assets.", file));
                let content =
                    std::str::from_utf8(bytes.as_ref()).expect(&format!("Cannot load template {} from assets.", file));
                templates.push((filename, content.to_owned()));
            }
        }

        let template_refs = templates.iter().map(|(a, b)| (a as &str, b as &str)).collect();
        tera.add_raw_templates(template_refs).expect("Cannot add tera templates.");

        tera
    };
}

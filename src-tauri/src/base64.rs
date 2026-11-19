use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub fn encode(input: &[u8]) -> String {
    STANDARD.encode(input)
}

// ! Only raster image types are embedded: an SVG served by Steam would run scripts in the page.
pub fn data_url(bytes: &[u8], mime: &str) -> Option<String> {
    let mime = match mime
        .split(';')
        .next()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("image/png") => "image/png",
        Some("image/jpeg") | Some("image/jpg") => "image/jpeg",
        Some("image/webp") => "image/webp",
        Some("image/gif") => "image/gif",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", encode(bytes)))
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(ALPHABET[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(ALPHABET[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

/// ! Only raster image types are embedded: an SVG served by Steam would run
/// its scripts inside the page.
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

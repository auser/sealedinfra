#[inline]
pub fn image_or_from_language(image: Option<String>, language: &str) -> String {
    match image {
        Some(image) => image,
        None => match language {
            "python" => "python:3.12".to_string(),
            "node" => "node:20".to_string(),
            "rust" => "rust".to_string(),
            _ => "alpine:latest".to_string(),
        },
    }
}

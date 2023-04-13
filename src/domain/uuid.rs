use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Uuid(String);

impl Uuid {
    pub fn parse(s: String) -> Result<Uuid, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() == 36;
        // TODO: Replace check with regular expression
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '%', '$', '#', '@', '!', '~', '`', '*', ',', ':'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

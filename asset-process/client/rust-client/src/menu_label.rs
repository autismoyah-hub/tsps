//! Java String.trim().isEmpty() removes only characters through U+0020.
pub fn present(value: &str) -> bool {
    value.chars().any(|ch| ch > '\u{20}')
}

//! Username canonicalization from aay, zh and wp.af. Length is measured in UTF-16 units.
pub fn normalize(name: &str, max_length: usize) -> Option<String> {
    normalize_utf16(&name.encode_utf16().collect::<Vec<_>>(), max_length)
}
pub fn normalize_utf16(name: &[u16], max_length: usize) -> Option<String> {
    fn separator(c: u16) -> bool {
        matches!(c, 32 | 160 | 95 | 45)
    }
    let start = name.iter().position(|&c| !separator(c))?;
    let end = name.iter().rposition(|&c| !separator(c))? + 1;
    if end - start > max_length {
        return None;
    }
    let mut out = String::new();
    for &c in &name[start..end] {
        let mapped = match c {
            48..=57 | 97..=122 | 35 | 91 | 93 => Some(c as u8 as char),
            65..=90 => Some((c + 32) as u8 as char),
            32 | 160 | 95 | 45 => Some('_'),
            192..=196 | 224..=228 => Some('a'),
            199 | 231 => Some('c'),
            200..=203 | 232..=235 => Some('e'),
            205..=207 | 237..=239 => Some('i'),
            209 | 241 => Some('n'),
            210..=214 | 242..=246 => Some('o'),
            217..=220 | 249..=252 => Some('u'),
            223 => Some('b'),
            255 | 376 => Some('y'),
            _ => None,
        };
        if let Some(c) = mapped {
            out.push(c);
        }
    }
    (!out.is_empty()).then_some(out)
}

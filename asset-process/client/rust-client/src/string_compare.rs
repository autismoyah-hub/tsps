//! Original GameWorldMapData.ag comparison, including ligatures and language tie-breaks.
//! Sparse Character mappings are exported from the reference JVM by StringCompareVectors.
pub(crate) fn casing(c: u16) -> (u16, u16, bool) {
    let bytes = include_bytes!("data/java-char-case.bin");
    let mut lo = 0;
    let mut hi = bytes.len() / 7;
    while lo < hi {
        let mid = (lo + hi) / 2;
        let p = mid * 7;
        let key = u16::from_be_bytes([bytes[p], bytes[p + 1]]);
        if key < c {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let p = lo * 7;
    if p < bytes.len() && u16::from_be_bytes([bytes[p], bytes[p + 1]]) == c {
        (
            u16::from_be_bytes([bytes[p + 2], bytes[p + 3]]),
            u16::from_be_bytes([bytes[p + 4], bytes[p + 5]]),
            bytes[p + 6] != 0,
        )
    } else {
        (c, c, false)
    }
}
fn standard(c: u16, language: u8) -> u16 {
    match c {
        192..=198 => 65,
        199 => 67,
        200..=203 => 69,
        204..=207 => 73,
        209 if language != 5 => 78,
        210..=214 => 79,
        217..=220 => 85,
        221 => 89,
        223 => 115,
        224..=230 => 97,
        231 => 99,
        232..=235 => 101,
        236..=239 => 105,
        241 if language != 5 => 110,
        242..=246 => 111,
        249..=252 => 117,
        253 | 255 => 121,
        338 => 79,
        339 => 111,
        376 => 89,
        _ => c,
    }
}
fn extra(c: u16) -> Option<u16> {
    match c {
        198 | 338 => Some(69),
        230 | 339 => Some(101),
        223 => Some(115),
        _ => None,
    }
}
fn weight(c: u16, language: u8) -> i32 {
    let (_, lower, upper) = casing(c);
    let c = if upper { lower } else { c };
    if c == 241 && language == 5 {
        1762
    } else {
        (i32::from(c) << 4) + i32::from(upper)
    }
}
fn difference(a: u16, b: u16, language: u8) -> i32 {
    if a == b || casing(a).0 == casing(b).0 {
        return 0;
    }
    let (a, b) = (casing(a).1, casing(b).1);
    if a == b {
        0
    } else {
        weight(a, language) - weight(b, language)
    }
}
/// Accept UTF-16 directly so surrogate code units retain Java char behavior.
pub fn compare(a: &[u16], b: &[u16], language: u8) -> i32 {
    let (mut i, mut j, mut left, mut right) = (0, 0, None, None);
    loop {
        let enda = i >= a.len() && left.is_none();
        let endb = j >= b.len() && right.is_none();
        if enda && endb {
            break;
        }
        if enda {
            return -1;
        }
        if endb {
            return 1;
        }
        let ac = left.take().unwrap_or_else(|| {
            let c = a[i];
            i += 1;
            c
        });
        let bc = right.take().unwrap_or_else(|| {
            let c = b[j];
            j += 1;
            c
        });
        left = extra(ac);
        right = extra(bc);
        let d = difference(standard(ac, language), standard(bc, language), language);
        if d != 0 {
            return d;
        }
    }
    let len = a.len().min(b.len());
    for n in 0..len {
        let (i, j) = if language == 2 {
            (a.len() - 1 - n, b.len() - 1 - n)
        } else {
            (n, n)
        };
        let d = difference(a[i], b[j], language);
        if d != 0 {
            return d;
        }
    }
    let d = (a.len() as i32).wrapping_sub(b.len() as i32);
    if d != 0 {
        return d;
    }
    for n in 0..len {
        if a[n] != b[n] {
            return weight(a[n], language) - weight(b[n], language);
        }
    }
    0
}
/// Java equalsIgnoreCase for wire names (CP1252, hence BMP only).
pub fn equals_ignore_case_bmp(a: &str, b: &str) -> bool {
    let a = a.encode_utf16().collect::<Vec<_>>();
    let b = b.encode_utf16().collect::<Vec<_>>();
    a.len() == b.len()
        && a.iter().zip(&b).all(|(&a, &b)| {
            if a == b {
                return true;
            }
            let a = casing(a).0;
            let b = casing(b).0;
            a == b || casing(a).1 == casing(b).1
        })
}

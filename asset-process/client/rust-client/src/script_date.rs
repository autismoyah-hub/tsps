//! Opcode4104's hybrid Gregorian calendar and fixed English month labels.
use crate::{Error, Result};
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MILLIS_PER_DAY: i64 = 86_400_000;

pub fn format(days: i32, offset_ms: i32, buddhist: bool) -> String {
    let day = 11_745 + days as i64 + (offset_ms as i64).div_euclid(MILLIS_PER_DAY);
    let [year, month, date] = if day >= -141_427 {
        gregorian(day)
    } else {
        julian(day)
    };
    // Calendar.YEAR is the year within its era; the opcode never prints ERA.
    let year = (if year > 0 { year } else { 1 - year }) + if buddhist { 543 } else { 0 };
    format!("{date}-{}-{year}", MONTHS[month as usize - 1])
}
fn gregorian(day: i64) -> [i64; 3] {
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let date = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    [year + i64::from(month <= 2), month, date]
}
fn julian(day: i64) -> [i64; 3] {
    let c = day + 2_440_588 + 32_082;
    let d = (4 * c + 3).div_euclid(1461);
    let e = c - (1461 * d).div_euclid(4);
    let m = (5 * e + 2).div_euclid(153);
    [
        d - 4800 + m.div_euclid(10),
        m + 3 - 12 * m.div_euclid(10),
        e - (153 * m + 2).div_euclid(5) + 1,
    ]
}

#[cfg(windows)]
mod local {
    use super::*;
    #[repr(C)]
    #[derive(Default)]
    struct SystemTime {
        year: u16,
        month: u16,
        weekday: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        millis: u16,
    }
    #[repr(C)]
    struct NativeZone {
        bias: i32,
        standard_name: [u16; 32],
        standard_date: SystemTime,
        standard_bias: i32,
        daylight_name: [u16; 32],
        daylight_date: SystemTime,
        daylight_bias: i32,
        key: [u16; 128],
        disabled: u8,
    }
    struct Context {
        zone: NativeZone,
        buddhist: bool,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDynamicTimeZoneInformation(zone: *mut NativeZone) -> u32;
        fn SystemTimeToTzSpecificLocalTimeEx(
            zone: *const NativeZone,
            utc: *const SystemTime,
            local: *mut SystemTime,
        ) -> i32;
        fn GetUserDefaultLocaleName(name: *mut u16, length: i32) -> i32;
    }
    fn context() -> Result<&'static Context> {
        static CONTEXT: std::sync::OnceLock<Option<Context>> = std::sync::OnceLock::new();
        CONTEXT
            .get_or_init(|| {
                // The FFI structure contains only integer fields and arrays.
                let mut zone: NativeZone = unsafe { std::mem::zeroed() };
                let mut locale = [0u16; 85];
                if unsafe { GetDynamicTimeZoneInformation(&mut zone) } == u32::MAX {
                    return None;
                }
                let len =
                    unsafe { GetUserDefaultLocaleName(locale.as_mut_ptr(), locale.len() as i32) };
                if len <= 0 {
                    return None;
                }
                let locale = String::from_utf16_lossy(&locale[..len as usize - 1]);
                Some(Context {
                    zone,
                    buddhist: locale.eq_ignore_ascii_case("th-TH"),
                })
            })
            .as_ref()
            .ok_or(Error::Unsupported("system script calendar unavailable"))
    }
    pub fn identity() -> Result<(String, bool)> {
        let context = context()?;
        let end = context
            .zone
            .key
            .iter()
            .position(|v| *v == 0)
            .unwrap_or(context.zone.key.len());
        Ok((
            String::from_utf16_lossy(&context.zone.key[..end]),
            context.buddhist,
        ))
    }
    pub fn format_local(days: i32) -> Result<String> {
        let context = context()?;
        let epoch_day = 11_745 + days as i64;
        let [year, month, date] = gregorian(epoch_day);
        let offset = if year < 1900 {
            // JDK ZoneInfo uses its latest raw offset before its transition table.
            let standard = if context.zone.standard_date.month == 0 {
                0
            } else {
                context.zone.standard_bias as i64
            };
            -(context.zone.bias as i64 + standard) * 60_000
        } else {
            // SYSTEMTIME is bounded. Repeat the Gregorian 400-year cycle under
            // Windows' last known rules for dates beyond its supported range.
            // Windows and Java historical/future zone databases are not identical.
            let mapped_year = if year <= 30_826 {
                year
            } else {
                2400 + year.rem_euclid(400)
            };
            let utc = SystemTime {
                year: mapped_year as u16,
                month: month as u16,
                day: date as u16,
                ..Default::default()
            };
            let mut local = SystemTime::default();
            if unsafe { SystemTimeToTzSpecificLocalTimeEx(&context.zone, &utc, &mut local) } == 0 {
                return Err(Error::Io(format!(
                    "script calendar timezone: {}",
                    std::io::Error::last_os_error()
                )));
            }
            let local_day = crate::date_of_birth::epoch_day([
                local.year as i32,
                local.month as i32,
                local.day as i32,
            ]);
            let utc_day =
                crate::date_of_birth::epoch_day([mapped_year as i32, month as i32, date as i32]);
            (local_day - utc_day) * MILLIS_PER_DAY
                + ((local.hour as i64 * 60 + local.minute as i64) * 60 + local.second as i64) * 1000
                + local.millis as i64
        };
        let offset =
            i32::try_from(offset).map_err(|_| Error::Invalid("script calendar timezone offset"))?;
        Ok(format(days, offset, context.buddhist))
    }
}
#[cfg(windows)]
pub use local::{format_local, identity as local_identity};
#[cfg(not(windows))]
pub fn format_local(_: i32) -> Result<String> {
    Err(Error::Unsupported("native script calendar platform"))
}
#[cfg(not(windows))]
pub fn local_identity() -> Result<(String, bool)> {
    Err(Error::Unsupported("native script calendar platform"))
}

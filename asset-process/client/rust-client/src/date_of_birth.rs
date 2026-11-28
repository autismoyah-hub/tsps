//! Original my.ac/ar.av DOB validation and epoch-day encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Missing,
    Date,
}
impl Invalid {
    pub fn lines(self) -> [&'static str; 3] {
        match self {
            Self::Missing => [
                "Date not valid.",
                "Please ensure all characters are populated.",
                "",
            ],
            Self::Date => [
                "Date not valid.",
                "Please ensure date follows the format",
                "DD/MM/YYYY and is after 01/01/1900",
            ],
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Today {
    pub date: [i32; 3],
    pub millis_since_midnight: i32,
}
/// Gregorian epoch day; permitting day overflow reproduces Calendar's leap-day year change.
pub fn epoch_day([mut year, month, day]: [i32; 3]) -> i64 {
    year -= i32::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let adjusted = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * adjusted + 2) / 5 + day - 1;
    (era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468) as i64
}
pub fn parse(digits: [Option<u8>; 8]) -> Result<[i32; 3], Invalid> {
    let mut n = [0i32; 8];
    for (i, digit) in digits.iter().enumerate() {
        n[i] = digit.ok_or(Invalid::Missing)? as i32;
        if n[i] > 9 {
            return Err(Invalid::Date);
        }
    }
    let date = [
        n[4] * 1000 + n[5] * 100 + n[6] * 10 + n[7],
        n[2] * 10 + n[3],
        n[0] * 10 + n[1],
    ];
    let [year, month, day] = date;
    if year == 0 || !(1..=12).contains(&month) {
        return Err(Invalid::Date);
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last = if month == 2 {
        if leap {
            29
        } else {
            28
        }
    } else if [4, 6, 9, 11].contains(&month) {
        30
    } else {
        31
    };
    if !(1..=last).contains(&day) {
        return Err(Invalid::Date);
    }
    Ok(date)
}
/// The UTC milliseconds must represent noon on the selected date in the local time zone.
pub fn encode(
    digits: [Option<u8>; 8],
    today: Today,
    utc_noon: impl FnOnce([i32; 3]) -> Option<i64>,
) -> Result<u32, Invalid> {
    let date = parse(digits)?;
    let noon = utc_noon(date).ok_or(Invalid::Date)?;
    if date < [1900, 1, 1] || (date == [1900, 1, 1] && today.millis_since_midnight >= 43_200_000) {
        return Err(Invalid::Date);
    }
    let [year, month, day] = today.date;
    if epoch_day(date) > epoch_day([year - 13, month, day]) {
        return Ok(8_388_607);
    }
    Ok((noon / 86_400_000 - 11_745) as i32 as u32)
}

#[cfg(windows)]
mod local {
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
    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(time: *mut SystemTime);
        fn TzSpecificLocalTimeToSystemTime(
            zone: *const std::ffi::c_void,
            local: *const SystemTime,
            utc: *mut SystemTime,
        ) -> i32;
    }
    pub fn today() -> super::Today {
        let mut now = SystemTime::default();
        unsafe {
            GetLocalTime(&mut now);
        }
        super::Today {
            date: [now.year as i32, now.month as i32, now.day as i32],
            millis_since_midnight: ((now.hour as i32 * 60 + now.minute as i32) * 60
                + now.second as i32)
                * 1000
                + now.millis as i32,
        }
    }
    pub fn noon(date: [i32; 3]) -> Option<i64> {
        let [year, month, day] = date;
        let local = SystemTime {
            year: year.try_into().ok()?,
            month: month.try_into().ok()?,
            day: day.try_into().ok()?,
            hour: 12,
            ..Default::default()
        };
        let mut utc = SystemTime::default();
        if unsafe { TzSpecificLocalTimeToSystemTime(std::ptr::null(), &local, &mut utc) } == 0 {
            return None;
        }
        Some(
            super::epoch_day([utc.year as i32, utc.month as i32, utc.day as i32]) * 86_400_000
                + ((utc.hour as i64 * 60 + utc.minute as i64) * 60 + utc.second as i64) * 1000
                + utc.millis as i64,
        )
    }
}
#[cfg(windows)]
pub fn local_today() -> Option<Today> {
    Some(local::today())
}
#[cfg(windows)]
pub fn local_noon(date: [i32; 3]) -> Option<i64> {
    local::noon(date)
}
#[cfg(not(windows))]
pub fn local_today() -> Option<Today> {
    None
}
#[cfg(not(windows))]
pub fn local_noon(_: [i32; 3]) -> Option<i64> {
    None
}

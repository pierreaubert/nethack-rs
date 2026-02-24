//! Time and date utility functions translated from NetHack hacklib.c
//!
//! Provides functions for time formatting, date checking (night, Friday 13th, etc.)
//! and time comparison operations.

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp (seconds since epoch)
pub fn getnow() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get current year (e.g., 2026)
pub fn getyear() -> i32 {
    Local::now().year()
}

/// Get local time structure equivalent to C's `struct tm`
#[derive(Debug, Clone, Copy)]
pub struct LocalTime {
    pub year: i32,
    pub month: u32,  // 1-12
    pub day: u32,    // 1-31
    pub hour: u32,   // 0-23
    pub minute: u32, // 0-59
    pub second: u32, // 0-59
    pub wday: u32,   // 0-6 (0=Sunday in C, but we use 0-6 same as chrono)
}

/// Get current local time as LocalTime struct
pub fn getlt() -> LocalTime {
    let now = Local::now();
    LocalTime {
        year: now.year(),
        month: now.month(),
        day: now.day(),
        hour: now.hour(),
        minute: now.minute(),
        second: now.second(),
        wday: now.weekday().number_from_sunday() - 1, // Convert to 0-6 Sunday-based
    }
}

/// Check if current time is night (before 6 AM or after 9 PM)
pub fn night() -> bool {
    let now = Local::now();
    let hour = now.hour();
    hour < 6 || hour >= 21
}

/// Check if current time is midnight (hour == 0)
pub fn midnight() -> bool {
    Local::now().hour() == 0
}

/// Moon phase constants
pub const NEW_MOON: i32 = 0;
pub const FULL_MOON: i32 = 4;

/// Calculate the phase of the moon (0-7, with 0: new, 4: full)
/// Ported from C hacklib.c:1106
pub fn phase_of_the_moon() -> i32 {
    let lt = getlt();
    // Day of year (0-based, like C's tm_yday)
    let diy = {
        let date = chrono::NaiveDate::from_ymd_opt(lt.year, lt.month, lt.day).unwrap();
        date.ordinal0() as i32
    };
    // Golden number (C: (tm_year % 19) + 1; tm_year = year - 1900)
    let goldn = ((lt.year - 1900) % 19) + 1;
    let mut epact = (11 * goldn + 18) % 30;
    if (epact == 25 && goldn > 11) || epact == 24 {
        epact += 1;
    }
    ((((diy + epact) * 6) + 11) % 177) / 22 & 7
}

/// Check if today is Friday the 13th
pub fn friday_13th() -> bool {
    let now = Local::now();
    now.day() == 13 && now.weekday().number_from_sunday() == 6 // 6 = Friday
}

/// Format time as HHMMSS integer (e.g., 123045 for 12:30:45)
pub fn hhmmss() -> u32 {
    let now = Local::now();
    now.hour() as u32 * 10000 + now.minute() as u32 * 100 + now.second() as u32
}

/// Format time as HHMMSS integer from Unix timestamp
pub fn hhmmss_from_timestamp(timestamp: u64) -> u32 {
    let dt = DateTime::<Local>::from(UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    dt.hour() as u32 * 10000 + dt.minute() as u32 * 100 + dt.second() as u32
}

/// Format date as YYYYMMDD integer (e.g., 20260119 for 2026-01-19)
pub fn yyyymmdd() -> u32 {
    let now = Local::now();
    now.year() as u32 * 10000 + now.month() * 100 + now.day()
}

/// Format date as YYYYMMDD integer from Unix timestamp
pub fn yyyymmdd_from_timestamp(timestamp: u64) -> u32 {
    let dt = DateTime::<Local>::from(UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    dt.year() as u32 * 10000 + dt.month() * 100 + dt.day()
}

/// Format date as YYMMDD integer (e.g., 260119 for 2026-01-19)
pub fn yymmdd() -> u32 {
    let now = Local::now();
    (now.year() % 100) as u32 * 10000 + now.month() * 100 + now.day()
}

/// Format date as YYMMDD integer from Unix timestamp
pub fn yymmdd_from_timestamp(timestamp: u64) -> u32 {
    let dt = DateTime::<Local>::from(UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    ((dt.year() % 100) as u32) * 10000 + dt.month() * 100 + dt.day()
}

/// Format datetime as YYYYMMDDhhmmss string (e.g., "20260119123045")
pub fn yyyymmddhhmmss() -> String {
    let now = Local::now();
    format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Format datetime as YYYYMMDDhhmmss string from Unix timestamp
pub fn yyyymmddhhmmss_from_timestamp(timestamp: u64) -> String {
    let dt = DateTime::<Local>::from(UNIX_EPOCH + std::time::Duration::from_secs(timestamp));
    format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

/// Parse a YYYYMMDDhhmmss string and return Unix timestamp
/// Returns None if the format is invalid
pub fn time_from_yyyymmddhhmmss(s: &str) -> Option<u64> {
    if s.len() != 14 {
        return None;
    }

    let year: i32 = s[0..4].parse().ok()?;
    let month: u32 = s[4..6].parse().ok()?;
    let day: u32 = s[6..8].parse().ok()?;
    let hour: u32 = s[8..10].parse().ok()?;
    let minute: u32 = s[10..12].parse().ok()?;
    let second: u32 = s[12..14].parse().ok()?;

    // Validate ranges
    if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    let naive = NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(year, month, day)?,
        chrono::NaiveTime::from_hms_opt(hour, minute, second)?,
    );

    let dt = Local.from_local_datetime(&naive).single()?;
    Some(dt.timestamp() as u64)
}

/// Compare two Unix timestamps
/// Returns: -1 if a < b, 0 if a == b, 1 if a > b
pub fn comp_times(a: u64, b: u64) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Find end of string (returns position of null terminator or length)
/// This is a utility function for string operations
pub fn eos(s: &str) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_night() {
        // We can't reliably test night() without mocking system time
        // But we can verify it returns a bool
        let _result = night();
    }

    #[test]
    fn test_friday_13th() {
        // Can't reliably test without mocking, but verify it returns a bool
        let _result = friday_13th();
    }

    #[test]
    fn test_yyyymmdd_format() {
        let date = yyyymmdd();
        // Should be a valid date in YYYYMMDD format
        assert!(date >= 10000101 && date <= 99991231);
    }

    #[test]
    fn test_yhmmss_format() {
        let time = hhmmss();
        // Should be a valid time in HHMMSS format
        assert!(time <= 235959);
    }

    #[test]
    fn test_time_parsing() {
        let formatted = yyyymmddhhmmss();
        assert_eq!(formatted.len(), 14);

        if let Some(_timestamp) = time_from_yyyymmddhhmmss(&formatted) {
            // Successfully parsed the formatted string
        } else {
            panic!("Failed to parse valid timestamp string");
        }
    }

    #[test]
    fn test_time_parsing_invalid() {
        assert!(time_from_yyyymmddhhmmss("").is_none());
        assert!(time_from_yyyymmddhhmmss("12345").is_none());
        assert!(time_from_yyyymmddhhmmss("20261301000000").is_none()); // Invalid month
        assert!(time_from_yyyymmddhhmmss("20260132000000").is_none()); // Invalid day
    }

    #[test]
    fn test_comp_times() {
        assert_eq!(comp_times(100, 200), -1);
        assert_eq!(comp_times(200, 100), 1);
        assert_eq!(comp_times(100, 100), 0);
    }

    #[test]
    fn test_eos() {
        assert_eq!(eos("hello"), 5);
        assert_eq!(eos(""), 0);
        assert_eq!(eos("test string"), 11);
    }
}

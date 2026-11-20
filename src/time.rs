//! UTC instants and calendar dates, with strict input and whole-second output.
//!
//! Input is RFC 3339 (any offset, normalized to UTC) or `YYYY-MM-DD`, nothing looser: `jiff`
//! alone would also take RFC 9557 annotations, ISO 8601 basic format, missing seconds, and a
//! date-time where a date is expected. Output is RFC 3339 in UTC to the second with a `Z` suffix;
//! a fractional second is dropped, never rounded.

use jiff::{SignedDuration, Timestamp, civil::Date, tz::Offset};

/// `YYYY-MM-DD` with ASCII digits.
fn date_shape(b: &[u8]) -> bool {
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

/// RFC 3339 `date-time`: `YYYY-MM-DD`, `T` (or `t` or a space), `HH:MM:SS`, an optional
/// fraction, then `Z` (or `z`) or `±HH:MM`.
fn timestamp_shape(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() < 20 || !date_shape(&b[..10]) || !matches!(b[10], b'T' | b't' | b' ') {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if !(digits(11..13) && b[13] == b':' && digits(14..16) && b[16] == b':' && digits(17..19)) {
        return false;
    }
    let mut i = 19;
    if b[i] == b'.' {
        let start = i + 1;
        i = start;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    match &b[i..] {
        [b'Z' | b'z'] => true,
        [b'+' | b'-', h1, h2, b':', m1, m2] => [h1, h2, m1, m2].iter().all(|c| c.is_ascii_digit()),
        _ => false,
    }
}

/// An RFC 3339 timestamp, or None.
pub fn parse_timestamp(s: &str) -> Option<Timestamp> {
    timestamp_shape(s).then(|| s.parse().ok()).flatten()
}

/// A `YYYY-MM-DD` date, or None.
pub fn parse_date(s: &str) -> Option<Date> {
    date_shape(s.as_bytes()).then(|| s.parse().ok()).flatten()
}

/// `t` in UTC to the second, e.g. `2026-08-01T12:00:00Z`.
pub fn format_seconds(t: Timestamp) -> String {
    let dt = Offset::UTC.to_datetime(t);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

/// The UTC calendar date of `t`, as `YYYY-MM-DD`.
pub fn utc_date(t: Timestamp) -> String {
    let d = Offset::UTC.to_datetime(t).date();
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// `d` at `hour:minute:second` UTC, if representable (timestamps end at
/// 9999-12-30T22:00:00Z).
pub fn at_utc(d: Date, hour: i8, minute: i8, second: i8) -> Option<Timestamp> {
    Offset::UTC.to_timestamp(d.at(hour, minute, second, 0)).ok()
}

/// `days` days of 24 hours after `t`, if representable.
pub fn add_days(t: Timestamp, days: u32) -> Option<Timestamp> {
    t.checked_add(SignedDuration::from_secs(i64::from(days) * 86_400))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_accept_rfc3339_only() {
        for good in [
            "2026-08-01T12:00:00Z",
            "2026-08-01t12:00:00z",
            "2026-08-01 12:00:00Z",
            "2026-08-01T14:00:00+02:00",
            "2026-08-01T12:00:00.5-00:00",
        ] {
            assert!(parse_timestamp(good).is_some(), "{good}");
        }
        for bad in [
            "",
            "2026-08-01",
            "2026-08-01T12:00:00",
            "2026-08-01T12:00Z",
            "2026-08-01T12:00:00Z[UTC]",
            "20260801T120000Z",
            "2026-08-01T12:00:00.Z",
            "2026-08-01T12:00:00+0200",
            "2026-02-30T12:00:00Z",
        ] {
            assert!(parse_timestamp(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn dates_accept_yyyy_mm_dd_only() {
        assert!(parse_date("2026-09-30").is_some());
        for bad in [
            "",
            "2026-9-30",
            " 2026-09-30",
            "2026-02-30",
            "2026-09-30T00:00:00",
        ] {
            assert!(parse_date(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn formatting_and_arithmetic() {
        let t = parse_timestamp("2026-08-01T14:00:00.75+02:00").unwrap();
        assert_eq!(format_seconds(t), "2026-08-01T12:00:00Z");
        assert_eq!(
            utc_date(parse_timestamp("2026-08-01T23:30:00-02:00").unwrap()),
            "2026-08-02"
        );
        assert_eq!(
            format_seconds(add_days(t, 30).unwrap()),
            "2026-08-31T12:00:00Z"
        );
        let d = parse_date("2026-09-30").unwrap();
        assert_eq!(
            format_seconds(at_utc(d, 23, 59, 59).unwrap()),
            "2026-09-30T23:59:59Z"
        );
        assert!(at_utc(parse_date("9999-12-31").unwrap(), 0, 0, 0).is_none());
    }
}

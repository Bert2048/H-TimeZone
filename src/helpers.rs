/// All IANA timezone names from the chrono_tz database.
pub fn all_tz_names() -> Vec<&'static str> {
    chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name()).collect()
}

/// Extract the city display name from an IANA timezone string.
/// `"America/New_York"` → `"New York"`, `"Asia/Ho_Chi_Minh"` → `"Ho Chi Minh"`.
pub fn city_name(iana: &str) -> String {
    iana.rsplit('/').next().unwrap_or(iana).replace('_', " ")
}

/// Format a UTC offset in seconds as `"+HH:MM"` or `"-HH:MM"`.
pub fn fmt_offset(secs: i32) -> String {
    let sign = if secs >= 0 { '+' } else { '-' };
    let abs = secs.unsigned_abs();
    format!("{}{:02}:{:02}", sign, abs / 3600, (abs % 3600) / 60)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // city_name: last path segment, underscores → spaces
    #[test]
    fn city_name_simple() {
        assert_eq!(city_name("Asia/Shanghai"), "Shanghai");
    }

    #[test]
    fn city_name_underscore() {
        assert_eq!(city_name("America/New_York"), "New York");
    }

    #[test]
    fn city_name_multi_word() {
        assert_eq!(city_name("Asia/Ho_Chi_Minh"), "Ho Chi Minh");
    }

    #[test]
    fn city_name_no_slash() {
        assert_eq!(city_name("UTC"), "UTC");
    }

    // fmt_offset: +HH:MM / -HH:MM formatting
    #[test]
    fn fmt_offset_positive() {
        assert_eq!(fmt_offset(28_800), "+08:00"); // UTC+8
    }

    #[test]
    fn fmt_offset_negative() {
        assert_eq!(fmt_offset(-18_000), "-05:00"); // UTC-5
    }

    #[test]
    fn fmt_offset_zero() {
        assert_eq!(fmt_offset(0), "+00:00");
    }

    #[test]
    fn fmt_offset_half_hour() {
        assert_eq!(fmt_offset(19_800), "+05:30"); // UTC+5:30 India
    }

    // all_tz_names: all default timezones must be valid IANA names
    #[test]
    fn all_tz_names_contains_defaults() {
        let names = all_tz_names();
        for tz in &[
            "Asia/Shanghai",
            "America/New_York",
            "Europe/London",
            "America/Edmonton", // Calgary uses America/Edmonton in the IANA DB
            "America/Los_Angeles",
            "Asia/Tokyo",
        ] {
            assert!(names.contains(tz), "missing default timezone: {tz}");
        }
    }

    // Regression: America/Calgary is not a valid IANA name — must use America/Edmonton.
    // Found by /qa on 2026-03-21. If this test fails, chrono_tz added Calgary;
    // update config.rs default_clocks() to use America/Calgary directly.
    #[test]
    fn calgary_not_in_iana_database() {
        assert!(
            !all_tz_names().contains(&"America/Calgary"),
            "America/Calgary was added to chrono_tz — update config default to use it"
        );
    }
}

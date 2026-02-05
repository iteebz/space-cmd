use chrono::{DateTime, NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_elapsed_time(iso_timestamp: &str) -> String {
    let created = parse_iso_timestamp(iso_timestamp);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let elapsed = now.saturating_sub(created);

    if elapsed < 60 {
        format!("{}s", elapsed)
    } else if elapsed < 3600 {
        format!("{}m{}s", elapsed / 60, elapsed % 60)
    } else {
        format!("{}h{}m", elapsed / 3600, (elapsed % 3600) / 60)
    }
}

fn parse_iso_timestamp(iso_str: &str) -> u64 {
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso_str) {
        return dt.timestamp().max(0) as u64;
    }

    if let Ok(dt) = NaiveDateTime::parse_from_str(iso_str, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)
            .timestamp()
            .max(0) as u64;
    }

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_recent_timestamp() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent = now - 30;
        let year = 1970 + (recent / 31536000) as i32;
        let formatted = format_elapsed_time(&format!("{}-01-01T00:00:{}Z", year, recent % 60));
        assert!(!formatted.is_empty());
    }

    #[test]
    fn format_elapsed_under_minute() {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ts = chrono::DateTime::from_timestamp(now_secs as i64 - 30, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let formatted = format_elapsed_time(&ts);
        assert!(formatted.ends_with('s'));
        assert!(!formatted.contains('m'));
    }

    #[test]
    fn format_elapsed_minutes() {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ts = chrono::DateTime::from_timestamp(now_secs as i64 - 90, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let formatted = format_elapsed_time(&ts);
        assert!(formatted.contains('m'));
    }

    #[test]
    fn format_elapsed_hours() {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ts = chrono::DateTime::from_timestamp(now_secs as i64 - 7200, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        let formatted = format_elapsed_time(&ts);
        assert!(formatted.contains('h'));
    }

    #[test]
    fn parse_rfc3339_with_offset() {
        let zulu = parse_iso_timestamp("2025-01-01T10:00:00Z");
        let offset = parse_iso_timestamp("2025-01-01T20:00:00+10:00");
        assert_eq!(zulu, offset);
    }
}

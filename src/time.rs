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
    let iso_str = iso_str.replace('T', " ").replace('Z', "");
    let parts: Vec<&str> = iso_str.split(' ').collect();

    if parts.len() < 2 {
        return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() < 3 || time_parts.len() < 3 {
        return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    let year = date_parts[0].parse::<i32>().unwrap_or(1970);
    let month = date_parts[1].parse::<u32>().unwrap_or(1);
    let day = date_parts[2].parse::<u32>().unwrap_or(1);
    let hour = time_parts[0].parse::<u32>().unwrap_or(0);
    let minute = time_parts[1].parse::<u32>().unwrap_or(0);
    let second = time_parts[2]
        .split('.')
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);

    let days_since_epoch =
        (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = days_since_epoch;

    for m in 1..month.min(13) as usize {
        days += days_in_month[m - 1];
        if m == 2 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            days += 1;
        }
    }

    days += day as i32 - 1;

    (days as u64 * 86400) + (hour as u64 * 3600) + (minute as u64 * 60) + second as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_seconds() {
        let ts = "2025-11-05T21:50:00Z";
        let formatted = format_elapsed_time(ts);
        assert!(formatted.ends_with('s'));
    }
}

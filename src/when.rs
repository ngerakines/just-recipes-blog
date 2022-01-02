use std::time::Duration;

const SECONDS_IN_MINUTE: u64 = 60;
const SECONDS_IN_HOUR: u64 = 3600;
const SECONDS_IN_DAY: u64 = 86400;
const SECONDS_IN_WEEK: u64 = 604800;

// NG: I'm sure there are better ways to do this.
pub fn duration_iso8601(duration: Duration) -> String {
    if duration.is_zero() {
        return String::from("P");
    }

    let mut duration_parts: Vec<String> = vec![String::from("P")];

    let mut seconds = duration.as_secs();
    if seconds >= SECONDS_IN_WEEK {
        let weeks = seconds / SECONDS_IN_WEEK;
        duration_parts.push(format!("{}W", weeks));
        seconds -= weeks * SECONDS_IN_WEEK;
    }
    if seconds >= SECONDS_IN_DAY {
        let days = seconds / SECONDS_IN_DAY;
        duration_parts.push(format!("{}D", days));
        seconds -= days * SECONDS_IN_DAY;
    }
    if duration_parts.len() > 1 && seconds > 0 {
        duration_parts.push(String::from("T"));
    }
    if seconds >= SECONDS_IN_HOUR {
        let hours = seconds / SECONDS_IN_HOUR;
        duration_parts.push(format!("{}H", hours));
        seconds -= hours * SECONDS_IN_HOUR;
    }
    if seconds >= SECONDS_IN_MINUTE {
        let minutes = seconds / SECONDS_IN_MINUTE;
        duration_parts.push(format!("{}M", minutes));
        seconds -= minutes * SECONDS_IN_MINUTE;
    }
    if seconds > 0 {
        duration_parts.push(format!("{}S", seconds));
    }

    duration_parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use humantime::parse_duration;

    #[test]
    fn fmt_duration_ok() {
        assert_eq!(duration_iso8601(Duration::new(300, 0)), "P5M");

        assert_eq!(
            duration_iso8601(parse_duration("2h 37min").unwrap()),
            "P2H37M"
        );
        assert_eq!(
            duration_iso8601(parse_duration("30 minutes").unwrap()),
            "P30M"
        );
        assert_eq!(
            duration_iso8601(parse_duration("70 minutes").unwrap()),
            "P1H10M"
        );
    }
}

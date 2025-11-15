use chrono::{Datelike, Duration, Utc, Weekday};
use chrono_tz::Europe::Moscow;

pub fn get_time_until_friday() -> Option<Duration> {
    let now = Utc::now().with_timezone(&Moscow);
    let current_weekday = now.weekday();

    let days_to_add = (Weekday::Fri.number_from_monday() + 7 - current_weekday.number_from_monday()) % 7;
    if days_to_add == 0 {
        return None;
    }

    let next_friday_date = now.date_naive() + Duration::days(days_to_add as i64);
    let next_friday_midnight = next_friday_date.and_hms_opt(0, 0, 0).unwrap();
    let duration = next_friday_midnight.signed_duration_since(now.naive_local());

    Some(duration)
}

pub fn format_time_delta(td: Duration) -> String {
    let days = td.num_days();
    let hours = td.num_hours() % 24;
    let minutes = td.num_minutes() % 60;
    format!("{days} дней, {hours} часов, {minutes} минут")
}

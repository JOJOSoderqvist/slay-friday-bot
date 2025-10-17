use chrono::{Datelike, Duration, Utc, Weekday};
use chrono_tz::Europe::Moscow;

pub fn get_time_until_friday() -> Duration {
    let now = Utc::now().with_timezone(&Moscow);
    let now_weekday = now.weekday();
    let days_until_friday = match now_weekday {
        Weekday::Fri => 7,
        _ => {
            let mut days = Weekday::Fri.num_days_from_monday() as i64 - now_weekday.num_days_from_monday() as i64;
            if days < 0 {
                days += 7
            }

            days
        }
    };

    let next_friday = now + Duration::days(days_until_friday);

    next_friday - now
}

pub fn format_timedelta(td: Duration) -> String {
    let total_seconds = td.num_seconds();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{days} дней, {hours} часов, {minutes} минут и {seconds} секунд")
}

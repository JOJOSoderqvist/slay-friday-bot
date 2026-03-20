use chrono::{Datelike, Duration, Utc, Weekday};
use chrono_tz::Europe::Moscow;
use reqwest::{Client, Proxy};
use std::fmt::Display;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, ReplyMarkup,
};

use crate::errors::ApiError::{self, ApiClientBuildError, ProxyURLBuildError};

const DEFAULT_REPLY_KEYBOARD_CHUNK_SIZE: usize = 3;
const DEFAULT_INLINE_KEYBOARD_CHUNK_SIZE: usize = 4;

pub fn get_time_until_friday() -> Option<Duration> {
    let now = Utc::now().with_timezone(&Moscow);
    let current_weekday = now.weekday();

    let days_to_add =
        (Weekday::Fri.number_from_monday() + 7 - current_weekday.number_from_monday()) % 7;
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

pub fn setup_inline_callback_keyboard<T: Display>(data: &[T]) -> Option<InlineKeyboardMarkup> {
    if data.is_empty() {
        return None;
    }

    let rows: Vec<Vec<InlineKeyboardButton>> = data
        .chunks(DEFAULT_INLINE_KEYBOARD_CHUNK_SIZE)
        .map(|chunk| {
            chunk
                .iter()
                .map(|elem| {
                    let text = elem.to_string();
                    let displayed_text = text.clone();
                    let callback_text = text.clone();

                    InlineKeyboardButton::callback(displayed_text, callback_text)
                })
                .collect()
        })
        .collect();

    Some(InlineKeyboardMarkup::new(rows))
}

pub fn reply_suggestions_keyboard<T: ToString>(data: &[T], cmd_prefix: &str) -> ReplyMarkup {
    let rows: Vec<Vec<KeyboardButton>> = data
        .chunks(DEFAULT_REPLY_KEYBOARD_CHUNK_SIZE)
        .map(|chunk| {
            chunk
                .iter()
                .map(|x| KeyboardButton::new(format!("{} {}", cmd_prefix, x.to_string())))
                .collect()
        })
        .collect();
    let mut keyboard = KeyboardMarkup::new(rows);
    keyboard.selective = true;
    keyboard.resize_keyboard = true;

    ReplyMarkup::Keyboard(keyboard)
}

pub fn get_client_with_proxy(proxy_url: &str) -> Result<Client, ApiError> {
    let proxy = Proxy::all(proxy_url).map_err(ProxyURLBuildError)?;

    Client::builder()
        .proxy(proxy)
        .build()
        .map_err(ApiClientBuildError)
}

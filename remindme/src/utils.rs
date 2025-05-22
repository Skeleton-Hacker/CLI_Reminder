// Common utilities for the application

use chrono::{DateTime, Local};

#[allow(dead_code)]
pub fn format_datetime(dt: &DateTime<Local>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

#[allow(dead_code)]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
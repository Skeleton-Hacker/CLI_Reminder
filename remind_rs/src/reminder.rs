use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecurrenceType {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Custom(String), // For cron-like expressions (optional for future)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub text: String,
    pub due_time: DateTime<Local>,
    pub recurrence: RecurrenceType,
    pub created_at: DateTime<Local>,
    pub last_notified: Option<DateTime<Local>>,
    pub completed: bool,
}

impl Reminder {
    pub fn new(text: String, due_time: DateTime<Local>, recurrence: RecurrenceType) -> Self {
        Reminder {
            id: Uuid::new_v4().to_string(),
            text,
            due_time,
            recurrence,
            created_at: Local::now(),
            last_notified: None,
            completed: false,
        }
    }

    pub fn is_due(&self) -> bool {
        let now = Local::now();
        self.due_time <= now && !self.completed && 
            // If already notified, check if it's a recurring reminder that should be notified again
            self.last_notified.map_or(true, |last| {
                match self.recurrence {
                    RecurrenceType::None => false, // Non-recurring, only notify once
                    // Only notify again if at least a day has passed since last notification
                    _ => (now - last).num_hours() >= 24
                }
            })
    }

    pub fn mark_notified(&mut self) {
        self.last_notified = Some(Local::now());
        
        // For recurring reminders, reschedule
        match self.recurrence {
            RecurrenceType::None => {
                self.completed = true;
            }
            RecurrenceType::Daily => {
                self.due_time = self.due_time + chrono::Duration::days(1);
            }
            RecurrenceType::Weekly => {
                self.due_time = self.due_time + chrono::Duration::weeks(1);
            }
            RecurrenceType::Monthly => {
                // This is a simplification; months have different lengths
                let new_month = self.due_time.month() % 12 + 1;
                let new_year = self.due_time.year() + if new_month == 1 { 1 } else { 0 };
                self.due_time = Local.with_ymd_and_hms(
                    new_year,
                    new_month,
                    self.due_time.day().min(days_in_month(new_month, new_year)),
                    self.due_time.hour(),
                    self.due_time.minute(),
                    self.due_time.second(),
                ).unwrap();
            }
            RecurrenceType::Yearly => {
                self.due_time = Local.with_ymd_and_hms(
                    self.due_time.year() + 1,
                    self.due_time.month(),
                    self.due_time.day(),
                    self.due_time.hour(),
                    self.due_time.minute(),
                    self.due_time.second(),
                ).unwrap();
            }
            RecurrenceType::Custom(_) => {
                // For future implementation with cron-like expressions
            }
        }
    }
}

impl fmt::Display for Reminder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (Due: {}) {}",
            self.id, // Show full UUID
            self.text,
            self.due_time.format("%Y-%m-%d %H:%M"),
            if self.completed { "[COMPLETED]" } else { "" }
        )
    }
}

// Helper function to get days in a month
fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month"),
    }
}
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};

use crate::reminder::RecurrenceType;

#[derive(Parser)]
#[command(name = "remindme")]
#[command(about = "A simple CLI reminder application", long_about = None)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    /// Start the TUI (Text User Interface) mode
    #[arg(short, long)]
    pub tui: bool,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new reminder
    Add {
        /// Text of the reminder
        #[arg(short, long)]
        text: String,

        /// Time of the reminder (HH:MM), date will default to today or tomorrow
        #[arg(short = 't', long)]
        time: String,

        /// Date of the reminder (YYYY-MM-DD), defaults to today/tomorrow based on time
        #[arg(short = 'd', long)]
        date: Option<String>,

        /// Recurrence pattern (none, daily, weekly, monthly, yearly)
        #[arg(short, long, default_value = "none")]
        recurrence: String,
    },

    /// List all reminders
    List,

    /// Delete a reminder by ID or index
    Delete {
        /// ID of the reminder to delete
        #[arg(short, long, group = "delete_selector")]
        id: Option<String>,
        
        /// Index of the reminder to delete (as shown in list)
        #[arg(short, long, group = "delete_selector")]
        index: Option<usize>,
    },
    
    /// Edit an existing reminder
    Edit {
        /// ID of the reminder to edit
        #[arg(short, long)]
        id: String,
        
        /// New text for the reminder
        #[arg(short, long)]
        text: Option<String>,
        
        /// New time for the reminder (YYYY-MM-DD HH:MM)
        #[arg(short = 't', long)]
        time: Option<String>,
        
        /// New recurrence pattern
        #[arg(short, long)]
        recurrence: Option<String>,
    },
    
    /// Check for due reminders and notify
    Notify {
        /// Send desktop notifications
        #[arg(short, long)]
        desktop: bool,
    },
    
    /// Export reminders as JSON
    Export,
    
    /// Show statistics about reminders
    Stats,
    
    /// Search for reminders
    Search {
        /// Search query
        #[arg(short, long)]
        query: String,
    },
    
    /// Show help information
    Help {
        /// Show help for a specific command
        #[arg(short, long)]
        command: Option<String>,
    },
}

pub fn parse_datetime(datetime_str: &str) -> Result<DateTime<Local>> {
    let naive_datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M")
        .context("Invalid date time format. Expected YYYY-MM-DD HH:MM")?;
    
    let local_datetime = Local.from_local_datetime(&naive_datetime)
        .single()
        .context("Failed to convert to local datetime")?;
    
    Ok(local_datetime)
}

pub fn parse_datetime_with_default_date(time_str: &str, date_option: Option<&str>) -> Result<DateTime<Local>> {
    // Get current date/time
    let now = Local::now();
    
    // Parse the time part
    let time_format = "%H:%M";
    let naive_time = chrono::NaiveTime::parse_from_str(time_str, time_format)
        .context("Invalid time format. Expected HH:MM")?;
    
    // If date is provided, use it
    if let Some(date_str) = date_option {
        let date_time_str = format!("{} {}", date_str, time_str);
        return parse_datetime(&date_time_str);
    }
    
    // Otherwise use today's date
    let today = now.date_naive();
    let naive_datetime = today.and_time(naive_time);
    
    // Convert to DateTime<Local>
    let mut local_datetime = Local.from_local_datetime(&naive_datetime)
        .single()
        .context("Failed to convert to local datetime")?;
    
    // If the time today has already passed, use tomorrow instead
    if local_datetime < now {
        local_datetime = local_datetime + chrono::Duration::days(1);
    }
    
    Ok(local_datetime)
}

pub fn parse_recurrence(recurrence_str: &str) -> Result<RecurrenceType> {
    match recurrence_str.to_lowercase().as_str() {
        "none" => Ok(RecurrenceType::None),
        "daily" => Ok(RecurrenceType::Daily),
        "weekly" => Ok(RecurrenceType::Weekly),
        "monthly" => Ok(RecurrenceType::Monthly),
        "yearly" => Ok(RecurrenceType::Yearly),
        _ => Err(anyhow::anyhow!("Invalid recurrence type. Valid options are: none, daily, weekly, monthly, yearly"))
    }
}
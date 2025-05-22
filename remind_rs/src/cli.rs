use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};

use crate::reminder::RecurrenceType;

#[derive(Parser)]
#[command(name = "remind-rs")]
#[command(about = "A simple CLI reminder application", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new reminder
    Add {
        /// Text of the reminder
        #[arg(short, long)]
        text: String,

        /// Due date and time (YYYY-MM-DD HH:MM)
        #[arg(short = 'd', long)]
        time: String,

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
        
        /// Index number shown in the list
        #[arg(short = 'n', long, group = "delete_selector")]
        index: Option<usize>,
    },

    /// Edit an existing reminder
    Edit {
        /// ID of the reminder to edit
        #[arg(short, long)]
        id: String,

        /// New text for the reminder (optional)
        #[arg(short, long)]
        text: Option<String>,

        /// New due date and time (optional, YYYY-MM-DD HH:MM)
        #[arg(short, long)]
        time: Option<String>,

        /// New recurrence pattern (optional)
        #[arg(short, long)]
        recurrence: Option<String>,
    },

    /// Check for due reminders and send notifications
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
        /// Search term to match against reminder text
        #[arg(short, long)]
        query: String,
    },

    /// Show detailed help about available commands
    Help {
        /// Get help about a specific command
        #[arg(short, long)]
        command: Option<String>,
    },
}

// Helper functions for parsing input
pub fn parse_datetime(datetime_str: &str) -> Result<DateTime<Local>> {
    let naive_datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M")
        .context("Invalid date time format. Expected YYYY-MM-DD HH:MM")?;
    
    let local_datetime = Local.from_local_datetime(&naive_datetime)
        .single()
        .context("Failed to convert to local datetime")?;
    
    Ok(local_datetime)
}

pub fn parse_recurrence(recurrence_str: &str) -> Result<RecurrenceType> {
    match recurrence_str.to_lowercase().as_str() {
        "none" => Ok(RecurrenceType::None),
        "daily" => Ok(RecurrenceType::Daily),
        "weekly" => Ok(RecurrenceType::Weekly),
        "monthly" => Ok(RecurrenceType::Monthly),
        "yearly" => Ok(RecurrenceType::Yearly),
        custom => Ok(RecurrenceType::Custom(custom.to_string())), // For future extension
    }
}
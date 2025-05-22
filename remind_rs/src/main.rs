mod cli;
mod reminder;
mod storage;
mod notification;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use reminder::Reminder;
use storage::Storage;
use notification::Notifier;

fn main() {
    if let Err(e) = run() {
        // Print the error and its causes
        eprintln!("Error: {}", e);
        
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("Caused by: {}", cause);
            source = cause.source();
        }
        
        // Only exit for critical errors, otherwise continue
        if is_critical_error(&e) {
            eprintln!("Critical error - exiting program.");
            std::process::exit(1);
        }
    }
}

// Determine if an error is critical enough to exit the program
fn is_critical_error(error: &anyhow::Error) -> bool {
    // Storage initialization errors are critical
    if error.to_string().contains("Failed to initialize storage") {
        return true;
    }
    
    // Access permission errors are critical
    if error.to_string().contains("permission denied") {
        return true;
    }
    
    // Other errors can be handled gracefully without exiting
    false
}

fn run() -> Result<()> {
    // Initialize the storage
    let storage = Storage::new()
        .context("Failed to initialize storage")?;
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Handle commands
    match cli.command {
        Commands::Add { text, time, recurrence } => {
            let due_time = cli::parse_datetime(&time)
                .context("Failed to parse date and time")?;
            let recurrence_type = cli::parse_recurrence(&recurrence)
                .context("Failed to parse recurrence")?;
            
            let reminder = Reminder::new(text, due_time, recurrence_type);
            storage.add_reminder(reminder)?;
            println!("Reminder added successfully.");
        }
        
        Commands::List => {
            let reminders = storage.load()?;
            if reminders.is_empty() {
                println!("No reminders found.");
            } else {
                println!("Your Reminders:");
                for (i, reminder) in reminders.iter().enumerate() {
                    println!("{}. {}", i + 1, reminder);
                }
            }
        }
        
        Commands::Delete { id, index } => {
            if let Some(id_str) = id {
                let success = storage.delete_reminder(&id_str)?;
                if success {
                    println!("Reminder deleted successfully.");
                } else {
                    println!("No reminder found with that ID.");
                }
            } else if let Some(idx) = index {
                let reminders = storage.load()?;
                if idx == 0 || idx > reminders.len() {
                    println!("Invalid index. Use 'list' to see available reminders.");
                } else {
                    let id_to_delete = &reminders[idx - 1].id;
                    storage.delete_reminder(id_to_delete)?;
                    println!("Reminder deleted successfully.");
                }
            } else {
                println!("Please provide either an ID or an index.");
            }
        }
        
        Commands::Edit { id, text, time, recurrence } => {
            let reminder_option = storage.get_reminder_by_id(&id)?;
            
            if let Some(mut reminder) = reminder_option {
                if let Some(new_text) = text {
                    reminder.text = new_text;
                }
                
                if let Some(new_time) = time {
                    reminder.due_time = cli::parse_datetime(&new_time)?;
                }
                
                if let Some(new_recurrence) = recurrence {
                    reminder.recurrence = cli::parse_recurrence(&new_recurrence)?;
                }
                
                storage.update_reminder(reminder)?;
                println!("Reminder updated successfully.");
            } else {
                println!("No reminder found with that ID.");
            }
        }
        
        Commands::Notify { desktop } => {
            let notifier = Notifier::new(storage);
            let due_reminders = notifier.check_due_reminders(desktop)?;
            
            if due_reminders.is_empty() {
                println!("No reminders due.");
            } else {
                println!("{} reminder(s) notified.", due_reminders.len());
            }
        }
        
        Commands::Export => {
            let reminders = storage.load()?;
            let json = serde_json::to_string_pretty(&reminders)
                .context("Failed to serialize reminders")?;
            println!("{}", json);
        }
    }
    
    Ok(())
}
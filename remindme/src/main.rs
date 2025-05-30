mod cli;
mod reminder;
mod storage;
mod notification;
mod utils;
mod tui;  
mod sound;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use reminder::Reminder;
use storage::Storage;
use notification::Notifier;
use chrono::{DateTime, Datelike, Local};

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
    let mut storage = Storage::new()
        .context("Failed to initialize storage")?;
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // If TUI mode is requested, start the TUI
    if cli.tui {
        return tui::start_tui(storage);
    }
    
    // Otherwise, continue with CLI mode
    match cli.command {
        Some(Commands::Add { text, time, date, recurrence }) => {
            // Use the helper function to parse time with default date logic
            let due_time = cli::parse_datetime_with_default_date(&time, date.as_deref())?;
            
            let recurrence_type = cli::parse_recurrence(&recurrence)?;
            let reminder = Reminder::new(text, due_time, recurrence_type);
            storage.add_reminder(reminder)?;
            println!("Reminder added successfully.");
        },
        
        Some(Commands::List) => {
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
        
        Some(Commands::Delete { id, index }) => {
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
        
        Some(Commands::Edit { id, text, time, recurrence }) => {
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
        
        Some(Commands::Notify { desktop }) => {
            let mut notifier = Notifier::new(storage);
            let due_reminders = notifier.check_due_reminders(desktop)?;
            
            if due_reminders.is_empty() {
                println!("No reminders due.");
            } else {
                println!("{} reminder(s) notified.", due_reminders.len());
            }
        }
        
        Some(Commands::Export) => {
            let reminders = storage.load()?;
            let json = serde_json::to_string_pretty(&reminders)
                .context("Failed to serialize reminders")?;
            println!("{}", json);
        }

        Some(Commands::Stats) => {
            let reminders = storage.load()?;
            let total = reminders.len();
            let completed = reminders.iter().filter(|r| r.completed).count();
            let due_today = reminders.iter()
                .filter(|r| !r.completed && is_today(&r.due_time))
                .count();
            let overdue = reminders.iter()
                .filter(|r| !r.completed && r.due_time < Local::now())
                .count();
                
            println!("Reminder Statistics:");
            println!("  Total: {}", total);
            println!("  Completed: {}", completed);
            println!("  Active: {}", total - completed);
            println!("  Due today: {}", due_today);
            println!("  Overdue: {}", overdue);
        }

        Some(Commands::Search { query }) => {
            let reminders = storage.load()?;
            let matches: Vec<_> = reminders.iter()
                .filter(|r| r.text.to_lowercase().contains(&query.to_lowercase()))
                .collect();
            
            if matches.is_empty() {
                println!("No reminders matching '{}'", query);
            } else {
                println!("Reminders matching '{}':", query);
                for (i, reminder) in matches.iter().enumerate() {
                    println!("{}. {}", i + 1, reminder);
                }
            }
        }

        Some(Commands::Help { command }) => {
            if let Some(cmd) = command {
                match cmd.to_lowercase().as_str() {
                    "add" => {
                        println!("Add a new reminder:");
                        println!("  remind add --text \"Your reminder text\" --time \"HH:MM\" [--date \"YYYY-MM-DD\"] [--recurrence daily|weekly|monthly|yearly] [--priority low|medium|high|urgent]");
                        println!("\nExamples:");
                        println!("  remind add --text \"Team meeting\" --time \"10:00\" --date \"2025-05-24\"");
                        println!("  remind add --text \"Daily standup\" --time \"09:00\" --recurrence daily");
                        println!("  remind add --text \"Urgent deadline\" --time \"17:00\" --date \"2025-05-30\" --priority high");
                    },
                    "list" => {
                        println!("List all reminders:");
                        println!("  remind list");
                        println!("\nThis command shows all your reminders with their IDs, text, due time, and status.");
                    },
                    "delete" => {
                        println!("Delete a reminder:");
                        println!("  remind delete --id [ID]");
                        println!("  remind delete --index [NUMBER]");
                        println!("\nExamples:");
                        println!("  remind delete --id c7613d0e");
                        println!("  remind delete --index 2");
                        println!("\nUse the list command first to see reminder IDs and indexes.");
                    },
                    "edit" => {
                        println!("Edit an existing reminder:");
                        println!("  remind edit --id [ID] [--text \"New text\"] [--time \"YYYY-MM-DD HH:MM\"] [--recurrence daily|weekly|monthly|yearly] [--priority low|medium|high|urgent]");
                        println!("\nExamples:");
                        println!("  remind edit --id c7613d0e --text \"Updated reminder\"");
                        println!("  remind edit --id c7613d0e --time \"2025-06-01 14:00\" --recurrence weekly");
                    },
                    "notify" => {
                        println!("Check for due reminders and get notifications:");
                        println!("  remind notify [--desktop]");
                        println!("\nOptions:");
                        println!("  --desktop    Send desktop notifications");
                        println!("\nThis command checks for due reminders and notifies you about them.");
                        println!("Use with --desktop to get desktop notifications instead of just terminal output.");
                    },
                    // Add other commands
                    _ => {
                        println!("Unknown command: {}", cmd);
                        println!("Run 'remind help' to see all available commands.");
                    }
                }
            } else {
                display_general_help();
            }
        },
        
        None => {
            // If no command was provided and not in TUI mode, show help
            display_general_help();
        }
    }
    
    Ok(())
}

// Helper function
fn is_today(dt: &DateTime<Local>) -> bool {
    let now = Local::now();
    dt.year() == now.year() && dt.month() == now.month() && dt.day() == now.day()
}

fn display_general_help() {
    println!("REMINDER CLI - A command line reminder application");
    println!("\nAVAILABLE COMMANDS:");
    println!("  add       Add a new reminder");
    println!("  list      List all reminders");
    println!("  delete    Delete a reminder by ID or index");
    println!("  edit      Edit an existing reminder");
    println!("  notify    Check for due reminders and send notifications");
    println!("  complete  Mark a reminder as completed or not completed");
    println!("  export    Export reminders as JSON");
    println!("  search    Search for reminders");
    println!("  stats     Show statistics about reminders");
    println!("  help      Show this help message or help for a specific command");
    
    println!("\nFor detailed help on a specific command, run:");
    println!("  remind help --command COMMAND");
    
    println!("\nEXAMPLES:");
    println!("  remind add --text \"Team meeting\" --time \"10:00\" --date \"2025-05-24\"");
    println!("  remind list");
    println!("  remind notify --desktop");
    println!("  remind help --command add");
    
    println!("\nSETUP AS SYSTEM SERVICE:");
    println!("  To receive automatic notifications, set up the systemd timer:");
    println!("    1. mkdir -p ~/.config/systemd/user/");
    println!("    2. Create timer files (see documentation)");
    println!("    3. systemctl --user enable remind-check.timer");
    println!("    4. systemctl --user start remind-check.timer");
}
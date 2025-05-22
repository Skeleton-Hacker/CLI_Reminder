use anyhow::{anyhow, Context, Result}; // Added anyhow macro here
use crate::reminder::Reminder;
use serde_json;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Storage {
    file_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Failed to determine config directory"))?
            .join("remindme"); // Changed from "remind-rs" to "remindme"
        
        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;
        
        let file_path = config_dir.join("reminders.json");
        
        Ok(Storage { file_path })
    }

    pub fn load(&self) -> Result<Vec<Reminder>> {
        // Create empty file if it doesn't exist
        if !self.file_path.exists() {
            File::create(&self.file_path)?;
            return Ok(Vec::new());
        }

        // Read file contents
        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Handle empty file
        if contents.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Parse JSON
        let reminders: Vec<Reminder> = serde_json::from_str(&contents)
            .context("Failed to parse reminders from JSON")?;
        
        Ok(reminders)
    }

    pub fn save(&self, reminders: &[Reminder]) -> Result<()> {
        let json = serde_json::to_string_pretty(reminders)
            .context("Failed to serialize reminders to JSON")?;
        
        let mut file = File::create(&self.file_path)
            .context("Failed to create or open reminders file")?;
        
        file.write_all(json.as_bytes())
            .context("Failed to write reminders to file")?;
        
        Ok(())
    }

    pub fn add_reminder(&self, reminder: Reminder) -> Result<()> {
        let mut reminders = self.load()?;
        reminders.push(reminder);
        self.save(&reminders)?;
        Ok(())
    }

    pub fn delete_reminder(&self, id: &str) -> Result<bool> {
        let mut reminders = self.load()?;
        let initial_len = reminders.len();
        reminders.retain(|r| r.id != id);
        
        if reminders.len() == initial_len {
            return Ok(false); // No reminder was deleted
        }
        
        self.save(&reminders)?;
        Ok(true)
    }

    pub fn update_reminder(&self, updated_reminder: Reminder) -> Result<bool> {
        let mut reminders = self.load()?;
        let found = reminders.iter_mut().any(|r| {
            if r.id == updated_reminder.id {
                *r = updated_reminder.clone();
                true
            } else {
                false
            }
        });
        
        if found {
            self.save(&reminders)?;
        }
        
        Ok(found)
    }

    pub fn get_reminder_by_id(&self, id: &str) -> Result<Option<Reminder>> {
        let reminders = self.load()?;
        Ok(reminders.into_iter().find(|r| r.id == id))
    }
}
use crate::reminder::Reminder;
use crate::storage::Storage;
use crate::sound;
use anyhow::Result;
use notify_rust::Notification;

pub struct Notifier {
    pub storage: Storage,
}

impl Notifier {
    pub fn new(storage: Storage) -> Self {
        Notifier { storage }
    }

    pub fn check_due_reminders(&mut self, send_desktop: bool) -> Result<Vec<Reminder>> {
        let mut reminders = self.storage.load()?;
        let mut due_reminders = Vec::new();
        
        for reminder in reminders.iter_mut() {
            if reminder.is_due() {
                due_reminders.push(reminder.clone());
                
                // Notify in terminal
                println!("REMINDER: {}", reminder.text);
                
                // Send desktop notification if requested
                if send_desktop {
                    self.send_desktop_notification(reminder)?;
                }
                
                // Mark as notified and update
                reminder.mark_notified();
                self.storage.update_reminder(reminder.clone())?;
            }
        }
        
        Ok(due_reminders)
    }
    
    fn send_desktop_notification(&self, reminder: &Reminder) -> Result<()> {
        println!("Sending desktop notification for: {}", reminder.text);
        
        // Show the notification
        Notification::new()
            .summary("RemindMe Reminder")
            .body(&reminder.text)
            .icon("appointment-soon")
            .timeout(5000)
            .show()?;
        
        // Play notification sound
        if let Err(e) = sound::play_notification_sound() {
            // Just log the error but don't fail the notification
            println!("Failed to play notification sound: {}", e);
        }
        
        println!("Desktop notification sent successfully");
        Ok(())
    }
}
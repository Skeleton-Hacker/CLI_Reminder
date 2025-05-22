use crate::reminder::Reminder;
use crate::storage::Storage;
use anyhow::Result;
use notify_rust::Notification; 

pub struct Notifier {
    storage: Storage,
}

impl Notifier {
    pub fn new(storage: Storage) -> Self {
        Notifier { storage }
    }

    pub fn check_due_reminders(&self, send_desktop: bool) -> Result<Vec<Reminder>> {
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
        // Add a print statement for debugging
        println!("Attempting to send desktop notification for: {}", reminder.text);
        
        Notification::new()
            .summary("RemindMe Reminder")
            .body(&reminder.text)
            .icon("appointment-soon")  // Use a standard icon
            .timeout(5000)  // 5 seconds
            .show()?;
        
        println!("Desktop notification sent successfully");
        Ok(())
    }
}
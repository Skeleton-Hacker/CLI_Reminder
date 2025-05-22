use crate::reminder::Reminder;
use crate::storage::Storage;
use anyhow::Result;

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
                #[cfg(not(target_os = "windows"))]
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
    
    #[cfg(not(target_os = "windows"))]
    fn send_desktop_notification(&self, reminder: &Reminder) -> Result<()> {
        use notify_rust::Notification;
        
        Notification::new()
            .summary("Reminder")
            .body(&reminder.text)
            .timeout(5000) // milliseconds
            .show()?;
        
        Ok(())
    }
}
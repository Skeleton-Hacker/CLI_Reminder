# RemindMe 

RemindMe is a CLI reminder application written in Rust. It provides a simple way to manage reminders directly from your terminal, with automatic desktop notifications when reminders are due, without the need to be connected to the internet.

## Features

- **Simple CLI Interface**: Easy-to-remember commands for managing reminders
- **Recurring Reminders**: Set daily, weekly, monthly, or yearly recurring reminders
- **Desktop Notifications**: Get notified with desktop alerts when reminders are due
- **Automatic Background Checks**: System integration checks for due reminders every minute
- **Flexible Management**: List, add, edit, and delete reminders easily
- **Smart Date Defaults**: Automatically uses today or tomorrow when only time is specified

## Installation

### Prerequisites

- Rust/Cargo (1.56.0 or newer)
- Linux system with systemd (for automatic notifications)
- libdbus development libraries (for desktop notifications)

### Quick Install

1. Clone this repository:
   ```bash
   git clone https://github.com/Skeleton-Hacker/CLI_Reminder.git
   ```

2. Run the setup script:
   ```bash
   ./setup.sh
   ```

The script will:
- Build the application from source
- Install it to your `~/.local/bin` directory
- Set up systemd services for automatic notifications
- Create an uninstall script and documentation

## Usage

### Core Commands

```bash
# Show help and available commands
remindme help

# Add a new reminder (with automatic date selection)
remindme add --text "Buy groceries" --time "18:00"

# Add a reminder with specific date
remindme add --text "Doctor appointment" --time "14:30" --date "2023-09-18"

# Add a recurring reminder
remindme add --text "Weekly team meeting" --time "10:00" --recurrence weekly

# List all reminders
remindme list

# Delete a reminder by ID
remindme delete --id c7613d0e

# Delete a reminder by index
remindme delete --index 2

# Edit a reminder
remindme edit --id c7613d0e --text "Updated reminder text" --time "10:00"

# Check for due reminders manually
remindme notify
# With desktop notifications
remindme notify --desktop
```

### Command Details

**Adding Reminders**:
```bash
# Basic syntax
remindme add --text "Your reminder text" --time "HH:MM" [--date "YYYY-MM-DD"] [--recurrence daily|weekly|monthly|yearly]

# When only time is provided, date defaults to:
# - Today if the time hasn't passed yet
# - Tomorrow if the time has already passed today
```

**Listing Reminders**:
```bash
remindme list
```

**Editing Reminders**:
```bash
remindme edit --id [ID] [--text "New text"] [--time "HH:MM"] [--date "YYYY-MM-DD"] [--recurrence none|daily|weekly|monthly|yearly]
```

**Deleting Reminders**:
```bash
remindme delete --id [ID]
# or
remindme delete --index [NUMBER]
```

## Automatic Notifications

After installation, RemindMe will check for due reminders every minute and display desktop notifications automatically. This is handled by a systemd user service.

## Configuration

All reminders are stored in `~/.config/remindme/reminders.json`. While you shouldn't need to edit this file directly, it's a simple JSON format for your reminders.

## TUI Mode

RemindMe now features an interactive Text User Interface (TUI) mode:

'''bash
# Launch RemindMe in TUI mode
remindme --tui
'''

In TUI mode, you can:
- View all reminders in a scrollable list
- Add new reminders with a form interface
- Delete reminders with a single keystroke
- Navigate with keyboard shortcuts

### TUI Keyboard Shortcuts

- `q`: Quit the application
- `a`: Add a new reminder
- `d`: Delete the selected reminder
- `h`: View help screen
- `l`: Return to reminder list
- `↑/↓`: Navigate through reminders

## Uninstallation

To uninstall RemindMe:

```bash
~/.config/remindme/uninstall.sh
```

## Troubleshooting

- **No desktop notifications**: Make sure your system's notification daemon is running
- **Service not running**: Check systemd status with `systemctl --user status remindme-check.timer`
- **Missing command**: Run `source ~/.bashrc` or restart your terminal if the command isn't found

## Development

RemindMe is built with Rust and uses the following major components:

- **clap**: For command-line argument parsing
- **chrono**: For date and time handling
- **serde/serde_json**: For JSON serialization and storage
- **notify-rust**: For desktop notifications
- **anyhow**: For error handling

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---
For additional help or questions, please open an issue in the GitHub repository.
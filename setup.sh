#!/bin/bash

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}RemindMe CLI Installation Script${NC}"
echo -e "${BLUE}========================================${NC}"

# Check if required tools are installed
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: Rust/Cargo is required but not installed. Please install Rust first.${NC}"; exit 1; }
command -v systemctl >/dev/null 2>&1 || { echo -e "${RED}Warning: systemctl not found. Automatic notifications might not work.${NC}"; }

# Define key paths
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$SCRIPT_DIR/remindme"
BINARY_NAME="remindme"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/remindme"
SYSTEMD_DIR="$HOME/.config/systemd/user"

# Check if project directory exists
if [ ! -d "$PROJECT_DIR" ]; then
    echo -e "${RED}Error: Project directory $PROJECT_DIR not found.${NC}"
    exit 1
fi

# Check if Cargo.toml exists
if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found in $PROJECT_DIR.${NC}"
    exit 1
fi

echo -e "${YELLOW}Building RemindMe from source...${NC}"
cd "$PROJECT_DIR"
if cargo build --release; then
    echo -e "${GREEN}Build successful!${NC}"
else
    echo -e "${RED}Build failed. Please fix the errors and try again.${NC}"
    exit 1
fi

echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$SYSTEMD_DIR"

echo -e "${YELLOW}Installing binary...${NC}"
cp "$PROJECT_DIR/target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Add binary directory to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Adding $INSTALL_DIR to your PATH...${NC}"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo -e "${GREEN}Added to PATH in .bashrc. Please restart your terminal or run 'source ~/.bashrc'${NC}"
fi

# Create systemd service file
echo -e "${YELLOW}Creating systemd service file...${NC}"
cat > "$SYSTEMD_DIR/remindme-check.service" << EOF
[Unit]
Description=Check for due reminders

[Service]
Type=oneshot
ExecStart=$INSTALL_DIR/$BINARY_NAME notify --desktop
Environment=DISPLAY=:0
Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u)/bus
EOF

# Create systemd timer file
echo -e "${YELLOW}Creating systemd timer file...${NC}"
cat > "$SYSTEMD_DIR/remindme-check.timer" << EOF
[Unit]
Description=Periodically check for due reminders

[Timer]
OnBootSec=1min
OnUnitActiveSec=1min
AccuracySec=1s

[Install]
WantedBy=timers.target
EOF

# Create desktop entry
echo -e "${YELLOW}Creating desktop entry...${NC}"
mkdir -p "$HOME/.local/share/applications"
cat > "$HOME/.local/share/applications/remindme.desktop" << EOF
[Desktop Entry]
Name=RemindMe
Comment=CLI Reminder Application
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=appointment-soon
Terminal=true
Type=Application
Categories=Utility;
EOF

# Reload systemd and start timer
echo -e "${YELLOW}Configuring systemd services...${NC}"
systemctl --user daemon-reload
systemctl --user enable remindme-check.timer
systemctl --user start remindme-check.timer

# Enable linger for the user to ensure services run even when not logged in
if command -v loginctl >/dev/null 2>&1; then
    echo -e "${YELLOW}Enabling lingering user session...${NC}"
    loginctl enable-linger "$(whoami)" >/dev/null 2>&1
fi

# Create an uninstall script for future use
echo -e "${YELLOW}Creating uninstall script...${NC}"
cat > "$CONFIG_DIR/uninstall.sh" << EOF
#!/bin/bash
echo "Uninstalling RemindMe..."

# Stop and disable services
systemctl --user stop remindme-check.timer
systemctl --user disable remindme-check.timer
systemctl --user stop remindme-check.service

# Remove systemd files
rm -f ~/.config/systemd/user/remindme-check.service
rm -f ~/.config/systemd/user/remindme-check.timer
systemctl --user daemon-reload

# Remove binary
rm -f ~/.local/bin/remindme

# Remove desktop file
rm -f ~/.local/share/applications/remindme.desktop

# Ask about config files
read -p "Do you want to remove all reminders and configuration? (y/N) " -n 1 -r
echo
if [[ \$REPLY =~ ^[Yy]$ ]]; then
    rm -rf ~/.config/remindme
    echo "All reminders and configuration removed."
else
    echo "Kept reminders and configuration at ~/.config/remindme"
fi

echo "RemindMe has been uninstalled."
EOF
chmod +x "$CONFIG_DIR/uninstall.sh"

# Create a user guide
# Update the user guide to include TUI information
echo -e "${YELLOW}Creating user guide...${NC}"
cat > "$CONFIG_DIR/README.md" << EOF
# RemindMe - CLI Reminder Application

A simple but powerful command-line reminder application.

## Usage

\`\`\`bash
# Get help
remindme help

# Add a reminder
remindme add --text "Meeting with team" --time "2023-05-25 10:00"

# Add a recurring reminder
remindme add --text "Weekly meeting" --time "2023-05-25 10:00" --recurrence weekly

# List reminders
remindme list

# Delete a reminder
remindme delete --id [ID]
remindme delete --index [INDEX]

# Check for notifications manually
remindme notify --desktop

# Launch the TUI mode
remindme --tui
\`\`\`

## TUI Mode

RemindMe features an interactive Text User Interface (TUI) mode:

\`\`\`bash
remindme --tui
\`\`\`

In TUI mode, you can:
- View all reminders in a scrollable list
- Add new reminders with a form interface
- Edit existing reminders
- Delete reminders with a single keystroke
- Navigate with keyboard shortcuts

### TUI Keyboard Shortcuts

- \`q\`: Quit the application
- \`a\`: Add a new reminder
- \`e\`: Edit the selected reminder
- \`d\`: Delete the selected reminder
- \`h\`: View help screen
- \`l\`: Return to reminder list
- \`↑/↓\`: Navigate through reminders

## Automatic Notifications

RemindMe checks for due reminders every minute and shows desktop notifications.
This was configured during installation.

## Configuration

Reminders are stored in \`~/.config/remindme/reminders.json\`.

## Uninstalling

To uninstall RemindMe, run:

\`\`\`bash
~/.config/remindme/uninstall.sh
\`\`\`

EOF

# Update the test options to include TUI mode
echo
echo -e "${YELLOW}Test Options:${NC}"
echo "1) Create a basic test reminder (2 minutes from now)"
echo "2) Test default date logic (today or tomorrow based on time)"
echo "3) Test recurring reminder"
echo "4) Launch TUI mode"
echo "5) Skip test reminders"
read -p "Select an option (1-5): " test_option
echo

case $test_option in
    1)
        # Get current time + 2 minutes
        CURRENT_HOUR=$(date +%H)
        MINUTES=$(date -d '+2 minutes' +%M)
        TEST_TIME="${CURRENT_HOUR}:${MINUTES}"
        
        # Test with only time parameter (should use default date)
        $INSTALL_DIR/$BINARY_NAME add --text "Test with default date" --time "$TEST_TIME"
        
        # Also test with explicit date parameter
        TOMORROW=$(date -d 'tomorrow' '+%Y-%m-%d')
        $INSTALL_DIR/$BINARY_NAME add --text "Test with explicit date" --time "$TEST_TIME" --date "$TOMORROW"
        
        echo -e "${GREEN}Created two test reminders:${NC}"
        echo "1. For time $TEST_TIME (should use today as default date)"
        echo "2. For time $TEST_TIME with explicit date $TOMORROW"
        echo -e "${YELLOW}Running 'remindme list' to show results:${NC}"
        $INSTALL_DIR/$BINARY_NAME list
        ;;
    2)
        # Current hour and 5 minutes from now
        CURRENT_HOUR=$(date +%H)
        FIVE_MIN_LATER=$(date -d '+5 minutes' +%M)
        TEST_TIME="${CURRENT_HOUR}:${FIVE_MIN_LATER}"
        
        # Time that's probably tomorrow (3 AM)
        EARLY_MORNING="03:00"
        
        # Add reminders with just the time parameter
        $INSTALL_DIR/$BINARY_NAME add --text "Test current day default" --time "$TEST_TIME"
        $INSTALL_DIR/$BINARY_NAME add --text "Test next day default" --time "$EARLY_MORNING"
        
        echo -e "${GREEN}Created two test reminders:${NC}"
        echo "1. Using time $TEST_TIME (should use today's date)"
        echo "2. Using time $EARLY_MORNING (likely uses tomorrow's date)"
        echo -e "${YELLOW}Running 'remindme list' to show results:${NC}"
        $INSTALL_DIR/$BINARY_NAME list
        ;;
    3)
        # Create a daily recurring reminder for 9 AM
        $INSTALL_DIR/$BINARY_NAME add --text "Daily recurring reminder" --time "09:00" --recurrence daily
        echo -e "${GREEN}Created a daily recurring reminder for 9:00 AM${NC}"
        echo -e "${YELLOW}Running 'remindme list' to show results:${NC}"
        $INSTALL_DIR/$BINARY_NAME list
        ;;
    4)
        # Launch the TUI mode
        echo -e "${YELLOW}Launching TUI mode. Use 'q' to quit when done.${NC}"
        $INSTALL_DIR/$BINARY_NAME --tui
        ;;
    5)
        echo "Skipping test reminders."
        ;;
    *)
        echo "Invalid option. Skipping test reminders."
        ;;
esac
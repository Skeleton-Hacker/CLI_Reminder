use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text, Line}, // Add Line import
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::reminder::Reminder;
use crate::storage::Storage;
use crate::cli; 

#[derive(PartialEq, Eq)] // Add these derive macros
enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq, Eq)] // Add these derive macros
enum CurrentView {
    List,
    Add,
    Edit,
    Help,
}

// Add this enum to track which field is active in the form
#[derive(PartialEq, Eq, Clone, Copy)]
enum ActiveField {
    Text,
    Time,
    Date,
    Recurrence,
    Submit,
}

#[allow(dead_code)]
struct App {
    reminders: Vec<Reminder>,
    storage: Storage,
    current_view: CurrentView,
    input_mode: InputMode,
    input: String,
    selected_index: usize,
    new_reminder_text: String,
    new_reminder_time: String,
    new_reminder_date: String,
    new_reminder_recurrence: String,
    editing_reminder_id: Option<String>, // Add this field for editing
    active_field: ActiveField,   // Add this field
    error_message: Option<String>,
}

impl App {
    fn new(storage: Storage) -> Result<Self> {
        let reminders = storage.load()?;
        
        Ok(Self {
            reminders,
            storage,
            current_view: CurrentView::List,
            input_mode: InputMode::Normal,
            input: String::new(),
            selected_index: 0,
            new_reminder_text: String::new(),
            new_reminder_time: String::new(),
            new_reminder_date: String::new(),
            new_reminder_recurrence: String::from("none"), // Initialize with default value
            editing_reminder_id: None, // No reminder being edited initially
            active_field: ActiveField::Text,  // Initialize to first field
            error_message: None,
        })
    }
    
    // Add method to get current active input based on field
    fn get_active_input_mut(&mut self) -> &mut String {
        match self.active_field {
            ActiveField::Text => &mut self.new_reminder_text,
            ActiveField::Time => &mut self.new_reminder_time,
            ActiveField::Date => &mut self.new_reminder_date,
            ActiveField::Recurrence => &mut self.new_reminder_recurrence,
            ActiveField::Submit => &mut self.input, // Dummy, not used
        }
    }
    
    // Add method to create a reminder from form data
    fn create_reminder(&mut self) -> Result<()> {
        // Validate fields
        if self.new_reminder_text.is_empty() {
            self.error_message = Some("Reminder text cannot be empty".to_string());
            return Ok(());
        }
        
        if self.new_reminder_time.is_empty() {
            self.error_message = Some("Time must be specified (HH:MM)".to_string());
            return Ok(());
        }
        
        // Parse time with optional date
        let date_option = if self.new_reminder_date.is_empty() {
            None
        } else {
            Some(self.new_reminder_date.as_str())
        };
        
        match cli::parse_datetime_with_default_date(&self.new_reminder_time, date_option) {
            Ok(due_time) => {
                // Parse recurrence
                let recurrence_type = match cli::parse_recurrence(&self.new_reminder_recurrence) {
                    Ok(rec) => rec,
                    Err(e) => {
                        self.error_message = Some(format!("Invalid recurrence: {}", e));
                        return Ok(());
                    }
                };
                
                // Create and save the reminder
                let reminder = Reminder::new(
                    self.new_reminder_text.clone(),
                    due_time,
                    recurrence_type
                );
                
                self.storage.add_reminder(reminder)?;
                
                // Clear form and error
                self.new_reminder_text.clear();
                self.new_reminder_time.clear();
                self.new_reminder_date.clear();
                self.new_reminder_recurrence = "none".to_string();
                self.error_message = None;
                
                // Return to list view
                self.current_view = CurrentView::List;
                self.input_mode = InputMode::Normal;
                self.refresh_reminders()?;
                
                Ok(())
            },
            Err(e) => {
                self.error_message = Some(format!("Invalid date/time: {}", e));
                Ok(())
            }
        }
    }
    
    fn update_reminder(&mut self) -> Result<()> {
        // Validate fields
        if self.new_reminder_text.is_empty() {
            self.error_message = Some("Reminder text cannot be empty".to_string());
            return Ok(());
        }
        
        if self.new_reminder_time.is_empty() {
            self.error_message = Some("Time must be specified (HH:MM)".to_string());
            return Ok(());
        }
        
        let date_option = if self.new_reminder_date.is_empty() {
            None
        } else {
            Some(self.new_reminder_date.as_str())
        };
        
        match cli::parse_datetime_with_default_date(&self.new_reminder_time, date_option) {
            Ok(due_time) => {
                // Parse recurrence
                let recurrence_type = match cli::parse_recurrence(&self.new_reminder_recurrence) {
                    Ok(rec) => rec,
                    Err(e) => {
                        self.error_message = Some(format!("Invalid recurrence: {}", e));
                        return Ok(());
                    }
                };
                
                if let Some(id) = &self.editing_reminder_id {
                    // Create updated reminder
                    let updated_reminder = Reminder::new_with_id(
                        id.clone(),
                        self.new_reminder_text.clone(),
                        due_time,
                        recurrence_type
                    );
                    
                    // Update in storage
                    self.storage.update_reminder(updated_reminder)?;
                    
                    // Clear form and editing state
                    self.new_reminder_text.clear();
                    self.new_reminder_time.clear();
                    self.new_reminder_date.clear();
                    self.new_reminder_recurrence = "none".to_string();
                    self.editing_reminder_id = None;
                    self.error_message = None;
                    
                    // Return to list view
                    self.current_view = CurrentView::List;
                    self.input_mode = InputMode::Normal;
                    self.refresh_reminders()?;
                }
                
                Ok(())
            },
            Err(e) => {
                self.error_message = Some(format!("Invalid date/time: {}", e));
                Ok(())
            }
        }
    }
    
    fn refresh_reminders(&mut self) -> Result<()> {
        self.reminders = self.storage.load()?;
        Ok(())
    }

    fn start_editing_selected_reminder(&mut self) -> Result<()> {
        if self.reminders.is_empty() {
            return Ok(());
        }
        
        let reminder = &self.reminders[self.selected_index];
        
        // Store the ID of the reminder being edited
        self.editing_reminder_id = Some(reminder.id.clone());
        
        // Populate form fields with the reminder's data
        self.new_reminder_text = reminder.text.clone();
        self.new_reminder_time = reminder.due_time.format("%H:%M").to_string();
        self.new_reminder_date = reminder.due_time.format("%Y-%m-%d").to_string();
        self.new_reminder_recurrence = format!("{:?}", reminder.recurrence).to_lowercase();
        
        // Set the view and mode
        self.current_view = CurrentView::Edit;
        self.input_mode = InputMode::Editing;
        self.active_field = ActiveField::Text;
        self.error_message = None;
        
        Ok(())
    }
}

pub fn start_tui(storage: Storage) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Show cursor for text input
    terminal.show_cursor()?;

    // Create app state
    let mut app = App::new(storage)?;
    
    // Run the main loop
    let result = run_app(&mut terminal, &mut app);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        // First determine if cursor should be visible
        let show_cursor = app.input_mode == InputMode::Editing && app.current_view == CurrentView::Add;
        
        // Then draw the UI
        terminal.draw(|f| ui(f, app))?;
        
        // Update cursor visibility after drawing
        if show_cursor {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        app.current_view = CurrentView::Add;
                        app.input_mode = InputMode::Editing;
                        app.active_field = ActiveField::Text;
                        app.error_message = None;
                    },
                    KeyCode::Char('e') => {
                        if app.current_view == CurrentView::List && !app.reminders.is_empty() {
                            app.start_editing_selected_reminder()?;
                        }
                    },
                    // Other normal mode handlers remain the same
                    KeyCode::Char('h') => {
                        app.current_view = CurrentView::Help;
                    },
                    KeyCode::Char('l') => {
                        app.current_view = CurrentView::List;
                        app.refresh_reminders()?;
                    },
                    KeyCode::Char('d') => {
                        if app.current_view == CurrentView::List && !app.reminders.is_empty() {
                            let reminder = &app.reminders[app.selected_index];
                            app.storage.delete_reminder(&reminder.id)?;
                            app.refresh_reminders()?;
                            if app.selected_index >= app.reminders.len() && !app.reminders.is_empty() {
                                app.selected_index = app.reminders.len() - 1;
                            }
                        }
                    },
                    KeyCode::Up => {
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if !app.reminders.is_empty() && app.selected_index < app.reminders.len() - 1 {
                            app.selected_index += 1;
                        }
                    },
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.current_view = CurrentView::List;
                        app.error_message = None;
                    },
                    KeyCode::Enter => {
                        match app.current_view {
                            CurrentView::Add => {
                                match app.active_field {
                                    ActiveField::Text => app.active_field = ActiveField::Time,
                                    ActiveField::Time => app.active_field = ActiveField::Date,
                                    ActiveField::Date => app.active_field = ActiveField::Recurrence,
                                    ActiveField::Recurrence => app.active_field = ActiveField::Submit,
                                    ActiveField::Submit => app.create_reminder()?,
                                }
                            },
                            CurrentView::Edit => {
                                match app.active_field {
                                    ActiveField::Text => app.active_field = ActiveField::Time,
                                    ActiveField::Time => app.active_field = ActiveField::Date,
                                    ActiveField::Date => app.active_field = ActiveField::Recurrence,
                                    ActiveField::Recurrence => app.active_field = ActiveField::Submit,
                                    ActiveField::Submit => app.update_reminder()?,
                                }
                            },
                            _ => {}
                        }
                    },
                    KeyCode::Tab => {
                        // Cycle through fields in the add form
                        if app.current_view == CurrentView::Add {
                            match app.active_field {
                                ActiveField::Text => app.active_field = ActiveField::Time,
                                ActiveField::Time => app.active_field = ActiveField::Date,
                                ActiveField::Date => {
                                    app.active_field = ActiveField::Recurrence;
                                    // If the recurrence field is empty, initialize it with the default
                                    if app.new_reminder_recurrence.is_empty() {
                                        app.new_reminder_recurrence = String::from("none");
                                    }
                                },
                                ActiveField::Recurrence => app.active_field = ActiveField::Submit,
                                ActiveField::Submit => app.active_field = ActiveField::Text,
                            }
                        }
                    },
                    KeyCode::BackTab => {
                        // Cycle backwards through fields in the add form
                        if app.current_view == CurrentView::Add {
                            match app.active_field {
                                ActiveField::Text => app.active_field = ActiveField::Submit,
                                ActiveField::Time => app.active_field = ActiveField::Text,
                                ActiveField::Date => app.active_field = ActiveField::Time,
                                ActiveField::Recurrence => app.active_field = ActiveField::Date,
                                ActiveField::Submit => app.active_field = ActiveField::Recurrence,
                            }
                        }
                    },
                    KeyCode::Char(c) => {
                        if app.current_view == CurrentView::Add && app.active_field != ActiveField::Submit {
                            let input = app.get_active_input_mut();
                            input.push(c);
                        }
                    },
                    KeyCode::Backspace => {
                        if app.current_view == CurrentView::Add && app.active_field != ActiveField::Submit {
                            let input = app.get_active_input_mut();
                            input.pop();
                        }
                    },
                    _ => {},
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    // Create a layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(f.area());

    // Create the title bar
    let title = Paragraph::new("RemindMe - TUI")
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // Render the appropriate view
    match app.current_view {
        CurrentView::List => render_list_view(f, app, chunks[1]),
        CurrentView::Add => render_add_view(f, app, chunks[1]),
        CurrentView::Edit => render_edit_view(f, app, chunks[1]),
        CurrentView::Help => render_help_view(f, app, chunks[1]),
    }
    
    // Create the status bar with updated Text/Span handling
    let status = match app.current_view {
        CurrentView::List => {
            let spans = vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit, "),
                Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to edit, "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to delete, "),
                Span::styled("h", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for help"),
            ];
            Text::from(Line::from(spans))
        },
        CurrentView::Add => {
            let spans = vec![
                Span::raw("Press "),
                Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("/"),
                Span::styled("Shift+Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to move between fields, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit, "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ];
            Text::from(Line::from(spans))
        },
        CurrentView::Edit => {
            let spans = vec![
                Span::raw("Press "),
                Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("/"),
                Span::styled("Shift+Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to move between fields, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit, "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ];
            Text::from(Line::from(spans))
        },
        CurrentView::Help => {
            let spans = vec![
                Span::raw("Press "),
                Span::styled("l", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to go back to the list view"),
            ];
            Text::from(Line::from(spans))
        },
    };

    let status_bar = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, chunks[2]);
}

fn render_list_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<_> = app.reminders
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let status = if r.completed { "[✓]" } else { "[ ]" };
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(format!("{} {} - {}", status, r.text, r.due_time.format("%Y-%m-%d %H:%M")))
                .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Reminders").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

fn render_add_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Create a layout for the form
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Text field
            Constraint::Length(3),  // Time field
            Constraint::Length(3),  // Date field
            Constraint::Length(3),  // Recurrence field
            Constraint::Length(3),  // Submit button
            Constraint::Min(1),     // Error message area
        ].as_ref())
        .split(area);
    
    // Render the text field
    let text_style = if app.active_field == ActiveField::Text {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let text_input = Paragraph::new(app.new_reminder_text.as_str())
        .style(text_style)
        .block(Block::default()
            .title("Reminder Text")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Text {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(text_input, chunks[0]);
    
    // Render the time field
    let time_style = if app.active_field == ActiveField::Time {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let time_input = Paragraph::new(app.new_reminder_time.as_str())
        .style(time_style)
        .block(Block::default()
            .title("Time (HH:MM)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Time {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(time_input, chunks[1]);
    
    // Render the date field
    let date_style = if app.active_field == ActiveField::Date {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let date_text = if app.new_reminder_date.is_empty() {
        if app.active_field == ActiveField::Date && app.input_mode == InputMode::Editing {
            // Show empty string when actively editing an empty date field
            ""
        } else {
            // Show placeholder when not editing
            "(Optional - defaults to today/tomorrow)"
        }
    } else {
        app.new_reminder_date.as_str()
    };

    let date_input = Paragraph::new(date_text)
        .style(date_style)
        .block(Block::default()
            .title("Date (YYYY-MM-DD)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Date {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(date_input, chunks[2]);
    
    // Render the recurrence field
    let recurrence_style = if app.active_field == ActiveField::Recurrence {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let recurrence_input = Paragraph::new(app.new_reminder_recurrence.as_str())
        .style(recurrence_style)
        .block(Block::default()
            .title("Recurrence (none/daily/weekly/monthly/yearly)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Recurrence {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(recurrence_input, chunks[3]);
    
    // Render the submit button
    let submit_style = if app.active_field == ActiveField::Submit {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let submit_button = Paragraph::new("[ Add Reminder ]")
        .style(submit_style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Submit {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }));
    f.render_widget(submit_button, chunks[4]);
    
    // Render error message if any
    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default()
                .borders(Borders::NONE));
        f.render_widget(error_msg, chunks[5]);
    }
    
    // Set the cursor to the active field's end of text
    if app.active_field != ActiveField::Submit && app.input_mode == InputMode::Editing {
        let input = match app.active_field {
            ActiveField::Text => &app.new_reminder_text,
            ActiveField::Time => &app.new_reminder_time,
            ActiveField::Date => {
                if app.new_reminder_date.is_empty() { &app.input } else { &app.new_reminder_date }
            },
            ActiveField::Recurrence => &app.new_reminder_recurrence,
            _ => &app.input,
        };
        
        // Add 1 to x position to account for left border, and cursor inside the field
        let cursor_position = match app.active_field {
            ActiveField::Text => chunks[0].x + input.len() as u16 + 1,
            ActiveField::Time => chunks[1].x + input.len() as u16 + 1,
            ActiveField::Date => {
                if app.new_reminder_date.is_empty() {
                    // Position at start of input field for empty date
                    chunks[2].x + 1
                } else {
                    chunks[2].x + app.new_reminder_date.len() as u16 + 1
                }
            },
            ActiveField::Recurrence => chunks[3].x + input.len() as u16 + 1,
            _ => 0,
        };
        
        // Add 1 to y position to account for top border and title
        let cursor_y = match app.active_field {
            ActiveField::Text => chunks[0].y + 1,
            ActiveField::Time => chunks[1].y + 1,
            ActiveField::Date => chunks[2].y + 1,
            ActiveField::Recurrence => chunks[3].y + 1,
            _ => 0,
        };
        
        f.set_cursor_position((cursor_position, cursor_y));
    }
}

fn render_edit_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Create a layout for the form
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Text field
            Constraint::Length(3),  // Time field
            Constraint::Length(3),  // Date field
            Constraint::Length(3),  // Recurrence field
            Constraint::Length(3),  // Submit button
            Constraint::Min(1),     // Error message area
        ].as_ref())
        .split(area);
    
    // Render the text field
    let text_style = if app.active_field == ActiveField::Text {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let text_input = Paragraph::new(app.new_reminder_text.as_str())
        .style(text_style)
        .block(Block::default()
            .title("Reminder Text")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Text {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(text_input, chunks[0]);
    
    // Render the time field
    let time_style = if app.active_field == ActiveField::Time {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let time_input = Paragraph::new(app.new_reminder_time.as_str())
        .style(time_style)
        .block(Block::default()
            .title("Time (HH:MM)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Time {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(time_input, chunks[1]);
    
    // Render the date field
    let date_style = if app.active_field == ActiveField::Date {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let date_input = Paragraph::new(app.new_reminder_date.as_str())
        .style(date_style)
        .block(Block::default()
            .title("Date (YYYY-MM-DD)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Date {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(date_input, chunks[2]);
    
    // Render the recurrence field
    let recurrence_style = if app.active_field == ActiveField::Recurrence {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let recurrence_input = Paragraph::new(app.new_reminder_recurrence.as_str())
        .style(recurrence_style)
        .block(Block::default()
            .title("Recurrence (none/daily/weekly/monthly/yearly)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Recurrence {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }));
    f.render_widget(recurrence_input, chunks[3]);
    
    // Render the submit button
    let submit_style = if app.active_field == ActiveField::Submit {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    
    let submit_button = Paragraph::new("[ Update Reminder ]")
        .style(submit_style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if app.active_field == ActiveField::Submit {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }));
    f.render_widget(submit_button, chunks[4]);
    
    // Render error message if any
    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default()
                .borders(Borders::NONE));
        f.render_widget(error_msg, chunks[5]);
    }
    
    // Set the cursor position
    if app.active_field != ActiveField::Submit && app.input_mode == InputMode::Editing {
        let input = match app.active_field {
            ActiveField::Text => &app.new_reminder_text,
            ActiveField::Time => &app.new_reminder_time,
            ActiveField::Date => &app.new_reminder_date,
            ActiveField::Recurrence => &app.new_reminder_recurrence,
            _ => &app.input,
        };
        
        let cursor_position = match app.active_field {
            ActiveField::Text => chunks[0].x + input.len() as u16 + 1,
            ActiveField::Time => chunks[1].x + input.len() as u16 + 1,
            ActiveField::Date => chunks[2].x + input.len() as u16 + 1,
            ActiveField::Recurrence => chunks[3].x + input.len() as u16 + 1,
            _ => 0,
        };
        
        let cursor_y = match app.active_field {
            ActiveField::Text => chunks[0].y + 1,
            ActiveField::Time => chunks[1].y + 1,
            ActiveField::Date => chunks[2].y + 1,
            ActiveField::Recurrence => chunks[3].y + 1,
            _ => 0,
        };
        
        f.set_cursor_position((cursor_position, cursor_y));
    }
}

fn render_help_view(f: &mut Frame, _app: &App, area: ratatui::layout::Rect) {
    let help_text = Text::from(
        "HELP\n\n\
         q - Quit\n\
         a - Add new reminder\n\
         d - Delete selected reminder\n\
         h - Show this help\n\
         l - Show reminder list\n\
         ↑/↓ - Navigate through reminders"
    );

    let text = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL));
    
    f.render_widget(text, area);
}
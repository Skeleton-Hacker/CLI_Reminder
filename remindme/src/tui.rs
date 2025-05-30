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

#[derive(PartialEq, Eq)] // Add these derive macros
enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq, Eq)] // Add these derive macros
enum CurrentView {
    List,
    Add,
    Help,
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
            new_reminder_recurrence: String::from("none"),
            error_message: None,
        })
    }
    
    fn refresh_reminders(&mut self) -> Result<()> {
        self.reminders = self.storage.load()?;
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
        // Remove the generic type parameter
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        app.current_view = CurrentView::Add;
                        app.input_mode = InputMode::Editing;
                        app.new_reminder_text = String::new();
                        app.new_reminder_time = String::new();
                        app.new_reminder_date = String::new();
                    },
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
                    },
                    KeyCode::Enter => {
                        // Attempt to add the reminder
                        // This would need to use your CLI parsing logic
                        app.input_mode = InputMode::Normal;
                        app.current_view = CurrentView::List;
                        app.refresh_reminders()?;
                    },
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    },
                    KeyCode::Backspace => {
                        app.input.pop();
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
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to save"),
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

fn render_add_view(f: &mut Frame, _app: &App, area: ratatui::layout::Rect) {
    // Implementation would go here - creating a form for adding a new reminder
    let text = Paragraph::new("Add Reminder Form - Not fully implemented")
        .block(Block::default().title("Add Reminder").borders(Borders::ALL));
    
    f.render_widget(text, area);
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
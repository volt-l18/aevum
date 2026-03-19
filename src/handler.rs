use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::Backend, Terminal};
use std::io;

use crate::app::{App, InputMode};
use crate::ui::ui;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()>
where
    io::Error: From<<B as Backend>::Error>,
{
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            app.status_message.clear(); // Clear status message on any key press
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Right => app.move_selection(1),
                    KeyCode::Left => app.move_selection(-1),
                    KeyCode::Down => app.move_selection(7),
                    KeyCode::Up => app.move_selection(-7),
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('r') => {
                        if app.events.contains_key(&app.selected_date) {
                            app.input_mode = InputMode::Removing;
                        }
                    }
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Search;
                        app.search_query.clear();
                        app.search_results.clear();
                    }
                    KeyCode::Char('n') => app.next_month(),
                    KeyCode::Char('p') => app.prev_month(),
                    KeyCode::Char('N') => app.next_year(),
                    KeyCode::Char('P') => app.prev_year(),
                    KeyCode::Char('t') => app.go_to_today(),
                    KeyCode::Char('m') => {
                        app.input_mode = InputMode::Menu;
                        app.menu_depth = 0;
                        app.menu_list_state.select(Some(0));
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if !app.input_text.is_empty() {
                            app.events
                                .entry(app.selected_date)
                                .or_default()
                                .push(app.input_text.drain(..).collect());
                            let _ = app.save_events();
                            app.update_event_selection();
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.input_text.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_text.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Removing => match key.code {
                    KeyCode::Char('d') | KeyCode::Delete | KeyCode::Enter => {
                        app.delete_selected_event();
                        if !app.events.contains_key(&app.selected_date) {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_event(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_event(),
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Left => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Menu => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.prev_menu_item(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_menu_item(),
                    KeyCode::Enter => app.execute_menu_action(),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        if app.menu_depth > 0 {
                            app.menu_depth -= 1;
                            app.menu_list_state.select(Some(app.menu_parent_index));
                        } else {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Char('m') => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Enter => {
                        app.sync_calendar_to_search_result();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_search_result(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_search_result(),
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.search_events();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.search_events();
                    }
                    _ => {}
                }
            }
        }
    }
}

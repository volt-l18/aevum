use chrono::{Datelike, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, InputMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // The root block
    let block = Block::default()
        .title(" Aevum ('m' menu, '/' search, 't' today, 'a' add, 'q' quit) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let inner_area = block.inner(size);
    f.render_widget(block, size);

    // Split horizontally: Calendar (Left) and Sidebar (Right)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .spacing(2)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(inner_area);

    render_calendar(f, app, main_chunks[0]);
    render_sidebar(f, app, main_chunks[1]);

    if app.input_mode == InputMode::Menu {
        render_menu_modal(f, app, size);
    }
}

fn render_calendar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .spacing(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let title = Paragraph::new(app.current_month.format("%B %Y").to_string())
        .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Build the calendar table
    let header_cells = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let mut rows = vec![];
    let mut current_row = vec![];

    let first_day = app.current_month;
    let start_offset = first_day.weekday().num_days_from_sunday();

    for _ in 0..start_offset {
        current_row.push(Cell::from(""));
    }

    let mut current_date = first_day;
    let today = Local::now().date_naive();

    while current_date.month() == app.current_month.month() {
        let mut cell_style = Style::default();
        let mut label = current_date.day().to_string();

        if let Some(events) = app.events.get(&current_date) {
            if !events.is_empty() {
                label.push('*');
                cell_style = cell_style.fg(Color::Green);
            }
        }

        if current_date == app.selected_date {
            cell_style = cell_style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
        } else if current_date == today {
            cell_style = cell_style.add_modifier(Modifier::UNDERLINED);
        }

        current_row.push(Cell::from(label).style(cell_style));

        if current_row.len() == 7 {
            rows.push(Row::new(current_row.clone()).height(area.height / 10));
            current_row.clear();
        }

        current_date = current_date.succ_opt().unwrap();
    }

    if !current_row.is_empty() {
        while current_row.len() < 7 {
            current_row.push(Cell::from(""));
        }
        rows.push(Row::new(current_row).height(area.height / 10));
    }

    let table = Table::new(rows, [Constraint::Percentage(14); 7]).header(header);
    f.render_widget(table, chunks[1]);
}

fn render_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .spacing(2)
        .constraints([
            Constraint::Length(3), // Date Header
            Constraint::Min(5),    // Event List / Search Results
            Constraint::Length(3), // Input/Instruction Box
        ].as_ref())
        .split(area);

    let date_header = Paragraph::new(app.selected_date.format("%a, %b %d").to_string())
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Selected Date"));
    
    f.render_widget(date_header, chunks[0]);

    if app.input_mode == InputMode::Search {
        let results: Vec<ListItem> = app.search_results
            .iter()
            .map(|(date, event)| ListItem::new(format!("{}: {}", date.format("%m/%d"), event)))
            .collect();
        
        let search_list = List::new(results)
            .block(Block::default().borders(Borders::ALL).title("Search Results"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(search_list, chunks[1], &mut app.search_list_state);
    } else {
        let event_items: Vec<ListItem> = match app.events.get(&app.selected_date) {
            Some(list) => list.iter().map(|e| ListItem::new(format!("• {}", e))).collect(),
            None => vec![ListItem::new("No events")],
        };

        let list_title = match app.input_mode {
            InputMode::Removing => "Events (Use Arrows to select, 'd' to delete)",
            _ => "Events",
        };

        let event_list = List::new(event_items)
            .block(Block::default().borders(Borders::ALL).title(list_title))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(event_list, chunks[1], &mut app.event_list_state);
    }

    let (input_title, input_text) = if !app.status_message.is_empty() {
        ("Status", app.status_message.as_str())
    } else {
        match app.input_mode {
            InputMode::Normal => ("Instructions", "'m' menu, '/' search, 't' today"),
            InputMode::Editing => ("Add Event (Enter to save)", app.input_text.as_str()),
            InputMode::Removing => ("Remove Mode", "Select event and press 'd'"),
            InputMode::Menu => ("Menu Mode", "Select action and press Enter"),
            InputMode::Search => ("Search Mode", app.search_query.as_str()),
        }
    };

    let input = Paragraph::new(input_text)
        .style(match app.input_mode {
            InputMode::Normal => if !app.status_message.is_empty() { Style::default().fg(Color::Green) } else { Style::default() },
            InputMode::Editing | InputMode::Removing | InputMode::Menu | InputMode::Search => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title(input_title));
    
    f.render_widget(input, chunks[2]);
}

fn render_menu_modal(f: &mut Frame, app: &mut App, size: Rect) {
    let area = centered_rect(30, 30, size);
    
    let (title, items) = if app.menu_depth == 0 {
        (" Main Menu ", vec![ListItem::new("Export..."), ListItem::new("Import...")])
    } else {
        let sub_title = if app.menu_parent_index == 0 { " Export Menu " } else { " Import Menu " };
        (sub_title, vec![ListItem::new("CSV File"), ListItem::new("iCalendar (.ics)")])
    };

    let menu_list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_widget(Clear, area);
    f.render_stateful_widget(menu_list, area, &mut app.menu_list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

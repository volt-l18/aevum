use chrono::{Datelike, Duration, Local, NaiveDate};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs};

pub const EVENTS_FILE: &str = "events.json";

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Removing,
    Menu,
    Search,
}

pub struct App {
    pub current_month: NaiveDate,
    pub selected_date: NaiveDate,
    pub events: HashMap<NaiveDate, Vec<String>>,
    pub input_text: String,
    pub input_mode: InputMode,
    pub event_list_state: ListState,
    pub menu_list_state: ListState,
    pub status_message: String,
    
    // Search fields
    pub search_query: String,
    pub search_results: Vec<(NaiveDate, String)>,
    pub search_list_state: ListState,

    // Menu fields
    pub menu_depth: u8,
    pub menu_parent_index: usize,
}

impl App {
    pub fn new() -> App {
        let today = Local::now().date_naive();
        let events = App::load_events().unwrap_or_default();
        let mut app = App {
            current_month: NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap(),
            selected_date: today,
            events,
            input_text: String::new(),
            input_mode: InputMode::Normal,
            event_list_state: ListState::default(),
            menu_list_state: ListState::default(),
            status_message: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            menu_depth: 0,
            menu_parent_index: 0,
        };
        app.update_event_selection();
        app
    }

    fn load_events() -> Result<HashMap<NaiveDate, Vec<String>>, Box<dyn Error>> {
        if fs::metadata(EVENTS_FILE).is_ok() {
            let data = fs::read_to_string(EVENTS_FILE)?;
            let events = serde_json::from_str(&data)?;
            Ok(events)
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn save_events(&self) -> Result<(), Box<dyn Error>> {
        let data = serde_json::to_string_pretty(&self.events)?;
        fs::write(EVENTS_FILE, data)?;
        Ok(())
    }

    pub fn next_month(&mut self) {
        let mut year = self.current_month.year();
        let mut month = self.current_month.month() + 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date");
    }

    pub fn prev_month(&mut self) {
        let mut year = self.current_month.year();
        let mut month = self.current_month.month();
        if month == 1 {
            month = 12;
            year -= 1;
        } else {
            month -= 1;
        }
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date");
    }

    pub fn move_selection(&mut self, days: i64) {
        if let Some(new_date) = self.selected_date.checked_add_signed(Duration::days(days)) {
            self.selected_date = new_date;
            self.current_month = NaiveDate::from_ymd_opt(new_date.year(), new_date.month(), 1).expect("Invalid date");
            self.update_event_selection();
        }
    }

    pub fn update_event_selection(&mut self) {
        if let Some(events) = self.events.get(&self.selected_date) {
            if !events.is_empty() {
                self.event_list_state.select(Some(0));
            } else {
                self.event_list_state.select(None);
            }
        } else {
            self.event_list_state.select(None);
        }
    }

    pub fn next_event(&mut self) {
        if let Some(events) = self.events.get(&self.selected_date) {
            if events.is_empty() { return; }
            let i = match self.event_list_state.selected() {
                Some(i) => if i >= events.len() - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.event_list_state.select(Some(i));
        }
    }

    pub fn prev_event(&mut self) {
        if let Some(events) = self.events.get(&self.selected_date) {
            if events.is_empty() { return; }
            let i = match self.event_list_state.selected() {
                Some(i) => if i == 0 { events.len() - 1 } else { i - 1 },
                None => 0,
            };
            self.event_list_state.select(Some(i));
        }
    }

    pub fn delete_selected_event(&mut self) {
        if let Some(i) = self.event_list_state.selected() {
            if let Some(events) = self.events.get_mut(&self.selected_date) {
                if i < events.len() {
                    events.remove(i);
                    if events.is_empty() {
                        self.events.remove(&self.selected_date);
                    }
                    self.update_event_selection();
                    let _ = self.save_events();
                }
            }
        }
    }

    pub fn go_to_today(&mut self) {
        let today = Local::now().date_naive();
        self.selected_date = today;
        self.current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("Invalid date");
        self.update_event_selection();
        self.status_message = "Jumped to today".to_string();
    }

    pub fn next_year(&mut self) {
        let year = self.current_month.year() + 1;
        let month = self.current_month.month();
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date");

        let sel_year = self.selected_date.year() + 1;
        let sel_month = self.selected_date.month();
        let sel_day = self.selected_date.day();
        self.selected_date = NaiveDate::from_ymd_opt(sel_year, sel_month, sel_day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(sel_year, sel_month, 28).unwrap());
        self.update_event_selection();
    }

    pub fn prev_year(&mut self) {
        let year = self.current_month.year() - 1;
        let month = self.current_month.month();
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date");

        let sel_year = self.selected_date.year() - 1;
        let sel_month = self.selected_date.month();
        let sel_day = self.selected_date.day();
        self.selected_date = NaiveDate::from_ymd_opt(sel_year, sel_month, sel_day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(sel_year, sel_month, 28).unwrap());
        self.update_event_selection();
    }

    pub fn next_menu_item(&mut self) {
        let max = 1;
        let i = match self.menu_list_state.selected() {
            Some(i) => if i >= max { 0 } else { i + 1 },
            None => 0,
        };
        self.menu_list_state.select(Some(i));
    }

    pub fn prev_menu_item(&mut self) {
        let max = 1;
        let i = match self.menu_list_state.selected() {
            Some(i) => if i == 0 { max } else { i - 1 },
            None => 0,
        };
        self.menu_list_state.select(Some(i));
    }

    pub fn execute_menu_action(&mut self) {
        if let Some(i) = self.menu_list_state.selected() {
            if self.menu_depth == 0 {
                self.menu_parent_index = i;
                self.menu_depth = 1;
                self.menu_list_state.select(Some(0));
            } else {
                match (self.menu_parent_index, i) {
                    (0, 0) => { let _ = self.export_csv(); }
                    (0, 1) => { let _ = self.export_ics(); }
                    (1, 0) => { let _ = self.import_csv(); }
                    (1, 1) => { let _ = self.import_ics(); }
                    _ => {}
                }
                self.input_mode = InputMode::Normal;
                self.menu_depth = 0;
            }
        }
    }

    pub fn search_events(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.search_list_state.select(None);
            return;
        }

        let mut results: Vec<(NaiveDate, String)> = Vec::new();
        for (date, event_list) in &self.events {
            for event in event_list {
                if event.to_lowercase().contains(&query) {
                    results.push((*date, event.clone()));
                }
            }
        }
        results.sort_by_key(|r| r.0);
        self.search_results = results;
        if !self.search_results.is_empty() {
            self.search_list_state.select(Some(0));
        } else {
            self.search_list_state.select(None);
        }
    }

    pub fn next_search_result(&mut self) {
        if self.search_results.is_empty() { return; }
        let i = match self.search_list_state.selected() {
            Some(i) => if i >= self.search_results.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.search_list_state.select(Some(i));
        self.sync_calendar_to_search_result();
    }

    pub fn prev_search_result(&mut self) {
        if self.search_results.is_empty() { return; }
        let i = match self.search_list_state.selected() {
            Some(i) => if i == 0 { self.search_results.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.search_list_state.select(Some(i));
        self.sync_calendar_to_search_result();
    }

    pub fn sync_calendar_to_search_result(&mut self) {
        if let Some(i) = self.search_list_state.selected() {
            if let Some((date, _)) = self.search_results.get(i) {
                self.selected_date = *date;
                self.current_month = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).expect("Invalid date");
                self.update_event_selection();
            }
        }
    }

    pub fn export_csv(&mut self) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_path("events.csv")?;
        writer.write_record(&["Date", "Event"])?;
        for (date, event_list) in &self.events {
            for event in event_list {
                writer.write_record(&[date.to_string(), event.to_string()])?;
            }
        }
        writer.flush()?;
        self.status_message = "Exported to events.csv".to_string();
        Ok(())
    }

    pub fn import_csv(&mut self) -> Result<(), Box<dyn Error>> {
        if fs::metadata("events.csv").is_err() { 
            self.status_message = "events.csv not found".to_string();
            return Ok(()); 
        }
        let mut reader = csv::Reader::from_path("events.csv")?;
        let mut count = 0;
        for result in reader.records() {
            let record = result?;
            if record.len() >= 2 {
                if let Ok(date) = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d") {
                    self.events.entry(date).or_default().push(record[1].to_string());
                    count += 1;
                }
            }
        }
        self.save_events()?;
        self.update_event_selection();
        self.status_message = format!("Imported {} events from events.csv", count);
        Ok(())
    }

    pub fn export_ics(&mut self) -> Result<(), Box<dyn Error>> {
        use icalendar::{Calendar, Component, Event, EventLike};
        let mut calendar = Calendar::new();
        for (date, event_list) in &self.events {
            for event_str in event_list {
                let event = Event::new()
                    .summary(event_str)
                    .all_day(*date)
                    .done();
                calendar.push(event);
            }
        }
        fs::write("events.ics", calendar.to_string())?;
        self.status_message = "Exported to events.ics".to_string();
        Ok(())
    }

    pub fn import_ics(&mut self) -> Result<(), Box<dyn Error>> {
        use icalendar::{Calendar, Component, CalendarDateTime, DatePerhapsTime};
        if fs::metadata("events.ics").is_err() { 
            self.status_message = "events.ics not found".to_string();
            return Ok(()); 
        }
        let data = fs::read_to_string("events.ics")?;
        let calendar: Calendar = data.parse()?;
        let mut count = 0;
        for component in calendar.components {
            if let Some(event) = component.as_event() {
                let summary = event.get_summary().unwrap_or("No summary").to_string();
                if let Some(date_prop) = event.get_start() {
                    let naive_date = match date_prop {
                        DatePerhapsTime::Date(d) => d,
                        DatePerhapsTime::DateTime(dt) => match dt {
                            CalendarDateTime::Floating(f) => f.date(),
                            CalendarDateTime::Utc(u) => u.date_naive(),
                            CalendarDateTime::WithTimezone { date_time, tzid: _ } => date_time.date(),
                        },
                    };
                    self.events.entry(naive_date).or_default().push(summary);
                    count += 1;
                }
            }
        }
        self.save_events()?;
        self.update_event_selection();
        self.status_message = format!("Imported {} events from events.ics", count);
        Ok(())
    }
}

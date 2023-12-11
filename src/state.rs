use std::{io::BufReader, collections::BTreeMap};

use chrono::{DateTime, Datelike, Local, Utc};
use icalendar::{Calendar, Event, CalendarComponent, Component, DatePerhapsTime};

use crate::config::Config;

pub struct CalendarState {
    pub view: CalendarView,
    pub events: BTreeMap<DateTime<Utc>, Event>,
}

impl CalendarState {
    pub fn new(config: Config) -> Self {
        let now = Local::now();

        let mut events = BTreeMap::new();

        walkdir::WalkDir::new(config.calendar_dir.canonicalize().unwrap())
            .into_iter()
            .flatten()
            .for_each(|file| {
                if file.file_name().to_string_lossy().ends_with(".ics") {
                    let Ok(file_content) = std::fs::read_to_string(file.path()) else {
                        return;
                    };

                    let Ok(calendar) = file_content.parse::<Calendar>() else {
                        return;
                    };

                    for ele in calendar.components {
                        if let CalendarComponent::Event(event) = ele {
                            let Some(s) = utc_start(event.get_start()) else { continue };

                            events.insert(s, event);
                        }
                    }
                }
            });

        let now_month = now.with_day(1).unwrap();

        Self {
            view: CalendarView::Month(now_month),
            events,
        }
    }
}

fn utc_start(start: Option<DatePerhapsTime>) -> Option<DateTime<Utc>> {
    let start = start?;

    match start {
        DatePerhapsTime::DateTime(dt) => dt.try_into_utc(),
        DatePerhapsTime::Date(date) => Some(date.and_hms_opt(0, 0, 0)?.and_utc()),
    }
}

pub enum CalendarView {
    Month(DateTime<Local>),
    Week(DateTime<Local>),
}

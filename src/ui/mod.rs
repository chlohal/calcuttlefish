use std::{collections::BTreeMap, iter::once};

use chrono::{format::format, DateTime, Datelike, Days, Local, Timelike, Utc, Weekday};
use icalendar::{Component, DatePerhapsTime, Event};
use itertools::Itertools;
use ratatui::{
    prelude::Constraint,
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

use crate::state::{CalendarState, CalendarView};

pub fn ui(frame: &mut Frame, state: &CalendarState) {
    match state.view {
        CalendarView::Month(time) => month_grid(frame, time, &state.events),
        CalendarView::Week(_) => todo!(),
    }
}

fn month_grid(frame: &mut Frame, time: DateTime<Local>, events: &BTreeMap<DateTime<Utc>, Event>) {
    let mut time = time;

    //Reset time to start of the week
    while time.weekday() != Weekday::Mon {
        time = time.checked_sub_days(Days::new(1)).unwrap();
    }
    time = time
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    let area = frame.size();
    let cell_height = area.height / 5;
    let cell_width = area.width / 7;
    let w_constraints = &(vec![Constraint::Length(cell_width); 7])[..];

    let cell_width = cell_width as usize;

    let current_day = Local::now().date_naive();

    let table = Table::new((0..5).map(|week| {
        Row::new((0..7).map(|day| {
            let day = time.clone() + Days::new(week * 7 + day);

            let day_start = day.with_timezone(&Utc);
            let day_end = day.with_timezone(&Utc) + Days::new(1);

            Cell::from(Text::from(
                once(Line::from({
                    let t = Span::raw(day.format("%b %d").to_string());

                    if day.date_naive() == current_day {
                        t.green().bold()
                    } else {
                        t.blue().bold()
                    }
                }))
                .chain(more_line(
                    (cell_height as usize) - 1,
                    events
                        .range(day_start..=day_end)
                        .map(|(_, e)| render_event(e, cell_width)),
                ))
                .collect::<Vec<Line>>(),
            ))
        }))
        .height(cell_height)
    }))
    .widths(w_constraints);

    let mut state = TableState::default();

    frame.render_stateful_widget(table, area, &mut state);
}

fn more_line<'a>(
    max: usize,
    events: impl Iterator<Item = Line<'a>>,
) -> impl Iterator<Item = Line<'a>> {
    let mut events: Vec<_> = events.collect();

    if events.len() > max {
        let more = events.len() - max;

        while events.len() > (max - 1) {
            events.pop();
        }
        events.push(Line::from(Span::raw(format!("{more} more")).yellow()));
    }
    events.into_iter()
}

fn render_event(event: &Event, width: usize) -> Line<'static> {
    let summary = event.get_summary().unwrap_or_default();

    if summary.len() > width {
        Line::raw(format!("{}…", &summary[0..(width - 1)]))
    } else {
        Line::raw(format!("{}", summary))
    }
}

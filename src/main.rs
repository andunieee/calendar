use std::cell::Cell;
use std::rc::Rc;

use chrono::{Datelike, NaiveDate};
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;

    let current_year: i32 = chrono::Local::now().year();
    let current_month: u32 = chrono::Local::now().month();

    let year = Rc::new(Cell::new(current_year));
    let month = Rc::new(Cell::new(current_month));

    update_calendar(&app, current_year, current_month);

    let app_weak = app.as_weak();

    // Month navigation callbacks
    app.on_prev_month({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move || {
            let (new_year, new_month) = if month.get() == 1 {
                (year.get() - 1, 12)
            } else {
                (year.get(), month.get() - 1)
            };
            year.set(new_year);
            month.set(new_month);
            if let Some(app) = app_weak.upgrade() {
                update_calendar(&app, new_year, new_month);
            }
        }
    });

    app.on_next_month({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move || {
            let (new_year, new_month) = if month.get() == 12 {
                (year.get() + 1, 1)
            } else {
                (year.get(), month.get() + 1)
            };
            year.set(new_year);
            month.set(new_month);
            if let Some(app) = app_weak.upgrade() {
                update_calendar(&app, new_year, new_month);
            }
        }
    });

    // Year navigation callbacks
    app.on_prev_year({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move || {
            let new_year = year.get() - 1;
            year.set(new_year);
            if let Some(app) = app_weak.upgrade() {
                update_calendar(&app, new_year, month.get());
            }
        }
    });

    app.on_next_year({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move || {
            let new_year = year.get() + 1;
            year.set(new_year);
            if let Some(app) = app_weak.upgrade() {
                update_calendar(&app, new_year, month.get());
            }
        }
    });

    // Year changed from text input
    app.on_year_changed({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move |text: SharedString| {
            if let Ok(new_year) = text.parse::<i32>() {
                if new_year > 0 && new_year < 10000 {
                    year.set(new_year);
                    if let Some(app) = app_weak.upgrade() {
                        update_calendar(&app, new_year, month.get());
                    }
                }
            }
        }
    });

    // Month changed from text input
    app.on_month_changed({
        let app_weak = app_weak.clone();
        let year = year.clone();
        let month = month.clone();
        move |text: SharedString| {
            let month_names = [
                "january",
                "february",
                "march",
                "april",
                "may",
                "june",
                "july",
                "august",
                "september",
                "october",
                "november",
                "december",
            ];
            let text_lower = text.to_lowercase();

            // Try to parse as number first
            if let Ok(new_month) = text.parse::<u32>() {
                if new_month >= 1 && new_month <= 12 {
                    month.set(new_month);
                    if let Some(app) = app_weak.upgrade() {
                        update_calendar(&app, year.get(), new_month);
                    }
                    return;
                }
            }

            // Try to parse as month name
            if let Some(idx) = month_names
                .iter()
                .position(|&name| name == text_lower.as_str())
            {
                let new_month = (idx + 1) as u32;
                month.set(new_month);
                if let Some(app) = app_weak.upgrade() {
                    update_calendar(&app, year.get(), new_month);
                }
            }
        }
    });

    app.run()
}

fn update_calendar(app: &MainWindow, year: i32, month: u32) {
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    app.set_year_label(SharedString::from(year.to_string()));
    app.set_month_label(SharedString::from(month_names[(month - 1) as usize]));

    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let days_in_month = days_in_month(year, month);
    let start_weekday = first_day.weekday().num_days_from_sunday() as usize;

    let mut days: Vec<SharedString> = vec![SharedString::from(""); 49];

    for day in 1..=days_in_month {
        let idx = start_weekday + (day as usize) - 1;
        if idx < 42 {
            days[idx] = SharedString::from(day.to_string());
        }
    }

    let model = ModelRc::new(VecModel::from(days));
    app.set_days(model);

    // Calculate today's index if we're viewing the current month
    let today = chrono::Local::now();
    if year == today.year() && month == today.month() {
        let today_idx = start_weekday + (today.day() as usize) - 1;
        app.set_today_index(today_idx as i32);
    } else {
        app.set_today_index(-1);
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

slint::include_modules!();

use chrono::{DateTime, Datelike, Local, NaiveDate};

fn main() {
    let main_window = MainWindow::new().unwrap();
    let current_date = Local::now();

    // Set initial calendar
    update_calendar(&main_window, current_date);

    let main_window_weak = main_window.as_weak();
    let mut prev_date = current_date;
    main_window.on_prev_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        prev_date = prev_date.with_day(1).unwrap() - chrono::Duration::days(1);
        prev_date = prev_date.with_day(1).unwrap();
        update_calendar(&window, prev_date);
    });

    let main_window_weak = main_window.as_weak();
    let mut next_date = current_date;
    main_window.on_next_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        next_date = next_date.with_day(1).unwrap() + chrono::Duration::days(32);
        next_date = next_date.with_day(1).unwrap();
        update_calendar(&window, next_date);
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_go_today(move || {
        let window = main_window_weak.upgrade().unwrap();
        let today = Local::now();
        update_calendar(&window, today);
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_day_selected(move |_day_idx: i32| {
        let _window = main_window_weak.upgrade().unwrap();
        // The selected-date is already updated in the UI
    });

    main_window.run().unwrap();
}

fn update_calendar(window: &MainWindow, date: DateTime<Local>) {
    let year = date.year();
    let month = date.month();

    // Format month and year display
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
    let month_name = month_names[(month - 1) as usize];
    window.set_current_month(format!("{} {}", month_name, year).into());

    // Generate calendar grid
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday = first_day.weekday();
    let first_weekday = weekday.number_from_sunday() as i32 - 1; // 0=Sunday

    // Get the number of days in the month
    let days_in_month = if month == 12 {
        (NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
            - NaiveDate::from_ymd_opt(year, 12, 1).unwrap())
        .num_days() as i32
    } else {
        (NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
            - NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as i32
    };

    // Create days array
    let mut days: Vec<i32> = vec![0; first_weekday as usize];
    for day in 1..=days_in_month {
        days.push(day);
    }
    // Pad to complete the grid (42 cells = 6 weeks * 7 days)
    while days.len() < 42 {
        days.push(0);
    }

    // Set the days property
    window.set_days(std::rc::Rc::new(slint::VecModel::from(days)).into());
}

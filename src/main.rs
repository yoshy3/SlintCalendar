slint::include_modules!();

use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone};
#[cfg(windows)]
use winreg::enums::HKEY_CURRENT_USER;
#[cfg(windows)]
use winreg::RegKey;

fn main() {
    // detect system dark mode on Windows
    let dark_mode = system_dark_mode();

    let main_window = MainWindow::new().unwrap();
    main_window.set_darkMode(dark_mode);
    // Provide an initial scale and register a handler for width changes from UI.
    main_window.set_scale(1.0);
    let current_date = Local::now();

    // Set initial calendar
    update_calendar(&main_window, current_date);

    // Scale is now updated by the UI (Timer) directly; no width callback needed.

    // Native polling threads removed — UI performs scale updates directly now.

    let main_window_weak = main_window.as_weak();
    main_window.on_prev_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        // parse current displayed month/year in format yyyy年mm月 (or yyyy年m月)
        let cur = window.get_current_month();
        let digits: String = cur.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 5 {
            let year: i32 = digits[..4].parse().unwrap_or_else(|_| Local::now().year());
            let month: u32 = digits[4..].parse().unwrap_or_else(|_| Local::now().month());
            // compute previous month
            let (ny, nm) = if month == 1 {
                (year - 1, 12)
            } else {
                (year, month - 1)
            };
            let naive = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
            let naive_dt = naive.and_hms_opt(0, 0, 0).expect("invalid time");
            let dt = match Local.from_local_datetime(&naive_dt) {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => Local::now(),
            };
            update_calendar(&window, dt);
        }
    });

    let main_window_weak = main_window.as_weak();
    main_window.on_next_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        // parse current displayed month/year in format yyyy年mm月 (or yyyy年m月)
        let cur = window.get_current_month();
        let digits: String = cur.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 5 {
            let year: i32 = digits[..4].parse().unwrap_or_else(|_| Local::now().year());
            let month: u32 = digits[4..].parse().unwrap_or_else(|_| Local::now().month());
            // compute next month
            let (ny, nm) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            let naive = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
            let naive_dt = naive.and_hms_opt(0, 0, 0).expect("invalid time");
            let dt = match Local.from_local_datetime(&naive_dt) {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => Local::now(),
            };
            update_calendar(&window, dt);
        }
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

    // Format month/year as Japanese: yyyy年mm月 (zero-padded month)
    window.set_current_month(format!("{:04}年{:02}月", year, month).into());

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

    // Determine number of week rows needed (4..=6)
    let total_slots = first_weekday as i32 + days_in_month;
    let num_rows = ((total_slots + 6) / 7) as i32; // ceiling division

    // Pad to complete the grid (num_rows * 7 cells)
    while days.len() < (num_rows as usize * 7) {
        days.push(0);
    }

    // Set the days and number of rows properties
    window.set_days(std::rc::Rc::new(slint::VecModel::from(days)).into());
    window.set_num_rows(num_rows);
}

// Return true if system prefers dark mode (Windows registry). Defaults to false.
#[cfg(windows)]
fn system_dark_mode() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
    {
        if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return val == 0; // 0 = dark, 1 = light
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn system_dark_mode() -> bool {
    // On macOS, use the `defaults` command to read AppleInterfaceStyle.
    // If it returns "Dark", we consider system in dark mode.
    use std::process::Command;
    if let Ok(output) = Command::new("defaults")
        .arg("read")
        .arg("-g")
        .arg("AppleInterfaceStyle")
        .output()
    {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            return out.trim().eq_ignore_ascii_case("Dark");
        }
    }
    false
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn system_dark_mode() -> bool {
    false
}

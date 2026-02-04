#![cfg_attr(windows, windows_subsystem = "windows")]

slint::include_modules!();
use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone};
use jpholiday::jpholiday::JPHoliday;
use serde::{Deserialize, Serialize};
use slint::PhysicalPosition;
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::HKEY_CURRENT_USER;

fn main() {
    // Windows のシステムダークモードを検出します
    let dark_mode = system_dark_mode();

    // MainWindow インスタンスを生成します
    let main_window: MainWindow = MainWindow::new().unwrap();
    main_window.set_darkMode(dark_mode);

    let mut win_x: Option<i32> = None;
    let mut win_y: Option<i32> = None;
    let last_window_metrics = std::rc::Rc::new(RefCell::new(None::<WindowMetrics>));
    let last_today = std::rc::Rc::new(RefCell::new(Local::now().date_naive()));

    // 保存されたウィンドウメトリクスを読み込み、UIに渡してサイズと位置を復元します
    if let Some(cfg) = load_window_metrics() {
        main_window.set_savedWidthPx(cfg.width);
        main_window.set_savedHeightPx(cfg.height);
        win_x = cfg.x;
        win_y = cfg.y;
    }

    let current_date = Local::now();

    // 初期カレンダーを設定します
    update_calendar(&main_window, current_date);

    // 前月ボタンのクリックイベントを処理します
    let main_window_weak = main_window.as_weak();
    main_window.on_prev_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        // 現在表示されている年月 (yyyy年mm月 または yyyy年m月) をパースします
        let cur = window.get_current_month();
        let digits: String = cur.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 5 {
            let year: i32 = digits[..4].parse().unwrap_or_else(|_| Local::now().year());
            let month: u32 = digits[4..].parse().unwrap_or_else(|_| Local::now().month());
            // 前月を計算します
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

    // 次月ボタンのクリックイベントを処理します
    let main_window_weak = main_window.as_weak();
    main_window.on_next_month(move || {
        let window = main_window_weak.upgrade().unwrap();
        // 現在表示されている年月 (yyyy年mm月 または yyyy年m月) をパースします
        let cur = window.get_current_month();
        let digits: String = cur.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 5 {
            let year: i32 = digits[..4].parse().unwrap_or_else(|_| Local::now().year());
            let month: u32 = digits[4..].parse().unwrap_or_else(|_| Local::now().month());
            // 次月を計算します
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

    // 「今日」ボタンのクリックイベントを処理します
    let main_window_weak = main_window.as_weak();
    main_window.on_go_today(move || {
        let window = main_window_weak.upgrade().unwrap();
        let today = Local::now();
        update_calendar(&window, today);
    });

    // 日付選択イベントを処理します
    let main_window_weak = main_window.as_weak();
    main_window.on_day_selected(move |_day_idx: i32| {
        let _window = main_window_weak.upgrade().unwrap();
        // 選択された日付はすでにUI側で更新されています
    });

    // ウィンドウのメトリクスが変更されたときに保存します
    let main_window_weak = main_window.as_weak();
    let metrics_storage = last_window_metrics.clone();
    let last_today = last_today.clone();
    main_window.on_tick(move || {
        let window = main_window_weak.upgrade().unwrap();
        let size = window.get_window_size();

        let now = Local::now();
        let today = now.date_naive();
        let mut last = last_today.borrow_mut();
        if *last != today {
            *last = today;
            window.set_today_year(now.year());
            window.set_today_month(now.month() as i32);
            window.set_today_day(now.day() as i32);

            // 表示中の年月でカレンダーを再描画します
            let display_year = window.get_display_year();
            let display_month = window.get_display_month() as u32;
            if let Some(naive) = NaiveDate::from_ymd_opt(display_year, display_month, 1) {
                let naive_dt = naive.and_hms_opt(0, 0, 0).expect("invalid time");
                let dt = match Local.from_local_datetime(&naive_dt) {
                    chrono::LocalResult::Single(dt) => dt,
                    chrono::LocalResult::Ambiguous(dt, _) => dt,
                    chrono::LocalResult::None => Local::now(),
                };
                update_calendar(&window, dt);
            }
        }
        // 標準出力に出力します
        println!(
            "Window Rect Changed: ({},{}) {}x{}",
            window.window().position().x,
            window.window().position().y,
            size.width,
            size.height
        );

        let cfg = WindowMetrics {
            width: size.width,
            height: size.height,
            x: Some(window.window().position().x),
            y: Some(window.window().position().y),
        };

        let changed = {
            let stored = metrics_storage.borrow();
            stored.as_ref().map(|prev| prev != &cfg).unwrap_or(true)
        };

        if changed {
            if let Err(e) = save_window_metrics(&cfg) {
                eprintln!("Failed to save window metrics: {}", e);
            }
            *metrics_storage.borrow_mut() = Some(cfg.clone());
        }
    });

    let main_window_weak = main_window.as_weak();
    {
        let window = main_window_weak.upgrade().unwrap();

        if let Some(_x) = win_x
            && let Some(_y) = win_y
        {
            // TODO: Slint の適切なAPIを使用してウィンドウ位置を設定します
            // window.window().set_position(slint::api::WindowPosition { x, y });
            window.window().set_position(PhysicalPosition::new(_x, _y));
        }
    }

    main_window.run().unwrap();
}

fn update_calendar(window: &MainWindow, date: DateTime<Local>) {
    let year = date.year();
    let month = date.month();

    // 今日の日付を設定します
    let now = Local::now();
    window.set_today_year(now.year());
    window.set_today_month(now.month() as i32);
    window.set_today_day(now.day() as i32);

    // 表示中の年月を設定します
    window.set_display_year(year);
    window.set_display_month(month as i32);

    // 年月を日本語形式 (yyyy年mm月) にフォーマットします
    window.set_current_month(format!("{:04}年{:02}月", year, month).into());

    // カレンダーグリッドを生成します
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday = first_day.weekday();
    let first_weekday = weekday.number_from_sunday() as i32 - 1; // 0=日曜日

    // 月の日数を取得します
    let days_in_month = if month == 12 {
        (NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
            - NaiveDate::from_ymd_opt(year, 12, 1).unwrap())
        .num_days() as i32
    } else {
        (NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
            - NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as i32
    };

    // 日付配列を作成します
    let mut days: Vec<i32> = vec![0; first_weekday as usize];
    for day in 1..=days_in_month {
        days.push(day);
    }

    // 必要な週の行数 (4から6) を決定します
    let total_slots = first_weekday as i32 + days_in_month;
    let num_rows = ((total_slots + 6) / 7) as i32; // ceiling division

    // グリッドを完成させるためにパディングします (num_rows * 7 セル)
    while days.len() < (num_rows as usize * 7) {
        days.push(0);
    }

    // 祝日名リストを作成します (days グリッドと同じ長さ)
    let jpholiday = JPHoliday::new();
    let mut holiday_names: Vec<slint::SharedString> = vec![slint::SharedString::new(); days.len()];
    for (idx, day) in days.iter().enumerate() {
        if *day > 0 {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, *day as u32) {
                if let Some(name) = jpholiday.is_holiday_name(&date) {
                    holiday_names[idx] = name.into();
                }
            }
        }
    }

    // 日付と行数のプロパティを設定します
    window.set_days(std::rc::Rc::new(slint::VecModel::from(days)).into());
    window.set_holiday_names(std::rc::Rc::new(slint::VecModel::from(holiday_names)).into());
    window.set_num_rows(num_rows);
}

// Windows のレジストリをチェックして、システムがダークモードを優先するかどうかを返します。デフォルトは false です。
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
    // macOS では `defaults` コマンドを使用して AppleInterfaceStyle を読み取ります。
    // "Dark" が返された場合、システムはダークモードであると見なします。
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
    true
}

// ウィンドウのメトリクスを保持するための構造体
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct WindowMetrics {
    width: f32,
    height: f32,
    x: Option<i32>,
    y: Option<i32>,
}

// 設定ファイルのパスを返します
fn config_file_path() -> Option<PathBuf> {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("slint_calendar");
        let _ = fs::create_dir_all(&dir);
        dir.push("window_metrics.json");
        return Some(dir);
    }
    None
}

// 設定ファイルからウィンドウメトリクスを読み込みます
fn load_window_metrics() -> Option<WindowMetrics> {
    let path = config_file_path()?;
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<WindowMetrics>(&data) {
                return Some(cfg);
            }
        }
    }
    None
}

// ウィンドウメトリクスを設定ファイルに保存します
fn save_window_metrics(cfg: &WindowMetrics) -> Result<(), std::io::Error> {
    if let Some(path) = config_file_path() {
        let tmp = path.with_extension("json.tmp");
        let serialized = serde_json::to_string_pretty(cfg).unwrap_or_default();
        let mut f = fs::File::create(&tmp)?;
        f.write_all(serialized.as_bytes())?;
        fs::rename(tmp, path)?;
    }
    Ok(())
}

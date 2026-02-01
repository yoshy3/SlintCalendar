fn main() {
    // ui/calendar.slint をコンパイルします
    slint_build::compile("ui/calendar.slint").unwrap();

    // Windows アイコン設定
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}

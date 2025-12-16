fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/timerIcon.ico");
        res.set("FileDescription", "LightTimer");
        res.set("ProductName", "LightTimer");
        res.set("CompanyName", "LightTimer");
        res.compile().unwrap();
    }
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();

        res.set_icon("overlay/assets/appicon.ico");

        res.set("ProductName", "NowPlaying");
        res.set("FileDescription", "NowPlaying - Stream Music Overlay");
        res.set("CompanyName", "NowPlaying");
        res.set("LegalCopyright", "Copyright © 2026 NowPlaying");

        res.set("FileVersion", "0.4.0.0");
        res.set("ProductVersion", "0.4.0.0");

        res.compile().expect("Failed to compile Windows resources");
    }
}

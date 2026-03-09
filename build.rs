fn main() {
    // Windows: embed icon resource in .exe
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to set Windows icon: {}", e);
        }
    }
}

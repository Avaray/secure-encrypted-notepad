fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        res.set("FileDescription", "Secure Encrypted Notepad");
        res.set("ProductName", "Secure Encrypted Notepad");
        res.set("InternalName", "sen");
        res.set("CompanyName", "Dawid Wasowski");
        res.set("LegalCopyright", "");
        res.compile().expect("Failed to compile Windows resources");
    }
}

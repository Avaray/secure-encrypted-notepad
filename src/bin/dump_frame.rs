fn main() {
    let ctx = eframe::egui::Context::default();
    ctx.style_mut(|style| style.spacing.item_spacing.x = 0.0);
    println!("style_mut works!");
}

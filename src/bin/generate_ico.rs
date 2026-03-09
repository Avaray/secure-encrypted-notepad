use std::io::Write;

/// One-off binary to generate `assets/app_icon.ico` from `assets/app_icon.svg`.
/// Run: `cargo run --bin generate_ico`
fn main() {
    let svg_data = std::fs::read("assets/app_icon.svg").expect("Cannot read assets/app_icon.svg");

    // Sizes to include in the ICO (standard Windows icon sizes)
    let sizes: &[u32] = &[16, 32, 48, 256];

    let mut png_entries: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in sizes {
        let png_bytes = render_svg_to_png(&svg_data, size, size);
        png_entries.push((size, png_bytes));
    }

    let ico_bytes = build_ico(&png_entries);
    std::fs::write("assets/app_icon.ico", &ico_bytes).expect("Cannot write assets/app_icon.ico");

    println!("✅ Generated assets/app_icon.ico with {} sizes: {:?}", sizes.len(), sizes);
}

fn render_svg_to_png(svg_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &opt).expect("Failed to parse SVG");

    let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("Failed to create pixmap");

    let tree_size = tree.size();
    let transform = tiny_skia::Transform::from_scale(
        width as f32 / tree_size.width(),
        height as f32 / tree_size.height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().expect("Failed to encode PNG")
}

fn build_ico(entries: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = entries.len() as u16;
    let mut buf: Vec<u8> = Vec::new();

    // ICONDIR header (6 bytes)
    buf.write_all(&0u16.to_le_bytes()).unwrap(); // reserved
    buf.write_all(&1u16.to_le_bytes()).unwrap(); // type = 1 (ICO)
    buf.write_all(&count.to_le_bytes()).unwrap(); // number of images

    // Calculate data offset: header(6) + entries(16 each)
    let mut data_offset: u32 = 6 + (count as u32) * 16;

    // ICONDIRENTRY (16 bytes each)
    for (size, png_data) in entries {
        let w = if *size >= 256 { 0u8 } else { *size as u8 };
        let h = w;
        buf.push(w);                                    // width (0 = 256)
        buf.push(h);                                    // height (0 = 256)
        buf.push(0);                                    // color palette
        buf.push(0);                                    // reserved
        buf.write_all(&1u16.to_le_bytes()).unwrap();    // color planes
        buf.write_all(&32u16.to_le_bytes()).unwrap();   // bits per pixel
        buf.write_all(&(png_data.len() as u32).to_le_bytes()).unwrap(); // size of data
        buf.write_all(&data_offset.to_le_bytes()).unwrap(); // offset to data
        data_offset += png_data.len() as u32;
    }

    // Image data (raw PNG)
    for (_, png_data) in entries {
        buf.write_all(png_data).unwrap();
    }

    buf
}

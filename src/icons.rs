use egui::{ColorImage, Context, TextureHandle};

pub struct Icons {
    pub new_doc: TextureHandle,
    pub open: TextureHandle,
    pub open_folder: TextureHandle,
    pub save: TextureHandle,
    pub save_as: TextureHandle,
    pub key: TextureHandle,
    pub generate: TextureHandle,
    pub settings: TextureHandle,
    pub history: TextureHandle,
    pub debug: TextureHandle,
    pub file_tree: TextureHandle,
    pub theme: TextureHandle,

    pub export: TextureHandle,
    pub rotate: TextureHandle,
    pub batch_convert: TextureHandle,
}

impl Icons {
    pub fn load(ctx: &Context) -> Self {
        Self {
            new_doc: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsAddDiamondRounded.svg"),
                "icon_new",
            ),
            open: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsFolderOpen.svg"),
                "icon_open",
            ),
            open_folder: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsLightFolderCopyRounded.svg"),
                "icon_folder",
            ),
            save: Self::load_svg(
                ctx,
                include_bytes!("../assets/IcSharpSave.svg"),
                "icon_save",
            ),
            save_as: Self::load_svg(
                ctx,
                include_bytes!("../assets/MdiContentSaveSettings.svg"),
                "icon_save_as",
            ),
            key: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsKeyVertical.svg"),
                "icon_key",
            ),
            generate: Self::load_svg(
                ctx,
                include_bytes!("../assets/FluentMagicWand20Regular.svg"),
                "icon_generate",
            ),
            settings: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsSettingsRounded.svg"),
                "icon_settings",
            ),
            history: Self::load_svg(
                ctx,
                include_bytes!("../assets/UimHistory.svg"),
                "icon_history",
            ),
            debug: Self::load_svg(
                ctx,
                include_bytes!("../assets/StreamlinePlumpBugSolid.svg"),
                "icon_debug",
            ),
            file_tree: Self::load_svg(
                ctx,
                include_bytes!("../assets/FluentTreeEvergreen20Filled.svg"),
                "icon_tree",
            ),
            theme: Self::load_svg(
                ctx,
                include_bytes!("../assets/StreamlineUltimateColorPalette.svg"),
                "icon_theme",
            ),
            export: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsExportNotes.svg"),
                "icon_export",
            ),
            rotate: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsKeyRotation.svg"),
                "icon_rotate",
            ),
            batch_convert: Self::load_svg(
                ctx,
                include_bytes!("../assets/MaterialSymbolsBatchConvert.svg"),
                "icon_batch_convert",
            ),
        }
    }

    fn load_svg(ctx: &Context, svg_bytes: &[u8], name: &str) -> TextureHandle {
        let image = Self::render_svg(svg_bytes, 128, 128); // Increased from 32x32 for sharp large icons
        ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
    }

    fn render_svg(svg_data: &[u8], width: u32, height: u32) -> ColorImage {
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_data, &opt).expect("Failed to parse SVG");

        let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("Failed to create pixmap");

        // Użyj metody size() zamiast pola
        let tree_size = tree.size();

        // Oblicz transform do skalowania SVG do docelowego rozmiaru
        let transform = tiny_skia::Transform::from_scale(
            width as f32 / tree_size.width(),
            height as f32 / tree_size.height(),
        );

        // Renderuj SVG
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Konwertuj pixmap do ColorImage dla egui
        let pixels: Vec<egui::Color32> = pixmap
            .data()
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        ColorImage {
            size: [width as usize, height as usize],
            pixels,
            source_size: egui::Vec2::new(width as f32, height as f32),
        }
    }

    /// Load application icon as IconData for window/taskbar (all platforms)
    pub fn load_app_icon() -> egui::IconData {
        let svg_data = include_bytes!("../assets/app_icon.svg");
        let size: u32 = 256;

        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_data, &opt).expect("Failed to parse app icon SVG");

        let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("Failed to create pixmap");

        let tree_size = tree.size();
        let transform = tiny_skia::Transform::from_scale(
            size as f32 / tree_size.width(),
            size as f32 / tree_size.height(),
        );

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        egui::IconData {
            rgba: pixmap.data().to_vec(),
            width: size,
            height: size,
        }
    }
}

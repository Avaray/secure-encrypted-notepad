use egui::{ColorImage, Context, TextureHandle};

#[allow(dead_code)]
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
    pub reset: TextureHandle,

    pub export: TextureHandle,
    pub rotate: TextureHandle,
    pub batch_convert: TextureHandle,
    pub zen: TextureHandle,
    pub spon_github: TextureHandle,
    pub spon_patreon: TextureHandle,
    pub spon_bmc: TextureHandle,
    pub spon_kofi: TextureHandle,
    pub spon_oc: TextureHandle, // Use CibKoFi as temporary if Open Collective is missing or intended
    pub flag_en: TextureHandle,
    pub flag_pl: TextureHandle,
    pub flag_de: TextureHandle,
    pub flag_ua: TextureHandle,
    pub flag_cz: TextureHandle,
    pub flag_sk: TextureHandle,
    pub flag_fr: TextureHandle,
    pub flag_es: TextureHandle,
    pub flag_br: TextureHandle,
    pub flag_jp: TextureHandle,
    pub flag_cn: TextureHandle,
    pub flag_nl: TextureHandle,
    pub flag_ru: TextureHandle,
    pub flag_it: TextureHandle,
    pub flag_ar: TextureHandle,
    pub folder_filled: TextureHandle,
    pub folder_open: TextureHandle,
    pub unknown_file: TextureHandle,
    pub status_dot: TextureHandle,
    pub locked_file: TextureHandle,
    pub asterisk_file: TextureHandle,
    pub close: TextureHandle,
}

impl Icons {
    pub fn load(ctx: &Context) -> Self {
        Self {
            new_doc: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsAddDiamondRounded.svg"),
                "icon_new",
            ),
            open: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsFolderOpen.svg"),
                "icon_open",
            ),
            open_folder: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsLightFolderCopyRounded.svg"),
                "icon_folder",
            ),
            save: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/IcSharpSave.svg"),
                "icon_save",
            ),
            save_as: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MdiContentSaveSettings.svg"),
                "icon_save_as",
            ),
            key: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsKeyVertical.svg"),
                "icon_key",
            ),
            generate: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/FluentMagicWand20Regular.svg"),
                "icon_generate",
            ),
            settings: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsSettingsRounded.svg"),
                "icon_settings",
            ),
            history: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/UimHistory.svg"),
                "icon_history",
            ),
            debug: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/AntDesignBugFilled.svg"),
                "icon_debug",
            ),
            file_tree: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/PhTreeViewFill.svg"),
                "icon_tree",
            ),
            folder_filled: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/QlementineIconsFolderFilled16.svg"),
                "icon_folder_filled",
            ),
            folder_open: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/QlementineIconsFolderOpen24.svg"),
                "icon_folder_open",
            ),
            unknown_file: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MynauiQuestionHexagonSolid.svg"),
                "icon_unknown",
            ),
            status_dot: Self::load_svg(ctx, include_bytes!("../../sen-desktop/assets/ui/Dot.svg"), "icon_dot"),
            locked_file: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MynauiLockHexagonSolid.svg"),
                "icon_locked",
            ),
            asterisk_file: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MynauiAsteriskHexagonSolid.svg"),
                "icon_asterisk",
            ),
            theme: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/FamiconsColorFilterSharp.svg"),
                "icon_theme",
            ),
            export: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsExportNotes.svg"),
                "icon_export",
            ),
            rotate: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/LucideLabCoinsExchange.svg"),
                "icon_rotate",
            ),
            batch_convert: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/MaterialSymbolsBatchConvert.svg"),
                "icon_batch_convert",
            ),
            zen: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/IxEyeFocus.svg"),
                "icon_zen",
            ),
            spon_github: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/sponsor/EntypoSocialGithub.svg"),
                "spon_github",
            ),
            spon_patreon: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/sponsor/SimpleIconsPatreon.svg"),
                "spon_patreon",
            ),
            spon_bmc: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/sponsor/SimpleIconsBuymeacoffee.svg"),
                "spon_bmc",
            ),
            spon_kofi: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/sponsor/SimpleIconsKofi.svg"),
                "spon_kofi",
            ),
            spon_oc: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/sponsor/CibKoFi.svg"), // If this is meant for something else, please correct me
                "spon_oc",
            ),
            flag_en: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForUnitedKingdom.svg"),
                "flag_en",
            ),
            flag_pl: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForPoland.svg"),
                "flag_pl",
            ),
            flag_de: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForGermany.svg"),
                "flag_de",
            ),
            flag_ua: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForUkraine.svg"),
                "flag_ua",
            ),
            flag_cz: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForCzechia.svg"),
                "flag_cz",
            ),
            flag_sk: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForSlovakia.svg"),
                "flag_sk",
            ),
            flag_fr: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForFrance.svg"),
                "flag_fr",
            ),
            flag_es: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForSpain.svg"),
                "flag_es",
            ),
            flag_br: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForBrazil.svg"),
                "flag_br",
            ),
            flag_jp: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForJapan.svg"),
                "flag_jp",
            ),
            flag_cn: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForChina.svg"),
                "flag_cn",
            ),
            flag_nl: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForNetherlands.svg"),
                "flag_nl",
            ),
            flag_ru: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForRussia.svg"),
                "flag_ru",
            ),
            flag_it: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/EmojioneV1FlagForItaly.svg"),
                "flag_it",
            ),
            flag_ar: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/flags/ArabicFlag.svg"),
                "flag_ar",
            ),
            close: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/IconamoonCloseLight.svg"),
                "icon_close",
            ),
            reset: Self::load_svg(
                ctx,
                include_bytes!("../../sen-desktop/assets/ui/CarbonResetAlt.svg"),
                "icon_reset",
            ),
        }
    }

    fn load_svg(ctx: &Context, svg_bytes: &[u8], name: &str) -> TextureHandle {
        let image = Self::render_svg(svg_bytes, 128, 128); // Increased from 32x32 for sharp large icons
        ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
    }

    fn render_svg(svg_data: &[u8], width: u32, height: u32) -> ColorImage {
        let opt = usvg::Options::default();

        // Support for currentColor: re-map to white so it can be tinted by theme colors in egui.
        let svg_str = String::from_utf8_lossy(svg_data).replace("currentColor", "white");
        let tree = usvg::Tree::from_data(svg_str.as_bytes(), &opt).expect("Failed to parse SVG");

        let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("Failed to create pixmap");

        // Use the size() method instead of the field
        let tree_size = tree.size();

        // Calculate transform to scale SVG to the target size
        let transform = tiny_skia::Transform::from_scale(
            width as f32 / tree_size.width(),
            height as f32 / tree_size.height(),
        );

        // Render SVG
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Convert pixmap to ColorImage for egui
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
        let svg_data = include_bytes!("../../sen-desktop/assets/app_icon.svg");
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

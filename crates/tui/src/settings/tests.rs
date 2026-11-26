use super::*;

const TRUE_COLOR: TerminalCaps = TerminalCaps {
    color: ColorDepth::TrueColor,
    unicode: true,
    remote: false,
};

#[test]
fn toml_round_trip_is_lossless() {
    let settings = Settings {
        locale: Locale::PlPl,
        theme: ThemeId::HighContrast,
        ..Settings::default()
    };
    let text = toml::to_string(&settings);
    assert!(text.is_ok());
    let back = text.map(|t| toml::from_str::<Settings>(&t));
    assert!(matches!(back, Ok(Ok(ref s)) if *s == settings));
}

#[test]
fn missing_fields_take_defaults() {
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.volume == 10 && s.cues));
}

#[test]
fn the_guide_panel_is_on_by_default_and_old_values_without_it_keep_it_on() {
    assert!(Settings::default().guide_panel);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.guide_panel));
    let off = toml::from_str::<Settings>("guide_panel = false");
    assert!(matches!(off, Ok(ref s) if !s.guide_panel));
}

#[test]
fn the_help_window_is_centred_by_default_and_old_values_load_centred() {
    assert_eq!(Settings::default().help_dock, DockSide::Centre);
    let old = toml::from_str::<Settings>("guide_panel = false");
    assert!(matches!(old, Ok(ref s) if s.help_dock == DockSide::Centre));
    let left = toml::from_str::<Settings>("help_dock = \"left\"");
    assert!(matches!(left, Ok(ref s) if s.help_dock == DockSide::Left));
}

#[test]
fn the_moves_panel_is_on_by_default_and_old_values_without_it_keep_it_on() {
    assert!(Settings::default().moves_panel);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.moves_panel));
    let off = toml::from_str::<Settings>("moves_panel = false");
    assert!(matches!(off, Ok(ref s) if !s.moves_panel));
}

#[test]
fn the_moves_panel_round_trips() {
    let settings = Settings {
        moves_panel: false,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings && !s.moves_panel));
}

#[test]
fn the_guide_panel_round_trips() {
    let settings = Settings {
        guide_panel: false,
        ..Settings::default()
    };
    let text = toml::to_string(&settings);
    assert!(text.is_ok());
    let back = text.map(|t| toml::from_str::<Settings>(&t));
    assert!(matches!(back, Ok(Ok(ref s)) if *s == settings && !s.guide_panel));
}

#[test]
fn undo_is_allowed_by_default_and_old_values_without_it_keep_it_allowed() {
    assert!(Settings::default().undo_allowed);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.undo_allowed));
    let off = toml::from_str::<Settings>("undo_allowed = false");
    assert!(matches!(off, Ok(ref s) if !s.undo_allowed));
}

#[test]
fn winnable_only_is_off_by_default_and_round_trips() {
    assert!(!Settings::default().winnable_only);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if !s.winnable_only));
    let settings = Settings {
        winnable_only: true,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if s.winnable_only));
}

#[test]
fn caps_without_unicode_give_ascii_glyphs() {
    let caps = TerminalCaps {
        unicode: false,
        ..TRUE_COLOR
    };
    assert_eq!(Settings::for_caps(&caps).glyph_mode, GlyphMode::Ascii);
    assert_eq!(
        Settings::for_caps(&TRUE_COLOR).glyph_mode,
        GlyphMode::Unicode
    );
}

#[test]
fn forced_color_mode_overrides_detection() {
    let mut settings = Settings::default();
    assert_eq!(settings.color_depth(&TRUE_COLOR), ColorDepth::TrueColor);
    settings.color_mode = ColorMode::Ansi16;
    assert_eq!(settings.color_depth(&TRUE_COLOR), ColorDepth::Ansi16);
}

#[test]
fn audio_volume_is_clamped() {
    let settings = Settings {
        volume: 250,
        ..Settings::default()
    };
    assert_eq!(settings.audio().volume, 100);
}

#[test]
fn auto_finish_is_on_by_default_and_old_values_without_it_keep_it_on() {
    assert!(Settings::default().auto_finish);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.auto_finish));
    let off = toml::from_str::<Settings>("auto_finish = false");
    assert!(matches!(off, Ok(ref s) if !s.auto_finish));
}

#[test]
fn quick_move_is_on_by_default_and_old_values_without_it_keep_it_on() {
    assert!(Settings::default().quick_move);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.quick_move));
    let off = toml::from_str::<Settings>("quick_move = false");
    assert!(matches!(off, Ok(ref s) if !s.quick_move));
}

#[test]
fn quick_move_off_round_trips_through_the_aids() {
    let settings = Settings {
        quick_move: false,
        ..Settings::default()
    };
    assert!(!settings.aids().quick_move);
    let text = toml::to_string(&settings);
    assert!(text.is_ok());
    let back = text.map(|t| toml::from_str::<Settings>(&t));
    assert!(matches!(back, Ok(Ok(ref s)) if *s == settings && !s.quick_move));
}

#[test]
fn the_aura_is_full_by_default_and_old_values_without_it_keep_it_full() {
    assert_eq!(Settings::default().aura, AuraMode::Full);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.aura == AuraMode::Full));
}

#[test]
fn the_aura_focus_only_round_trips() {
    let settings = Settings {
        aura: AuraMode::FocusOnly,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("aura = \"focus_only\""));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn target_marks_are_on_by_default_and_old_values_without_them_keep_them_on() {
    assert!(Settings::default().target_marks);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.target_marks));
    let off = toml::from_str::<Settings>("target_marks = false");
    assert!(matches!(off, Ok(ref s) if !s.target_marks));
}

#[test]
fn auto_finish_round_trips() {
    let settings = Settings {
        auto_finish: false,
        ..Settings::default()
    };
    let text = toml::to_string(&settings);
    assert!(text.is_ok());
    let back = text.map(|t| toml::from_str::<Settings>(&t));
    assert!(matches!(back, Ok(Ok(ref s)) if *s == settings && !s.auto_finish));
}

#[test]
fn card_back_is_theme_by_default_and_old_values_without_it_keep_theme() {
    assert_eq!(Settings::default().card_back, CardBack::Theme);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.card_back == CardBack::Theme));
}

#[test]
fn card_back_round_trips() {
    let settings = Settings {
        card_back: CardBack::Monokai,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("card_back = \"monokai\""));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn the_aura_look_is_halo_thin_theme_by_default_and_old_values_keep_it() {
    let settings = Settings::default();
    assert_eq!(settings.aura_colour, AuraColour::Theme);
    assert_eq!(settings.aura_style, AuraStyle::Halo);
    assert_eq!(settings.aura_size, AuraSize::Thin);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(
        matches!(parsed, Ok(ref s) if s.aura_style == AuraStyle::Halo
        && s.aura_size == AuraSize::Thin && s.aura_colour == AuraColour::Theme)
    );
}

#[test]
fn the_aura_look_round_trips() {
    let settings = Settings {
        aura_colour: AuraColour::Hint,
        aura_style: AuraStyle::Frame,
        aura_size: AuraSize::Thick,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("aura_style = \"frame\""));
    assert!(text.contains("aura_size = \"thick\""));
    assert!(text.contains("aura_colour = \"hint\""));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn the_interface_fields_come_from_the_saved_settings_and_the_rest_stays() {
    let previewed = Settings {
        theme: ThemeId::Dracula,
        aura_size: AuraSize::Thick,
        card_size: CardSizeSetting::Huge,
        volume: 20,
        ..Settings::default()
    };
    let kept = previewed.with_interface_of(&Settings::default());
    assert_eq!(kept.theme, ThemeId::Dark);
    assert_eq!(kept.aura_size, AuraSize::Thin);
    assert_eq!(kept.card_size, CardSizeSetting::Auto);
    assert_eq!(kept.volume, 20);
}

#[test]
fn the_console_font_choices_are_the_console_default_and_old_values_keep_it() {
    let settings = Settings::default();
    assert_eq!(settings.console_face, ConsoleFace::Default);
    assert_eq!(settings.console_font_size, 0);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(
        matches!(parsed, Ok(ref s) if s.console_face == ConsoleFace::Default
        && s.console_font_size == 0)
    );
}

#[test]
fn the_console_font_choices_round_trip() {
    let settings = Settings {
        console_face: ConsoleFace::CascadiaMono,
        console_font_size: 24,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("console_face = \"cascadia_mono\""));
    assert!(text.contains("console_font_size = 24"));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn the_faces_are_named_and_the_sizes_start_with_the_default() {
    assert_eq!(ConsoleFace::Default.name(), None);
    for face in &ConsoleFace::ALL[1..] {
        assert!(face.name().is_some());
    }
    assert_eq!(CONSOLE_FONT_SIZES[0], 0);
    assert!(CONSOLE_FONT_SIZES.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn the_console_font_choices_are_previewed_with_the_rest_of_the_interface() {
    let previewed = Settings {
        console_face: ConsoleFace::Consolas,
        console_font_size: 16,
        ..Settings::default()
    };
    let kept = previewed.with_interface_of(&Settings::default());
    assert_eq!(kept.console_face, ConsoleFace::Default);
    assert_eq!(kept.console_font_size, 0);
}

#[test]
fn card_face_is_theme_by_default_and_old_values_without_it_keep_theme() {
    assert_eq!(Settings::default().card_face, CardFaceTint::Theme);
    let parsed = toml::from_str::<Settings>("card_back = \"monokai\"\nvolume = 10");
    assert!(matches!(parsed, Ok(ref s) if s.card_face == CardFaceTint::Theme));
}

#[test]
fn card_face_round_trips() {
    let settings = Settings {
        card_face: CardFaceTint::LightGrey,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("card_face = \"light_grey\""));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn table_colour_and_feel_default_to_the_theme_and_plain_and_old_values_keep_them() {
    assert_eq!(Settings::default().table_colour, TableColour::Theme);
    assert_eq!(Settings::default().table_feel, TableFeel::Plain);
    let parsed = toml::from_str::<Settings>("card_back = \"monokai\"\nvolume = 10");
    assert!(
        matches!(parsed, Ok(ref s) if s.table_colour == TableColour::Theme && s.table_feel == TableFeel::Plain)
    );
}

#[test]
fn table_colour_and_feel_round_trip() {
    let settings = Settings {
        table_colour: TableColour::Brown,
        table_feel: TableFeel::Felt,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("table_colour = \"brown\""));
    assert!(text.contains("table_feel = \"felt\""));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn the_table_is_previewed_with_the_rest_of_the_interface() {
    let saved = Settings::default();
    let previewed = Settings {
        table_colour: TableColour::Blue,
        table_feel: TableFeel::Lined,
        ..saved.clone()
    };
    let kept = previewed.with_interface_of(&saved);
    assert_eq!(kept.table_colour, TableColour::Theme);
    assert_eq!(kept.table_feel, TableFeel::Plain);
}

#[test]
fn color_vision_and_four_color_deck_default_and_round_trip() {
    assert_eq!(Settings::default().color_vision, ColorVision::Normal);
    assert!(!Settings::default().four_color_deck);
    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(
        matches!(parsed, Ok(ref s) if s.color_vision == ColorVision::Normal && !s.four_color_deck)
    );

    let settings = Settings {
        color_vision: ColorVision::Deuteranopia,
        four_color_deck: true,
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("color_vision = \"deuteranopia\""));
    assert!(text.contains("four_color_deck = true"));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

#[test]
fn color_vision_and_four_color_deck_are_previewed_with_interface() {
    let saved = Settings::default();
    let previewed = Settings {
        color_vision: ColorVision::Tritanopia,
        four_color_deck: true,
        ..saved.clone()
    };
    let kept = previewed.with_interface_of(&saved);
    assert_eq!(kept.color_vision, ColorVision::Normal);
    assert!(!kept.four_color_deck);
}

#[test]
fn low_vision_defaults_off_and_unoffered_and_old_settings_load_as_not_offered() {
    let defaults = Settings::default();
    assert!(!defaults.low_vision);
    assert!(!defaults.low_vision_offered);
    assert_eq!(defaults.low_vision_saved, None);

    let parsed = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(
        parsed,
        Ok(ref s) if !s.low_vision && !s.low_vision_offered && s.low_vision_saved.is_none()
    ));
}

#[test]
fn low_vision_bundle_applies_and_reverts_exact_previous_values() {
    let base = Settings {
        theme: ThemeId::Dracula,
        card_size: CardSizeSetting::Medium,
        aura_size: AuraSize::Thin,
        console_font_size: 16,
        ..Settings::default()
    };

    let on_classic = base.with_low_vision(true, true);
    assert!(on_classic.low_vision);
    assert_eq!(on_classic.theme, ThemeId::HighContrast);
    assert_eq!(on_classic.card_size, CardSizeSetting::Auto);
    assert_eq!(on_classic.aura_size, AuraSize::Thick);
    assert_eq!(on_classic.console_font_size, LOW_VISION_MIN_FONT_SIZE);

    let adjusted = Settings {
        theme: ThemeId::Monokai,
        card_size: CardSizeSetting::Large,
        aura_size: AuraSize::Thin,
        console_font_size: 32,
        ..on_classic
    };
    let reverted = adjusted.with_low_vision(false, true);
    assert!(!reverted.low_vision);
    assert_eq!(reverted.low_vision_saved, None);
    assert_eq!(reverted, base);

    let on_other = base.with_low_vision(true, false);
    assert_eq!(on_other.console_font_size, 16);
    assert_eq!(on_other.with_low_vision(false, false), base);

    let already_large = Settings {
        console_font_size: 28,
        ..base.clone()
    };
    let kept_large = already_large.with_low_vision(true, true);
    assert_eq!(kept_large.console_font_size, 28);
    assert_eq!(kept_large.with_low_vision(false, true), already_large);
}

#[test]
fn low_vision_round_trips_and_previews_with_interface() {
    let base = Settings {
        theme: ThemeId::Light,
        console_font_size: 14,
        ..Settings::default()
    };
    let on = base.with_low_vision(true, true);
    let text = toml::to_string(&on).unwrap_or_default();
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == on));

    let kept = on.with_interface_of(&base);
    assert_eq!(kept, base);
}

#[test]
fn interface_size_defaults_to_m_for_old_saves_and_round_trips() {
    let old = "locale = \"en-US\"\n";
    let loaded = toml::from_str::<Settings>(old);
    assert!(matches!(loaded, Ok(ref s) if s.interface_size == crate::layout::InterfaceSize::M));

    for step in crate::layout::InterfaceSize::ALL {
        let settings = Settings {
            interface_size: step,
            ..Settings::default()
        };
        let encoded = toml::to_string(&settings).unwrap_or_default();
        let decoded = toml::from_str::<Settings>(&encoded);
        assert!(matches!(decoded, Ok(ref s) if s.interface_size == step));
        assert_eq!(
            settings
                .with_interface_of(&Settings::default())
                .interface_size,
            crate::layout::InterfaceSize::M
        );
    }
}

#[test]
fn the_game_options_default_to_the_old_rules_and_an_old_file_loads_them() {
    assert_eq!(
        Settings::default().game_options,
        oxysol_core::GameOptions::DEFAULT
    );
    let old = toml::from_str::<Settings>("volume = 10");
    assert_eq!(
        old.map(|settings| settings.game_options).ok(),
        Some(oxysol_core::GameOptions::DEFAULT)
    );
    let changed = toml::from_str::<Settings>(
        "[game_options]
klondike_passes = \"one\"
golf_wrap = true",
    );
    let Ok(changed) = changed else {
        panic!("the game options parse");
    };
    assert_eq!(
        changed.game_options.klondike_passes,
        oxysol_core::StockPasses::One
    );
    assert!(changed.game_options.golf_wrap);
    assert!(!changed.game_options.spider_deal_empty);
}

#[test]
fn folded_families_default_empty_for_old_saves_and_round_trip() {
    assert!(Settings::default().folded_families.is_empty());
    let old = toml::from_str::<Settings>("volume = 10");
    assert!(matches!(old, Ok(ref s) if s.folded_families.is_empty()));

    let settings = Settings {
        folded_families: vec![
            GameGroup::Favourites,
            GameGroup::FreeCell,
            GameGroup::TriPeaks,
        ],
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap_or_default();
    assert!(text.contains("folded_families = ["));
    let back = toml::from_str::<Settings>(&text);
    assert!(matches!(back, Ok(ref s) if *s == settings));
}

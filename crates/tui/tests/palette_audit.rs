//! Palette audit tests to prevent color drift.
//!
//! These tests ensure that deprecated colors are not used directly in
//! user-visible code. Backward-compatible DeepSeek aliases should point
//! at the current CodeWhale semantic tokens instead of stale brand RGBs.

use ratatui::style::Color;

#[path = "../src/palette/mod.rs"]
#[allow(dead_code)]
mod palette;

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Gray => (128, 128, 128),
        Color::DarkGray => (169, 169, 169),
        Color::Red => (255, 0, 0),
        Color::LightRed => (255, 102, 102),
        Color::Green => (0, 255, 0),
        Color::LightGreen => (102, 255, 102),
        Color::Yellow => (255, 255, 0),
        Color::LightYellow => (255, 255, 153),
        Color::Blue => (0, 0, 255),
        Color::LightBlue => (102, 153, 255),
        Color::Magenta => (255, 0, 255),
        Color::LightMagenta => (255, 153, 255),
        Color::Cyan => (0, 255, 255),
        Color::LightCyan => (153, 255, 255),
        _ => panic!("unsupported color variant for contrast test: {color:?}"),
    }
}

fn linearize_srgb(component: u8) -> f64 {
    let srgb = f64::from(component) / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(color: Color) -> f64 {
    let (r, g, b) = color_to_rgb(color);
    0.2126 * linearize_srgb(r) + 0.7152 * linearize_srgb(g) + 0.0722 * linearize_srgb(b)
}

fn contrast_ratio(foreground: Color, background: Color) -> f64 {
    let fg = relative_luminance(foreground);
    let bg = relative_luminance(background);
    if fg >= bg {
        (fg + 0.05) / (bg + 0.05)
    } else {
        (bg + 0.05) / (fg + 0.05)
    }
}

fn assert_min_contrast(label: &str, foreground: Color, background: Color, min_ratio: f64) {
    let ratio = contrast_ratio(foreground, background);
    assert!(
        ratio >= min_ratio,
        "{label} contrast {ratio:.2} is below minimum {min_ratio:.2}"
    );
}

// NOTE: The deprecated color audit (DEEPSEEK_AQUA) was removed because
// the deprecated constant no longer exists in the palette.

#[test]
fn verify_status_success_uses_success_token() {
    assert_eq!(
        palette::STATUS_SUCCESS,
        Color::Rgb(
            palette::WHALE_SUCCESS_RGB.0,
            palette::WHALE_SUCCESS_RGB.1,
            palette::WHALE_SUCCESS_RGB.2
        ),
        "STATUS_SUCCESS should use the current success token"
    );
    assert_ne!(
        palette::STATUS_SUCCESS,
        palette::WHALE_ACCENT_PRIMARY,
        "STATUS_SUCCESS should not regress to the primary accent"
    );
}

#[test]
fn whale_blue_stage_roles_are_pinned_and_non_colliding() {
    assert_eq!(palette::WHALE_BG_RGB, (3, 7, 13));
    assert_eq!(palette::WHALE_PANEL_RGB, (14, 23, 41));
    assert_eq!(palette::WHALE_ELEVATED_RGB, (24, 39, 66));
    assert_eq!(palette::WHALE_ACTION_RGB, (106, 174, 242));
    assert_eq!(palette::WHALE_ACCENT_SECONDARY_RGB, (79, 209, 197));
    assert_eq!(palette::WHALE_HUMAN_RGB, (246, 196, 83));
    assert_eq!(palette::WHALE_WARNING_RGB, (255, 122, 89));
    assert_eq!(palette::WHALE_ERROR_RGB, (255, 134, 178));
    assert_eq!(palette::WHALE_MODE_OPERATE_RGB, (173, 136, 255));

    let ui = palette::UI_THEME;
    assert_eq!(ui.accent_primary, palette::WHALE_ACTION);
    assert_eq!(ui.info, palette::WHALE_ACTION);
    assert_eq!(ui.status_working, palette::WHALE_LIVE);
    assert_eq!(ui.accent_action, palette::WHALE_HUMAN);
    assert_eq!(ui.warning, palette::STATUS_WARNING);
    assert_eq!(ui.error_fg, palette::WHALE_ERROR);
    assert_eq!(ui.mode_operate, palette::MODE_OPERATE);
    assert_ne!(
        ui.status_working, ui.success,
        "live and done need separate ink"
    );
    assert_ne!(ui.accent_action, ui.warning, "human asks are not warnings");
    assert_ne!(
        ui.warning, ui.error_fg,
        "warning and danger must not collapse"
    );
}

#[test]
fn contrast_guardrails_for_key_ui_pairs() {
    let min_readable = 4.5;

    assert_min_contrast(
        "TEXT_BODY on WHALE_BG",
        palette::TEXT_BODY,
        palette::WHALE_BG,
        min_readable,
    );
    assert_min_contrast(
        "TEXT_SECONDARY on WHALE_BG",
        palette::TEXT_SECONDARY,
        palette::WHALE_BG,
        min_readable,
    );
    assert_min_contrast(
        "TEXT_HINT on WHALE_BG",
        palette::TEXT_HINT,
        palette::WHALE_BG,
        min_readable,
    );
    assert_min_contrast(
        "STATUS_WARNING on WHALE_BG",
        palette::STATUS_WARNING,
        palette::WHALE_BG,
        min_readable,
    );
    assert_min_contrast(
        "STATUS_ERROR on WHALE_BG",
        palette::STATUS_ERROR,
        palette::WHALE_BG,
        min_readable,
    );
    assert_min_contrast(
        "SELECTION_TEXT on SELECTION_BG",
        palette::SELECTION_TEXT,
        palette::SELECTION_BG,
        min_readable,
    );
    assert_min_contrast(
        "TEXT_PRIMARY on SURFACE_ELEVATED",
        palette::TEXT_PRIMARY,
        palette::SURFACE_ELEVATED,
        min_readable,
    );
    for (label, foreground) in [
        ("action", palette::UI_THEME.accent_primary),
        ("live", palette::UI_THEME.status_working),
        ("human", palette::UI_THEME.accent_action),
        ("warning", palette::UI_THEME.warning),
        ("danger", palette::UI_THEME.error_fg),
        ("operate", palette::UI_THEME.mode_operate),
        ("success", palette::UI_THEME.success),
    ] {
        assert_min_contrast(label, foreground, palette::SURFACE_ELEVATED, min_readable);
    }
    assert_min_contrast(
        "LIGHT_TEXT_BODY on LIGHT_SURFACE",
        palette::LIGHT_TEXT_BODY,
        palette::LIGHT_SURFACE,
        min_readable,
    );
    assert_min_contrast(
        "LIGHT_TEXT_MUTED on LIGHT_SURFACE",
        palette::LIGHT_TEXT_MUTED,
        palette::LIGHT_SURFACE,
        min_readable,
    );
    assert_min_contrast(
        "LIGHT_TEXT_BODY on LIGHT_SELECTION_BG",
        palette::LIGHT_TEXT_BODY,
        palette::LIGHT_SELECTION_BG,
        min_readable,
    );
    for (label, foreground) in [
        ("light action", palette::LIGHT_UI_THEME.accent_primary),
        ("light live", palette::LIGHT_UI_THEME.status_working),
        ("light human", palette::LIGHT_UI_THEME.accent_action),
        ("light warning", palette::LIGHT_UI_THEME.warning),
        ("light danger", palette::LIGHT_UI_THEME.error_fg),
        ("light operate", palette::LIGHT_UI_THEME.mode_operate),
        ("light success", palette::LIGHT_UI_THEME.success),
    ] {
        assert_min_contrast(label, foreground, palette::LIGHT_SURFACE, min_readable);
    }
}

//! Ambient ocean life for the underwater transcript field.
//!
//! One clear owner for schools of fish, jellyfish, kelp, bubbles, bio-luminescence,
//! and the rare whale cameo. Motion stays inside the existing delta/interpolation
//! path: this module never requests frames on its own.
//!
//! Under reduced motion, entities remain visible but static.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::tui::ocean::{self, OceanColumn};

/// Depth layers for parallax. Nearer life is larger, faster, and more visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Depth {
    Background,
    Midground,
    Foreground,
}

impl Depth {
    #[must_use]
    fn speed_scale(self) -> f64 {
        match self {
            Self::Background => 0.55,
            Self::Midground => 1.0,
            Self::Foreground => 1.45,
        }
    }

    #[must_use]
    fn ink_index(self) -> usize {
        match self {
            Self::Background => 1,
            Self::Midground | Self::Foreground => 0,
        }
    }
}

/// Creature density tier mirrored from shell width/height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeDensity {
    Sparse,
    Normal,
    Rich,
}

impl LifeDensity {
    #[must_use]
    pub fn from_area(area: Rect) -> Self {
        if area.width < 56 || area.height < 12 {
            Self::Sparse
        } else if area.width < 88 || area.height < 20 {
            Self::Normal
        } else {
            Self::Rich
        }
    }

    #[must_use]
    fn school_count(self) -> usize {
        match self {
            Self::Sparse => 1,
            Self::Normal => 2,
            Self::Rich => 3,
        }
    }

    #[must_use]
    fn jellyfish_count(self) -> usize {
        match self {
            Self::Sparse => 1,
            Self::Normal | Self::Rich => 2,
        }
    }

    #[must_use]
    fn kelp_count(self) -> usize {
        match self {
            Self::Sparse => 2,
            Self::Normal => 3,
            Self::Rich => 5,
        }
    }

    #[must_use]
    fn bubble_streams(self) -> usize {
        match self {
            Self::Sparse => 1,
            Self::Normal => 2,
            Self::Rich => 3,
        }
    }

    #[must_use]
    fn bio_particles(self) -> usize {
        match self {
            Self::Sparse => 2,
            Self::Normal => 4,
            Self::Rich => 6,
        }
    }
}

/// Lower floors so smaller windows still retain some life (was 68×15).
/// Keep in sync with [`crate::tui::ocean::AMBIENT_MIN_WIDTH`].
pub const AMBIENT_MIN_WIDTH: u16 = crate::tui::ocean::AMBIENT_MIN_WIDTH;
pub const AMBIENT_MIN_HEIGHT: u16 = crate::tui::ocean::AMBIENT_MIN_HEIGHT;

/// Whale cameo state: brief breach → spout → fluke → submerge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhaleCameoPhase {
    Hidden,
    Breach,
    Spout,
    Fluke,
    Submerge,
}

/// Snapshot of ambient positions for one frame (memoized once per draw).
#[derive(Debug, Clone)]
struct FrameMarks {
    marks: Vec<AmbientMark>,
}

#[derive(Debug, Clone, Copy)]
struct AmbientMark {
    x: u16,
    y: u16,
    glyph: &'static str,
    depth: Depth,
    style_mod: Option<Modifier>,
}

/// Optional pointer reaction for fish dart / bubble rise.
#[derive(Debug, Clone, Copy, Default)]
pub struct AmbientCursor {
    pub column: u16,
    pub row: u16,
    /// When set, fish flee from this point for ~800 ms of shared ocean clock.
    pub flee_elapsed_ms: Option<u128>,
}

/// Optional whale cameo trigger (e.g. successful turn completion).
#[derive(Debug, Clone, Copy, Default)]
pub struct WhaleCameo {
    pub elapsed_ms: Option<u128>,
    /// Anchor column within the field (composer / center).
    pub anchor_x: u16,
    pub anchor_y: u16,
}

const WHALE_CAMEO_MS: u128 = 2_400;

/// Render ambient life into empty water cells of `area`.
pub fn render_ambient_life(
    area: Rect,
    buf: &mut Buffer,
    inks: (Color, Color),
    lines: &[Line<'static>],
    elapsed_ms: u128,
    animated: bool,
    cursor: AmbientCursor,
    whale: WhaleCameo,
) {
    if area.width < AMBIENT_MIN_WIDTH || area.height < AMBIENT_MIN_HEIGHT {
        return;
    }

    let density = LifeDensity::from_area(area);
    let frame = build_frame_marks(area, elapsed_ms, animated, density, cursor, whale);
    paint_marks(area, buf, inks, lines, &frame);
}

fn build_frame_marks(
    area: Rect,
    elapsed_ms: u128,
    animated: bool,
    density: LifeDensity,
    cursor: AmbientCursor,
    whale: WhaleCameo,
) -> FrameMarks {
    let mut marks = Vec::with_capacity(32);
    let t = if animated { elapsed_ms } else { 0 };

    // --- Schools of small fish (foreground / mid) ---
    let school_n = density.school_count();
    for i in 0..school_n {
        let depth = if i == 0 {
            Depth::Foreground
        } else if i == 1 {
            Depth::Midground
        } else {
            Depth::Background
        };
        let span = ((area.width as f64 / (5.0 + f64::from(i as u16))) as u16).clamp(6, 22);
        let phase = (i as u128).saturating_mul(2_100);
        let step = (420.0 / depth.speed_scale()) as u128;
        let (drift, forward) = if animated {
            eased_drift(t, step.max(200), span, phase)
        } else {
            ((span / 3).max(1), i % 2 == 0)
        };
        let bob = if animated {
            sine_bob(t, 1_800 + phase, 1 + (i as u16 % 2))
        } else {
            0
        };
        let base_y = match i {
            0 => area.height.saturating_mul(3) / 4,
            1 => area.height * 3 / 8,
            _ => area.height / 6,
        }
        .saturating_add(bob)
        .min(area.height.saturating_sub(1));
        let base_x = match i {
            0 => area.width / 12,
            1 => area.width.saturating_mul(5) / 6,
            _ => area.width / 3,
        };
        let mut x = if i == 1 {
            base_x.saturating_sub(drift)
        } else {
            base_x.saturating_add(drift)
        };
        // Cursor / flee reaction
        if let Some(flee_ms) = cursor.flee_elapsed_ms {
            let flee = fish_flee_offset(flee_ms);
            let ptr = cursor.column.saturating_sub(area.x);
            if x.abs_diff(ptr) < 14 {
                if x >= ptr {
                    x = x.saturating_add(flee);
                } else {
                    x = x.saturating_sub(flee);
                }
            }
        }
        let max_x = area.width.saturating_sub(3);
        x = x.min(max_x);
        let faces_right = if i == 1 { !forward } else { forward };
        marks.push(AmbientMark {
            x,
            y: base_y,
            glyph: fish_mark(faces_right, depth),
            depth,
            style_mod: None,
        });
        // School mates (small trailing fish)
        if density != LifeDensity::Sparse {
            let mate_x = if faces_right {
                x.saturating_sub(4 + (i as u16))
            } else {
                x.saturating_add(4 + (i as u16)).min(max_x)
            };
            let mate_y = base_y
                .saturating_add(if i % 2 == 0 { 1 } else { 0 })
                .min(area.height.saturating_sub(1));
            marks.push(AmbientMark {
                x: mate_x,
                y: mate_y,
                glyph: if faces_right { "›" } else { "‹" },
                depth: Depth::Background,
                style_mod: None,
            });
        }
    }

    // --- Jellyfish (midground, restrained tentacle sway) ---
    for j in 0..density.jellyfish_count() {
        let phase = 3_400u128.saturating_add((j as u128) * 2_700);
        let span = (area.width / 10).clamp(4, 14);
        let (drift, _) = if animated {
            eased_drift(t, 900, span, phase)
        } else {
            (span / 2, true)
        };
        let bob = if animated {
            sine_bob(t, 2_400 + phase, 1)
        } else {
            0
        };
        let x = (area.width / 5 + (j as u16) * (area.width / 3) + drift)
            .min(area.width.saturating_sub(2));
        let y = (area.height / 5 + (j as u16) + bob).min(area.height.saturating_sub(2));
        let tentacle = if animated {
            match (t.saturating_add(phase) / 500) % 3 {
                0 => "⎞",
                1 => "│",
                _ => "⎠",
            }
        } else {
            "│"
        };
        marks.push(AmbientMark {
            x,
            y,
            glyph: "◉",
            depth: Depth::Midground,
            style_mod: Some(Modifier::DIM),
        });
        if y + 1 < area.height {
            marks.push(AmbientMark {
                x,
                y: y + 1,
                glyph: tentacle,
                depth: Depth::Midground,
                style_mod: Some(Modifier::DIM),
            });
        }
    }

    // --- Bottom-anchored kelp ---
    for k in 0..density.kelp_count() {
        let phase = (k as u128).saturating_mul(1_100);
        let sway = if animated {
            sine_bob(t.saturating_add(phase), 2_200, 1) as i16 - 0
        } else {
            0
        };
        let base_x = (area.width / (density.kelp_count() as u16 + 1)).saturating_mul(k as u16 + 1);
        let x = if sway > 0 {
            base_x.saturating_add(sway as u16)
        } else {
            base_x.saturating_sub((-sway) as u16)
        }
        .min(area.width.saturating_sub(1));
        let height = 2 + (k % 2) as u16;
        for h in 0..height {
            let y = area.height.saturating_sub(1 + h);
            let glyph = if h + 1 == height { "⌃" } else { "│" };
            marks.push(AmbientMark {
                x,
                y,
                glyph,
                depth: Depth::Background,
                style_mod: Some(Modifier::DIM),
            });
        }
    }

    // --- Rising bubble streams ---
    for b in 0..density.bubble_streams() {
        let phase = (b as u128).saturating_mul(1_700);
        let column = area.width / 4 + (b as u16) * (area.width / 5);
        let rise_period = 2_800u128.saturating_add(phase % 900);
        let rise = if animated {
            let cycle = (t.saturating_add(phase) % rise_period) as f64 / rise_period as f64;
            let max_rise = area.height.saturating_sub(2) as f64;
            (cycle * max_rise) as u16
        } else {
            area.height / 3
        };
        // Bubbles rise faster near cursor activity
        let boost = if cursor.flee_elapsed_ms.is_some()
            && column.abs_diff(cursor.column.saturating_sub(area.x)) < 10
        {
            2
        } else {
            0
        };
        let y = area
            .height
            .saturating_sub(1)
            .saturating_sub(rise.saturating_add(boost));
        let glyph = if animated {
            ["·", "˚", "°", "˚"][((t.saturating_add(phase)) / 280) as usize % 4]
        } else {
            "°"
        };
        marks.push(AmbientMark {
            x: column.min(area.width.saturating_sub(1)),
            y,
            glyph,
            depth: Depth::Foreground,
            style_mod: None,
        });
    }

    // --- Sparse bioluminescent particles ---
    for p in 0..density.bio_particles() {
        let seed = (p as u128).saturating_mul(9973).saturating_add(13);
        let x = ((seed.wrapping_mul(17).wrapping_add(t / 4_000)) % u128::from(area.width.max(1)))
            as u16;
        let y = ((seed.wrapping_mul(31).wrapping_add(t / 5_500)) % u128::from(area.height.max(1)))
            as u16;
        let twinkle = if animated {
            ((t.saturating_add(seed) / 600) % 4) < 2
        } else {
            true
        };
        if !twinkle {
            continue;
        }
        marks.push(AmbientMark {
            x: x.min(area.width.saturating_sub(1)),
            y: y.min(area.height.saturating_sub(1)),
            glyph: "·",
            depth: Depth::Background,
            style_mod: Some(Modifier::DIM),
        });
    }

    // --- Rare whale cameo ---
    if let Some(cameo_ms) = whale.elapsed_ms.filter(|ms| *ms < WHALE_CAMEO_MS) {
        let phase = whale_cameo_phase(cameo_ms);
        if phase != WhaleCameoPhase::Hidden {
            let ax = whale
                .anchor_x
                .saturating_sub(area.x)
                .min(area.width.saturating_sub(4));
            let ay = whale
                .anchor_y
                .saturating_sub(area.y)
                .min(area.height.saturating_sub(2));
            let (glyph, y_off) = match phase {
                WhaleCameoPhase::Breach => ("🐋", 0u16),
                WhaleCameoPhase::Spout => ("🐳", 0),
                WhaleCameoPhase::Fluke => ("～", 1),
                WhaleCameoPhase::Submerge => ("·", 1),
                WhaleCameoPhase::Hidden => ("", 0),
            };
            if !glyph.is_empty() {
                marks.push(AmbientMark {
                    x: ax,
                    y: ay.saturating_add(y_off).min(area.height.saturating_sub(1)),
                    glyph,
                    depth: Depth::Foreground,
                    style_mod: None,
                });
                if phase == WhaleCameoPhase::Spout && ay > 0 {
                    marks.push(AmbientMark {
                        x: ax.saturating_add(1).min(area.width.saturating_sub(1)),
                        y: ay.saturating_sub(1),
                        glyph: "˚",
                        depth: Depth::Foreground,
                        style_mod: None,
                    });
                }
            }
        }
    }

    FrameMarks { marks }
}

fn paint_marks(
    area: Rect,
    buf: &mut Buffer,
    inks: (Color, Color),
    lines: &[Line<'static>],
    frame: &FrameMarks,
) {
    for mark in &frame.marks {
        let protected = lines
            .get(usize::from(mark.y))
            .and_then(occupied_text_bounds);
        let mark_width = UnicodeWidthStr::width(mark.glyph);
        let collides = protected.is_some_and(|(start, end)| {
            usize::from(mark.x) < end.saturating_add(1)
                && usize::from(mark.x) + mark_width > start.saturating_sub(1)
        });
        if collides || mark.x.saturating_add(mark_width as u16) > area.width {
            continue;
        }
        let fg = if mark.depth.ink_index() == 1 {
            inks.1
        } else {
            inks.0
        };
        let mut style = Style::default().fg(fg);
        if let Some(m) = mark.style_mod {
            style = style.add_modifier(m);
        }
        for (offset, ch) in mark.glyph.chars().enumerate() {
            let cell = &mut buf[(area.x + mark.x + offset as u16, area.y + mark.y)];
            cell.set_symbol(&ch.to_string());
            cell.set_style(style);
        }
    }
}

/// Width-only occupied-text measurement (no per-line String allocation).
#[must_use]
pub fn occupied_text_bounds(line: &Line<'_>) -> Option<(usize, usize)> {
    if line.spans.is_empty() {
        return None;
    }
    let mut total = 0usize;
    let mut leading = 0usize;
    let mut seen_non_ws = false;
    let mut trailing_run = 0usize;

    for span in &line.spans {
        for ch in span.content.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            total = total.saturating_add(w);
            if ch.is_whitespace() {
                if !seen_non_ws {
                    leading = leading.saturating_add(w);
                } else {
                    trailing_run = trailing_run.saturating_add(w);
                }
            } else {
                seen_non_ws = true;
                trailing_run = 0;
            }
        }
    }
    if !seen_non_ws {
        return None;
    }
    Some((leading, total.saturating_sub(trailing_run)))
}

/// Cosine-eased drift (replaces mechanical linear ping-pong feel).
#[must_use]
pub fn eased_drift(elapsed_ms: u128, step_ms: u128, span: u16, phase_ms: u128) -> (u16, bool) {
    if span == 0 || step_ms == 0 {
        return (0, true);
    }
    let leg_ms = step_ms.saturating_mul(u128::from(span));
    let period_ms = leg_ms.saturating_mul(2);
    let phase = (elapsed_ms.saturating_add(phase_ms)) % period_ms;
    let (leg_elapsed, forward) = if phase <= leg_ms {
        (phase, true)
    } else {
        (phase.saturating_sub(leg_ms), false)
    };
    let progress = leg_elapsed as f64 / leg_ms as f64;
    let eased = (1.0 - (progress * std::f64::consts::PI).cos()) * 0.5;
    let position = if forward { eased } else { 1.0 - eased };
    ((position * f64::from(span)).round() as u16, forward)
}

#[must_use]
fn sine_bob(elapsed_ms: u128, period_ms: u128, amplitude: u16) -> u16 {
    if period_ms == 0 || amplitude == 0 {
        return 0;
    }
    let phase = (elapsed_ms % period_ms) as f64 / period_ms as f64;
    let s = (phase * std::f64::consts::TAU).sin();
    // Map [-1,1] → [0, amplitude]
    (((s + 1.0) * 0.5) * f64::from(amplitude)).round() as u16
}

/// One-shot flee arc keyed to Working transition / pointer motion.
#[must_use]
pub fn fish_flee_offset(elapsed_ms: u128) -> u16 {
    let progress = elapsed_ms.min(800) as f32 / 800.0;
    let excursion = (progress * std::f32::consts::PI).sin() * 9.0;
    excursion.round().clamp(0.0, 9.0) as u16
}

#[must_use]
fn fish_mark(facing_right: bool, depth: Depth) -> &'static str {
    match (facing_right, depth) {
        (true, Depth::Foreground) => "><>",
        (false, Depth::Foreground) => "<><",
        (true, _) => ">",
        (false, _) => "<",
    }
}

#[must_use]
pub fn whale_cameo_phase(elapsed_ms: u128) -> WhaleCameoPhase {
    match elapsed_ms {
        0..400 => WhaleCameoPhase::Breach,
        400..1_000 => WhaleCameoPhase::Spout,
        1_000..1_700 => WhaleCameoPhase::Fluke,
        1_700..WHALE_CAMEO_MS => WhaleCameoPhase::Submerge,
        _ => WhaleCameoPhase::Hidden,
    }
}

/// Subtle caustic shimmer applied to empty water cells when the field would
/// otherwise read as a static ramp. Cheap: one phase lookup per cell, only
/// when `animated` and density allows.
pub fn apply_caustic_shimmer(
    area: Rect,
    buf: &mut Buffer,
    column: &OceanColumn,
    elapsed_ms: u128,
    animated: bool,
    lines: &[Line<'static>],
) {
    if !animated || area.width < AMBIENT_MIN_WIDTH || area.height < AMBIENT_MIN_HEIGHT {
        return;
    }
    // Sparse sampling: every 3rd column on every other row near the surface.
    let band = (area.height / 3).max(2);
    for local_y in 0..band {
        let protected = lines
            .get(usize::from(local_y))
            .and_then(occupied_text_bounds);
        let ramp = frame_ocean_ramp(
            column,
            area.height,
            area.y,
            elapsed_ms,
            column.phase_tag(),
            column.ramp_fingerprint(),
        );
        let row_bg = ramp
            .get(usize::from(local_y))
            .copied()
            .unwrap_or_else(|| column.color_at_y(area.y.saturating_add(local_y)));
        for local_x in (0..area.width).step_by(3) {
            if protected.is_some_and(|(start, end)| {
                usize::from(local_x) >= start && usize::from(local_x) < end
            }) {
                continue;
            }
            let phase = ((elapsed_ms / 80)
                .wrapping_add(u128::from(local_x))
                .wrapping_add(u128::from(local_y) * 3))
                % 12;
            if phase > 2 {
                continue;
            }
            let cell = &mut buf[(area.x + local_x, area.y + local_y)];
            // Soften toward ambient ink without replacing semantic glyphs.
            if cell.symbol() == " " || cell.symbol().is_empty() {
                let shimmer = ocean::scale_color(row_bg, 1.08);
                cell.set_bg(shimmer);
            }
        }
    }
}

/// Cached ocean row colors invalidated only when phase/dimensions/palette/breath tick.
/// Shared across widgets that paint the same [`OceanColumn`] within a frame.
#[derive(Debug, Clone, Default)]
pub struct OceanRampCache {
    colors: Vec<Color>,
    height: u16,
    top: u16,
    elapsed_bucket: u128,
    phase_tag: u8,
    ramp_fingerprint: u64,
}

impl OceanRampCache {
    /// Return a per-row color ramp, recomputing only when inputs change.
    pub fn colors_for(
        &mut self,
        column: &OceanColumn,
        height: u16,
        top: u16,
        elapsed_ms: u128,
        phase_tag: u8,
        ramp_fingerprint: u64,
    ) -> &[Color] {
        // Breath cycle is 90s; bucket at ~80ms atmosphere cadence so we don't
        // recompute every draw when nothing visible changed.
        let bucket = elapsed_ms / 80;
        if self.colors.len() == usize::from(height)
            && self.height == height
            && self.top == top
            && self.elapsed_bucket == bucket
            && self.phase_tag == phase_tag
            && self.ramp_fingerprint == ramp_fingerprint
        {
            return &self.colors;
        }
        self.colors.clear();
        self.colors.reserve(usize::from(height));
        for local_y in 0..height {
            self.colors
                .push(column.color_at_y(top.saturating_add(local_y)));
        }
        self.height = height;
        self.top = top;
        self.elapsed_bucket = bucket;
        self.phase_tag = phase_tag;
        self.ramp_fingerprint = ramp_fingerprint;
        &self.colors
    }
}

thread_local! {
    static FRAME_RAMP: std::cell::RefCell<OceanRampCache> =
        const { std::cell::RefCell::new(OceanRampCache {
            colors: Vec::new(),
            height: 0,
            top: 0,
            elapsed_bucket: 0,
            phase_tag: 0,
            ramp_fingerprint: 0,
        }) };
}

/// Process-local per-frame ocean ramp shared by chat field, caustics, and
/// other widgets that paint the same column.
#[must_use]
pub fn frame_ocean_ramp(
    column: &OceanColumn,
    height: u16,
    top: u16,
    elapsed_ms: u128,
    phase_tag: u8,
    ramp_fingerprint: u64,
) -> Vec<Color> {
    FRAME_RAMP.with(|cache| {
        cache
            .borrow_mut()
            .colors_for(column, height, top, elapsed_ms, phase_tag, ramp_fingerprint)
            .to_vec()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;

    #[test]
    fn ambient_min_dimensions_allow_small_windows() {
        assert!(AMBIENT_MIN_WIDTH < 68);
        assert!(AMBIENT_MIN_HEIGHT < 15);
    }

    #[test]
    fn eased_drift_is_continuous_and_settles() {
        let span = 12u16;
        let step = 400u128;
        let (a, _) = eased_drift(0, step, span, 0);
        let (b, _) = eased_drift(step * 6, step, span, 0);
        assert!(a <= span);
        assert!(b <= span);
    }

    #[test]
    fn occupied_text_bounds_skips_string_join() {
        let line = Line::from(vec![Span::raw("  hello  "), Span::raw("world  ")]);
        let (start, end) = occupied_text_bounds(&line).expect("bounds");
        assert_eq!(start, 2);
        assert!(end > start);
    }

    #[test]
    fn whale_cameo_is_brief() {
        assert_eq!(whale_cameo_phase(0), WhaleCameoPhase::Breach);
        assert_eq!(whale_cameo_phase(500), WhaleCameoPhase::Spout);
        assert_eq!(whale_cameo_phase(1_200), WhaleCameoPhase::Fluke);
        assert_eq!(whale_cameo_phase(2_000), WhaleCameoPhase::Submerge);
        assert_eq!(whale_cameo_phase(3_000), WhaleCameoPhase::Hidden);
    }

    #[test]
    fn density_scales_with_area() {
        assert_eq!(
            LifeDensity::from_area(Rect::new(0, 0, 40, 10)),
            LifeDensity::Sparse
        );
        assert_eq!(
            LifeDensity::from_area(Rect::new(0, 0, 100, 30)),
            LifeDensity::Rich
        );
    }
}

use crate::banner::{build_card_texture, decode_bnr, extract_banner};
use macroquad::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub const SUPPORTED_EXTS: &[&str] = &["iso", "gcm", "rvz", "wbfs", "ciso", "gcz"];

pub struct Game {
    pub path: PathBuf,
    pub title: String,
    pub texture: Texture2D,
    pub dolphin_entry: bool,
}

pub fn load_sidecar(path: &Path) -> Option<Texture2D> {
    let base = path.with_extension("");
    let candidates = [
        base.with_extension("jpg"),
        base.with_extension("jpeg"),
        base.with_extension("png"),
    ];
    for p in &candidates {
        if let Ok(img) = image::open(p) {
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let tex = Texture2D::from_rgba8(w as u16, h as u16, &rgba.into_raw());
            tex.set_filter(FilterMode::Linear);
            return Some(tex);
        }
    }
    None
}

pub fn load_games(dir: &Path) -> Vec<Game> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
    let mut games: Vec<Game> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !SUPPORTED_EXTS.contains(&ext.as_str()) { return None; }
            let stem = path.file_stem()?.to_str()?.to_string();
            let bg = color_from_str(&stem);
            let (banner_rgba, bnr_title) = extract_banner(&path, &stem)
                .and_then(|d| decode_bnr(&d))
                .map(|(px, t)| (Some(px), t))
                .unwrap_or((None, None));
            let title = bnr_title.unwrap_or_else(|| clean_filename_title(&stem));
            let texture = load_sidecar(&path)
                .unwrap_or_else(|| build_card_texture(banner_rgba.as_deref(), bg));
            Some(Game { path, title, texture, dolphin_entry: false })
        })
        .collect();
    games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    let dolphin_tex = option_env!("DOLPHIN_ICON")
        .and_then(|p| load_sidecar(Path::new(p)))
        .unwrap_or_else(|| build_card_texture(None, Color::new(0.38, 0.22, 0.72, 1.0)));
    games.push(Game {
        path: PathBuf::new(),
        title: "Open Dolphin".to_string(),
        texture: dolphin_tex,
        dolphin_entry: true,
    });

    games
}

fn clean_filename_title(stem: &str) -> String {
    let mut s = stem.to_string();
    while let (Some(o), Some(c)) = (s.find('('), s.find(')')) {
        if o < c { s.replace_range(o..=c, ""); } else { break; }
    }
    s.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn color_from_str(s: &str) -> Color {
    let h = s.bytes().fold(2_166_136_261u32, |a, b| {
        (a ^ b as u32).wrapping_mul(16_777_619)
    });
    let (r, g, b) = hsv_to_rgb((h % 360) as f32, 0.50, 0.80);
    Color::new(r, g, b, 1.0)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h6 = h / 60.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let (r, g, b) = match h6 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r + m, g + m, b + m)
}

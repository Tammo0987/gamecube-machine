use macroquad::prelude::*;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env};

pub const BANNER_W: usize = 96;
pub const BANNER_H: usize = 32;
pub const TEX_SIZE: usize = 96;

pub fn cache_dir() -> PathBuf {
    let base = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env::var("HOME").unwrap_or_default()).join(".cache"));
    base.join("gamecube-launcher")
}

pub fn extract_banner(game_path: &Path, stem: &str) -> Option<Vec<u8>> {
    let game_cache = cache_dir().join(stem);
    let bnr_path = game_cache.join("files").join("opening.bnr");
    if !bnr_path.exists() {
        fs::create_dir_all(&game_cache).ok()?;
        let out = Command::new("dolphin-tool")
            .args(["extract", "-i", game_path.to_str()?, "-s", "opening.bnr", "-o", game_cache.to_str()?])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
    }
    let mut data = Vec::new();
    fs::File::open(&bnr_path).ok()?.read_to_end(&mut data).ok()?;
    Some(data)
}

pub fn decode_bnr(data: &[u8]) -> Option<(Vec<u8>, Option<String>)> {
    if data.len() < 0x1820 { return None; }
    if &data[0..4] != b"BNR1" && &data[0..4] != b"BNR2" { return None; }

    let pixels = &data[0x20..0x20 + BANNER_W * BANNER_H * 2];
    let mut rgba = vec![0u8; BANNER_W * BANNER_H * 4];
    let tiles_x = BANNER_W / 4;
    let tiles_y = BANNER_H / 4;
    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            for py in 0..4 {
                for px in 0..4 {
                    let tile = ty * tiles_x + tx;
                    let pix = py * 4 + px;
                    let off = (tile * 16 + pix) * 2;
                    let p = u16::from_be_bytes([pixels[off], pixels[off + 1]]);
                    let (r, g, b, a) = if p & 0x8000 != 0 {
                        let r5 = (p >> 10) & 0x1F;
                        let g5 = (p >> 5) & 0x1F;
                        let b5 = p & 0x1F;
                        ((r5 * 255 / 31) as u8, (g5 * 255 / 31) as u8, (b5 * 255 / 31) as u8, 255)
                    } else {
                        let a3 = (p >> 12) & 0x7;
                        let r4 = (p >> 8) & 0xF;
                        let g4 = (p >> 4) & 0xF;
                        let b4 = p & 0xF;
                        ((r4 * 255 / 15) as u8, (g4 * 255 / 15) as u8, (b4 * 255 / 15) as u8, (a3 * 255 / 7) as u8)
                    };
                    let x = tx * 4 + px;
                    let y = ty * 4 + py;
                    let i = (y * BANNER_W + x) * 4;
                    rgba[i] = r; rgba[i + 1] = g; rgba[i + 2] = b; rgba[i + 3] = a;
                }
            }
        }
    }

    let title = if data.len() >= 0x18A0 {
        let bytes = &data[0x1860..0x18A0];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let s: String = bytes[..end].iter().map(|&b| b as char).collect();
        let s = s.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    };

    Some((rgba, title))
}

pub fn build_card_texture(banner_rgba: Option<&[u8]>, bg: Color) -> Texture2D {
    let bg8 = [
        (bg.r * 255.0) as u8,
        (bg.g * 255.0) as u8,
        (bg.b * 255.0) as u8,
        255u8,
    ];
    let mut img = vec![0u8; TEX_SIZE * TEX_SIZE * 4];
    for i in 0..TEX_SIZE * TEX_SIZE {
        img[i * 4..i * 4 + 4].copy_from_slice(&bg8);
    }
    if let Some(b) = banner_rgba {
        let y_off = (TEX_SIZE - BANNER_H) / 2;
        for y in 0..BANNER_H {
            for x in 0..BANNER_W {
                let src = (y * BANNER_W + x) * 4;
                let dst = ((y_off + y) * TEX_SIZE + x) * 4;
                img[dst..dst + 4].copy_from_slice(&b[src..src + 4]);
            }
        }
    }
    let tex = Texture2D::from_rgba8(TEX_SIZE as u16, TEX_SIZE as u16, &img);
    tex.set_filter(FilterMode::Linear);
    tex
}

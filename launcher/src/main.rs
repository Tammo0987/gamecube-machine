use gilrs::{Axis, Button, EventType, Gilrs};
use macroquad::prelude::*;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const CARD_W: f32 = 0.75;
const CARD_H: f32 = 1.0;
const CENTER_GAP: f32 = 1.0;
const SIDE_GAP: f32 = 0.30;
const MAX_ROT: f32 = std::f32::consts::PI / 5.0;
const SUPPORTED_EXTS: &[&str] = &["iso", "gcm", "rvz", "wbfs", "ciso", "gcz"];

const BANNER_W: usize = 96;
const BANNER_H: usize = 32;
const TEX_SIZE: usize = 96; // square card texture: banner centered vertically

struct Game {
    path: PathBuf,
    title: String,
    texture: Texture2D,
    dolphin_entry: bool,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    base_alpha: f32,
    bokeh: bool,
}

// ── title / color helpers ────────────────────────────────────────────────────

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

// ── banner extraction & decoding ─────────────────────────────────────────────

fn cache_dir() -> PathBuf {
    let base = env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env::var("HOME").unwrap_or_default()).join(".cache"));
    base.join("gamecube-launcher")
}

fn extract_banner(game_path: &Path, stem: &str) -> Option<Vec<u8>> {
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

fn decode_bnr(data: &[u8]) -> Option<(Vec<u8>, Option<String>)> {
    if data.len() < 0x1820 { return None; }
    if &data[0..4] != b"BNR1" && &data[0..4] != b"BNR2" { return None; }

    // Image data: 96×32, 4×4 tiled, RGB5A3
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

    // Long title at 0x1860 (64 bytes, Latin-1)
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

fn build_card_texture(banner_rgba: Option<&[u8]>, bg: Color) -> Texture2D {
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
        let y_off = (TEX_SIZE - BANNER_H) / 2; // vertically center 32-row banner in 96-row texture
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

// ── particles ─────────────────────────────────────────────────────────────────

fn spawn_particle(bokeh: bool) -> Particle {
    use macroquad::rand::gen_range;
    if bokeh {
        Particle {
            x: gen_range(0.0f32, 1.0),
            y: gen_range(0.0f32, 1.0),
            vx: gen_range(-0.010f32, 0.010),
            vy: gen_range(-0.015f32, -0.005),
            radius: gen_range(50.0f32, 120.0),
            base_alpha: gen_range(0.04f32, 0.10),
            bokeh: true,
        }
    } else {
        Particle {
            x: gen_range(0.0f32, 1.0),
            y: gen_range(1.0f32, 1.3),
            vx: gen_range(-0.025f32, 0.025),
            vy: gen_range(-0.06f32, -0.025),
            radius: gen_range(2.0f32, 4.5),
            base_alpha: gen_range(0.5f32, 0.9),
            bokeh: false,
        }
    }
}

fn init_particles() -> Vec<Particle> {
    use macroquad::rand::gen_range;
    let mut p: Vec<Particle> = (0..16).map(|_| spawn_particle(true)).collect();
    let mut smalls: Vec<Particle> = (0..110).map(|_| {
        let mut s = spawn_particle(false);
        s.y = gen_range(0.0f32, 1.0);
        s
    }).collect();
    p.append(&mut smalls);
    p
}

fn update_particles(particles: &mut Vec<Particle>, dt: f32) {
    for p in particles.iter_mut() {
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        if p.bokeh {
            if p.x < -0.1 { p.x += 1.2; }
            if p.x > 1.1  { p.x -= 1.2; }
            if p.y < -0.1 { p.y += 1.2; }
            if p.y > 1.1  { p.y -= 1.2; }
        } else if p.y < -0.05 {
            *p = spawn_particle(false);
        }
    }
}

fn draw_particles(particles: &[Particle]) {
    let (w, h) = (screen_width(), screen_height());
    for p in particles {
        let px = p.x * w;
        let py = p.y * h;
        let alpha = if p.bokeh {
            p.base_alpha
        } else {
            p.base_alpha * (p.y / 0.25).min(1.0)
        };
        if p.bokeh {
            // single soft blob
            draw_circle(px, py, p.radius, Color::new(0.55, 0.70, 1.0, alpha));
        } else {
            // glow: outer halo → mid ring → bright core
            draw_circle(px, py, p.radius * 4.0, Color::new(0.55, 0.75, 1.0, alpha * 0.10));
            draw_circle(px, py, p.radius * 2.2, Color::new(0.65, 0.82, 1.0, alpha * 0.25));
            draw_circle(px, py, p.radius,       Color::new(0.88, 0.95, 1.0, alpha));
        }
    }
}

// ── sidecar image loader ──────────────────────────────────────────────────────

fn load_sidecar(game_path: &Path) -> Option<Texture2D> {
    let base = game_path.with_extension("");
    let candidates = [
        base.with_extension("jpg"),
        base.with_extension("jpeg"),
        base.with_extension("png"),
    ];
    for p in &candidates {
        if let Ok(img) = image::open(p) {
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let card = Texture2D::from_rgba8(w as u16, h as u16, &rgba.into_raw());
            card.set_filter(FilterMode::Linear);
            return Some(card);
        }
    }
    None
}

// ── game scanning ─────────────────────────────────────────────────────────────

fn load_games(dir: &Path) -> Vec<Game> {
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

    // Special entry: open Dolphin's own game browser
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

// ── rendering ─────────────────────────────────────────────────────────────────

fn draw_background() {
    let w = screen_width();
    let h = screen_height();
    // Dark navy-to-deep-blue gradient
    draw_mesh(&Mesh {
        vertices: vec![
            Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, Color::new(0.04, 0.05, 0.12, 1.0)),
            Vertex::new(w,   0.0, 0.0, 1.0, 0.0, Color::new(0.04, 0.05, 0.12, 1.0)),
            Vertex::new(w,   h,   0.0, 1.0, 1.0, Color::new(0.08, 0.08, 0.18, 1.0)),
            Vertex::new(0.0, h,   0.0, 0.0, 1.0, Color::new(0.08, 0.08, 0.18, 1.0)),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

fn card_mesh(center: Vec3, hw: f32, hh: f32, rot_y: f32, alpha_top: f32, alpha_bot: f32, flip_v: bool, tex: Option<Texture2D>) -> Mesh {
    let cos = rot_y.cos();
    let sin = rot_y.sin();
    let pt = |lx: f32, ly: f32, u: f32, v: f32, a: f32| {
        Vertex::new(center.x + lx * cos, center.y + ly, center.z - lx * sin, u, v, Color::new(1.0, 1.0, 1.0, a))
    };
    let (v0, v1) = if flip_v { (1.0, 0.0) } else { (0.0, 1.0) };
    Mesh {
        vertices: vec![
            pt(-hw,  hh, 0.0, v0, alpha_top),
            pt( hw,  hh, 1.0, v0, alpha_top),
            pt( hw, -hh, 1.0, v1, alpha_bot),
            pt(-hw, -hh, 0.0, v1, alpha_bot),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: tex,
    }
}

fn draw_card(center: Vec3, rot_y: f32, game: &Game) {
    let hw = CARD_W * 0.5;
    let hh = CARD_H * 0.5;
    // Main card
    draw_mesh(&card_mesh(center, hw, hh, rot_y, 1.0, 1.0, false, Some(game.texture.clone())));
    // Reflection: top (near card) opaque → bottom transparent, UVs flipped
    let rc = Vec3::new(center.x, center.y - CARD_H - 0.003, center.z);
    draw_mesh(&card_mesh(rc, hw, hh, rot_y, 0.35, 0.0, true, Some(game.texture.clone())));
}

fn card_position(offset: f32) -> (f32, f32, f32) {
    let sign = if offset >= 0.0 { 1.0 } else { -1.0 };
    let abs = offset.abs();
    let x = if abs <= 1.0 { offset * CENTER_GAP } else { sign * (CENTER_GAP + (abs - 1.0) * SIDE_GAP) };
    let rot = sign * MAX_ROT * abs.min(1.0);
    let z = -(abs.min(3.0)) * 0.15;
    (x, rot, z)
}

// ── window & main ─────────────────────────────────────────────────────────────

fn window_conf() -> Conf {
    Conf {
        window_title: "GameCube Launcher".into(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let games_dir = args
        .iter()
        .position(|a| a == "--games-dir")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "./games".into());

    let font: Option<Font> = match option_env!("LAUNCHER_FONT") {
        Some(path) => load_ttf_font(path).await.ok(),
        None => None,
    };

    eprintln!("Loading games from {}…", games_dir);
    let games = load_games(Path::new(&games_dir));
    eprintln!("Loaded {} game(s)", games.len());

    let mut gilrs = Gilrs::new().ok();
    let mut particles = init_particles();
    let mut target: f32 = 0.0;
    let mut animated: f32 = 0.0;
    let mut stick_held = false;
    let mut last_stick_step = 0.0f64;

    loop {
        let dt = get_frame_time();
        let now = get_time();
        let max_idx = games.len().saturating_sub(1) as f32;
        let mut nav = 0i32;
        let mut launch = false;

        // Keyboard
        if is_key_pressed(KeyCode::Left)  { nav -= 1; }
        if is_key_pressed(KeyCode::Right) { nav += 1; }
        if is_key_pressed(KeyCode::Enter) { launch = true; }
        if is_key_pressed(KeyCode::Escape) { break; }

        // Controller
        if let Some(g) = gilrs.as_mut() {
            while let Some(ev) = g.next_event() {
                match ev.event {
                    EventType::ButtonPressed(Button::DPadLeft, _)             => nav -= 1,
                    EventType::ButtonPressed(Button::DPadRight, _)            => nav += 1,
                    EventType::ButtonPressed(Button::South, _)
                    | EventType::ButtonPressed(Button::East, _)               => launch = true,
                    _ => {}
                }
            }
            let mut stick = 0.0_f32;
            for (_, gp) in g.gamepads() {
                let v = gp.value(Axis::LeftStickX);
                if v.abs() > stick.abs() { stick = v; }
            }
            if stick.abs() > 0.5 {
                if !stick_held || now - last_stick_step > 0.18 {
                    nav += stick.signum() as i32;
                    stick_held = true;
                    last_stick_step = now;
                }
            } else {
                stick_held = false;
            }
        }

        if nav != 0 && !games.is_empty() {
            target = (target + nav as f32).clamp(0.0, max_idx);
        }
        if launch {
            let idx = target.round() as usize;
            if let Some(g) = games.get(idx) {
                if g.dolphin_entry {
                    let _ = Command::new("dolphin-emu").status();
                } else {
                    let _ = Command::new("dolphin-emu").args(["-b", "-e"]).arg(&g.path).status();
                }
            }
        }

        let lerp = 1.0 - (-dt * 12.0).exp();
        animated += (target - animated) * lerp;

        // ── draw ────────────────────────────────────────────────────────────
        clear_background(BLACK);
        draw_background();
        update_particles(&mut particles, dt);
        draw_particles(&particles);

        set_camera(&Camera3D {
            position: vec3(0.0, 0.15, 3.8),
            target:   vec3(0.0, 0.0, 0.0),
            up:       vec3(0.0, 1.0, 0.0),
            fovy:     42.0_f32.to_radians(),
            aspect:   Some(screen_width() / screen_height()),
            projection: Projection::Perspective,
            ..Default::default()
        });

        // Back-to-front so alpha blending works
        let mut order: Vec<usize> = (0..games.len()).collect();
        order.sort_by(|&a, &b| {
            let da = (a as f32 - animated).abs();
            let db = (b as f32 - animated).abs();
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
        for i in order {
            let offset = i as f32 - animated;
            if offset.abs() > 6.0 { continue; }
            let (x, rot, z) = card_position(offset);
            draw_card(vec3(x, 0.0, z), rot, &games[i]);
        }

        set_default_camera();
        let font_ref = font.as_ref();

        // Title
        let title_str = "GameCube";
        let td = measure_text(title_str, font_ref, 52, 1.0);
        let tx = (screen_width() - td.width) * 0.5;
        let ty = screen_height() * 0.10;
        // subtle shadow
        draw_text_ex(title_str, tx + 2.0, ty + 2.0,
            TextParams { font: font_ref, font_size: 52, color: Color::new(0.0, 0.0, 0.0, 0.5), ..Default::default() });
        draw_text_ex(title_str, tx, ty,
            TextParams { font: font_ref, font_size: 52, color: WHITE, ..Default::default() });

        if games.is_empty() {
            let msg = format!("No games found in {}", games_dir);
            let d = measure_text(&msg, font_ref, 28, 1.0);
            draw_text_ex(&msg, (screen_width() - d.width) * 0.5, screen_height() * 0.5,
                TextParams { font: font_ref, font_size: 28, color: GRAY, ..Default::default() });
        } else {
            let idx = animated.round() as usize;
            if let Some(g) = games.get(idx) {
                let d = measure_text(&g.title, font_ref, 38, 1.0);
                draw_text_ex(&g.title, (screen_width() - d.width) * 0.5, screen_height() * 0.88,
                    TextParams { font: font_ref, font_size: 38, color: WHITE, ..Default::default() });
                let counter = format!("{} / {}", idx + 1, games.len());
                let cd = measure_text(&counter, font_ref, 18, 1.0);
                draw_text_ex(&counter, (screen_width() - cd.width) * 0.5, screen_height() * 0.93,
                    TextParams { font: font_ref, font_size: 18, color: Color::new(0.6, 0.6, 0.7, 1.0), ..Default::default() });
            }
        }

        next_frame().await;
    }
}

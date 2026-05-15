use crate::game::Game;
use macroquad::prelude::*;

const CARD_W: f32 = 0.75;
const CARD_H: f32 = 1.0;
const CENTER_GAP: f32 = 1.0;
const SIDE_GAP: f32 = 0.30;
const MAX_ROT: f32 = std::f32::consts::PI / 5.0;

pub fn draw_background() {
    let w = screen_width();
    let h = screen_height();
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

pub fn draw_carousel(games: &[Game], animated: f32) {
    set_camera(&Camera3D {
        position: vec3(0.0, 0.15, 3.8),
        target:   vec3(0.0, 0.0, 0.0),
        up:       vec3(0.0, 1.0, 0.0),
        fovy:     42.0_f32.to_radians(),
        aspect:   Some(screen_width() / screen_height()),
        projection: Projection::Perspective,
        ..Default::default()
    });

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
}

pub fn draw_hud(games: &[Game], animated: f32, games_dir: &str, font: Option<&Font>) {
    // Header title
    let title = "GameCube";
    let td = measure_text(title, font, 52, 1.0);
    let tx = (screen_width() - td.width) * 0.5;
    let ty = screen_height() * 0.10;
    draw_text_ex(title, tx + 2.0, ty + 2.0,
        TextParams { font, font_size: 52, color: Color::new(0.0, 0.0, 0.0, 0.5), ..Default::default() });
    draw_text_ex(title, tx, ty,
        TextParams { font, font_size: 52, color: WHITE, ..Default::default() });

    if games.is_empty() {
        let msg = format!("No games found in {}", games_dir);
        let d = measure_text(&msg, font, 28, 1.0);
        draw_text_ex(&msg, (screen_width() - d.width) * 0.5, screen_height() * 0.5,
            TextParams { font, font_size: 28, color: GRAY, ..Default::default() });
    } else {
        let idx = animated.round() as usize;
        if let Some(g) = games.get(idx) {
            let d = measure_text(&g.title, font, 38, 1.0);
            draw_text_ex(&g.title, (screen_width() - d.width) * 0.5, screen_height() * 0.88,
                TextParams { font, font_size: 38, color: WHITE, ..Default::default() });
            let counter = format!("{} / {}", idx + 1, games.len());
            let cd = measure_text(&counter, font, 18, 1.0);
            draw_text_ex(&counter, (screen_width() - cd.width) * 0.5, screen_height() * 0.93,
                TextParams { font, font_size: 18, color: Color::new(0.6, 0.6, 0.7, 1.0), ..Default::default() });
        }
    }
}

fn card_position(offset: f32) -> (f32, f32, f32) {
    let sign = if offset >= 0.0 { 1.0 } else { -1.0 };
    let abs = offset.abs();
    let x = if abs <= 1.0 { offset * CENTER_GAP } else { sign * (CENTER_GAP + (abs - 1.0) * SIDE_GAP) };
    let rot = sign * MAX_ROT * abs.min(1.0);
    let z = -(abs.min(3.0)) * 0.15;
    (x, rot, z)
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
    draw_mesh(&card_mesh(center, hw, hh, rot_y, 1.0, 1.0, false, Some(game.texture.clone())));
    let rc = Vec3::new(center.x, center.y - CARD_H - 0.003, center.z);
    draw_mesh(&card_mesh(rc, hw, hh, rot_y, 0.35, 0.0, true, Some(game.texture.clone())));
}

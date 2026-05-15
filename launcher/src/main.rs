mod banner;
mod game;
mod particles;
mod render;

use game::{load_games, Game};
use gilrs::{Axis, Button, EventType, Gilrs};
use macroquad::prelude::*;
use particles::{draw_particles, init_particles, update_particles};
use render::{draw_background, draw_carousel, draw_hud};
use std::env;
use std::path::Path;
use std::process::Command;

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

        if is_key_pressed(KeyCode::Left)   { nav -= 1; }
        if is_key_pressed(KeyCode::Right)  { nav += 1; }
        if is_key_pressed(KeyCode::Enter)  { launch = true; }
        if is_key_pressed(KeyCode::Escape) { break; }

        if let Some(g) = gilrs.as_mut() {
            while let Some(ev) = g.next_event() {
                match ev.event {
                    EventType::ButtonPressed(Button::DPadLeft, _)  => nav -= 1,
                    EventType::ButtonPressed(Button::DPadRight, _) => nav += 1,
                    EventType::ButtonPressed(Button::South, _)
                    | EventType::ButtonPressed(Button::East, _)    => launch = true,
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
                launch_game(g);
            }
        }

        let lerp = 1.0 - (-dt * 12.0).exp();
        animated += (target - animated) * lerp;

        clear_background(BLACK);
        draw_background();
        update_particles(&mut particles, dt);
        draw_particles(&particles);
        draw_carousel(&games, animated);
        draw_hud(&games, animated, &games_dir, font.as_ref());

        next_frame().await;
    }
}

fn launch_game(game: &Game) {
    if game.dolphin_entry {
        let _ = Command::new("dolphin-emu").status();
    } else {
        let _ = Command::new("dolphin-emu").args(["-b", "-e"]).arg(&game.path).status();
    }
}

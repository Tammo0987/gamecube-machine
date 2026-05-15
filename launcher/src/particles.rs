use macroquad::prelude::*;

pub struct Particle {
    pub x: f32,
    pub y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    base_alpha: f32,
    bokeh: bool,
}

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

pub fn init_particles() -> Vec<Particle> {
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

pub fn update_particles(particles: &mut Vec<Particle>, dt: f32) {
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

pub fn draw_particles(particles: &[Particle]) {
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
            draw_circle(px, py, p.radius, Color::new(0.55, 0.70, 1.0, alpha));
        } else {
            draw_circle(px, py, p.radius * 4.0, Color::new(0.55, 0.75, 1.0, alpha * 0.10));
            draw_circle(px, py, p.radius * 2.2, Color::new(0.65, 0.82, 1.0, alpha * 0.25));
            draw_circle(px, py, p.radius,       Color::new(0.88, 0.95, 1.0, alpha));
        }
    }
}

use std::collections::VecDeque;

use itertools::Itertools;
use nannou::prelude::*;

const ORBITAL_RADIUS: f32 = 1000.0;
const PARTICLES: usize = 16;
const HISTORY: usize = 200;
const VARY_VELOCITY: f32 = 100.0;

fn gm() -> f32 {
    ORBITAL_RADIUS * 57.0
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Debug, Clone)]
struct Particle {
    pos: Point2,
    vel: Vec2,
}

impl Particle {
    fn update(&mut self, delta_seconds: f32) {
        self.pos += self.vel * delta_seconds;
        let force = 1.0 / (self.pos.length() * 2.0);
        let gravity = -self.pos.normalize() + force * delta_seconds;
        self.vel += gravity;
    }
}

struct Record {
    pos: Point2,
    color: Hsla,
}

struct Model {
    particles: Vec<Particle>,
    colors: Vec<Hsla>,
    background: Hsl,
    circle_color: Hsl,
    history: VecDeque<Record>,
}

fn model(_app: &App) -> Model {
    let hue_start = random::<f32>();
    let hue_run = random_range(0.2, 0.4);
    let background_hue = random_range(0.0, 1.0);
    let background = hsl(background_hue, 0.38, 0.33);
    let circle_color = hsl(background_hue, 0.36, 0.33);

    let particles: Vec<Particle> = (0..PARTICLES)
        .map(|_| {
            let pos = point_on_circle() * ORBITAL_RADIUS;
            // on average velocity will be just enough to keep the particle circular orbit
            let speed = gm().sqrt();
            let speed = if random() { speed } else { -speed };
            let speed = speed + random_range(-VARY_VELOCITY, VARY_VELOCITY);
            let vel = speed * pt2(pos.y, -pos.x).normalize();
            Particle { pos, vel }
        })
        .collect();

    let linecount = particles.len();
    let colors = (0..linecount)
        .map(|i| {
            let hue = map_range(
                i as f32,
                0.0,
                (linecount - 1) as f32,
                hue_start,
                hue_start + hue_run,
            );
            let hue = hue % 1.0;
            hsla(hue, 0.5, 0.5, 0.5)
        })
        .collect();

    dbg!(hue_start, hue_run, background_hue);

    Model {
        particles,
        colors,
        background,
        circle_color,
        history: VecDeque::new(),
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    for particle in &mut model.particles {
        particle.update(update.since_last.as_secs_f32());
    }

    for (particle, color) in model.particles.iter().zip(model.colors.iter()) {
        model.history.push_front(Record {
            pos: particle.pos,
            color: tweak_color(color),
        });
    }
    model.history.truncate(HISTORY * PARTICLES);
}

fn tweak_color(c: &Hsla) -> Hsla {
    let mag = 0.008;
    let rr = || -> f32 { random_range(-mag, mag) };
    let hue = (c.hue.to_radians() / TAU + rr()) % 1.0;
    let sat = (c.saturation + rr()).clamp(0.0, 1.0);
    let light = (c.lightness + rr()).clamp(0.0, 1.0);
    hsla(hue, sat, light, c.alpha)
}

fn draw_history(history: &VecDeque<Record>, draw: &Draw) {
    if history.is_empty() {
        return;
    }

    let history_epochs = history.len() / PARTICLES;
    assert!(history.len() % PARTICLES == 0);

    fn idx(history: usize, particle: usize) -> usize {
        history * PARTICLES + particle
    }

    let verts = history
        .iter()
        .enumerate()
        .map(|(i, record)| (record.pos.extend((i / PARTICLES) as f32), record.color));
    let idxs = (0..PARTICLES)
        .tuple_windows()
        .flat_map(|(particle_a, particle_b)| {
            (0..history_epochs)
                .tuple_windows()
                .flat_map(move |(past, pres)| {
                    [
                        idx(past, particle_a),
                        idx(pres, particle_a),
                        idx(pres, particle_b),
                        idx(past, particle_a),
                        idx(pres, particle_b),
                        idx(past, particle_b),
                    ]
                })
        });

    draw.mesh().indexed_colored(verts, idxs);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let win = app.window_rect();
    // zoom out such that the entire window is visible
    let scale = win.w().min(win.h()) / 2.0 / ORBITAL_RADIUS / 1.1;

    let draw = app.draw().scale(scale);

    draw.background().color(model.background);
    // draw the average orbit, a circle
    draw.ellipse()
        .radius(ORBITAL_RADIUS)
        .color(model.circle_color);

    draw_history(&model.history, &draw);

    draw.to_frame(app, &frame).unwrap();
}

// come up with a random point on a sphere
fn point_on_circle() -> Point2 {
    loop {
        let x = random_range(-1.0, 1.0);
        let y = random_range(-1.0, 1.0);
        let len = x * x + y * y;
        if len != 0.0 {
            return pt2(x, y) / len.sqrt();
        }
    }
}

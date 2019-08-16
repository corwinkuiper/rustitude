use ggez::event::{EventHandler, KeyCode};
use ggez::graphics::{Color, DrawMode, DrawParam};
use ggez::input;
use ggez::nalgebra::Point2;
use ggez::*;

use std::collections::VecDeque;

use rand::prelude::*;

const GAME_SPEED: f32 = 150.0;
const WAVE_FRONT_FREQUENCY: f32 = 1.0;
const WAVE_FRONT_AMPLITUDE: f32 = 70.0;
const WAVE_FRONT_AMPLITUDE_SMALL: f32 = 20.0;
const WAVE_RADIUS: f32 = 16.0;

const LIFE_RECOVER: f32 = 10.0;
const LIFE_DEPLETE: f32 = 20.0;
const LIFE_MAXIMUM: f32 = 100.0;

const OBSTACLE_COUNTDOWN: f32 = 2.0;
const OBSTACLE_ANGLE_FREQUENCY: f32 = 1.0;

struct AmplitudeGameState {
    wave_front: Wave,
    wave_section: VecDeque<WaveSection>,
    time: f32,
    life: f32,
    obstacle: Obstacles,
    generator: rand::rngs::ThreadRng,
}

struct Obstacles {
    sprite: graphics::Image,
    objects: VecDeque<Obstacle>,
    countdown: f32,
}

struct Obstacle {
    x: f32,
    y: f32,
    angle: f32,
}

struct WaveSection {
    x: f32,
    y: f32,
    color: Color,
}

struct Wave {
    x: f32,
    y: f32,
}

impl AmplitudeGameState {
    fn new(mut ctx: &mut Context) -> AmplitudeGameState {
        let screen_size = graphics::screen_coordinates(&ctx);
        let sawblade_image = graphics::Image::new(&mut ctx, "/sawblade.png").unwrap();

        AmplitudeGameState {
            wave_front: Wave {
                x: screen_size.w / 8.0,
                y: screen_size.h / 2.0,
            },
            wave_section: VecDeque::new(),
            time: 0.0,
            life: LIFE_MAXIMUM,
            obstacle: Obstacles {
                objects: VecDeque::new(),
                sprite: sawblade_image,
                countdown: OBSTACLE_COUNTDOWN,
            },
            generator: thread_rng(),
        }
    }

    fn restart(&mut self, ctx: &mut Context) {
        let screen_size = graphics::screen_coordinates(&ctx);

        self.wave_front.x = screen_size.w / 8.0;
        self.wave_front.y = screen_size.h / 2.0;

        self.wave_section.clear();
        self.time = 0.0;
        self.life = LIFE_MAXIMUM;
        self.obstacle.objects.clear();
        self.obstacle.countdown = OBSTACLE_COUNTDOWN;
    }
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Amplitude", "Corwin")
        .window_setup(conf::WindowSetup::default().title("Amplitude"))
        .build()
        .expect("Could not create ggez context");

    let mut state = AmplitudeGameState::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

impl EventHandler for AmplitudeGameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx);
        let dt = (dt.as_nanos() as f32) / 1_000_000_000.0;

        let mut end_game = false;

        // update wave front
        let previous_sine_function =
            (self.time * 2.0 * std::f32::consts::PI * WAVE_FRONT_FREQUENCY).sin();
        let next_sine_function =
            ((self.time + dt) * 2.0 * std::f32::consts::PI * WAVE_FRONT_FREQUENCY).sin();

        let sine_function_difference = previous_sine_function - next_sine_function;

        // apply checks here to reduce amplitude

        let amplitude: f32;

        if input::keyboard::is_key_pressed(&ctx, KeyCode::Space) {
            if self.life > 0.0 {
                amplitude = WAVE_FRONT_AMPLITUDE_SMALL;
                self.life -= LIFE_DEPLETE * dt;
            } else {
                amplitude = WAVE_FRONT_AMPLITUDE;
            }
        } else {
            amplitude = WAVE_FRONT_AMPLITUDE;
            self.life += LIFE_RECOVER * dt;
        }

        if self.life > LIFE_MAXIMUM {
            self.life = LIFE_MAXIMUM;
        }

        let section_color: Color;
        if amplitude == WAVE_FRONT_AMPLITUDE {
            section_color = Color::new(1.0, 0.0, 0.0, 1.0)
        } else {
            section_color = Color::new(0.0, 0.0, 1.0, 1.0)
        }

        self.wave_front.y += sine_function_difference * amplitude;

        self.time += dt;

        // update wave back

        let new_wave_section = WaveSection {
            x: self.wave_front.x,
            y: self.wave_front.y,
            color: section_color,
        };
        for section in self.wave_section.iter_mut() {
            section.x -= dt * GAME_SPEED;
        }

        self.wave_section.push_back(new_wave_section);

        // obstacle update

        let sprite_size = self.obstacle.sprite.width() as f32 / 2.0;

        for o in self.obstacle.objects.iter_mut() {
            o.x -= dt * GAME_SPEED;
            o.angle -= 2.0 * std::f32::consts::PI * OBSTACLE_ANGLE_FREQUENCY * dt;

            if (o.x - self.wave_front.x).powi(2) < sprite_size.powi(2)
                && (o.y - self.wave_front.y).powi(2) < sprite_size.powi(2)
            {
                end_game = true;
            }
        }

        // add obstacle

        self.obstacle.countdown -= dt;
        if self.obstacle.countdown <= 0.0 {
            self.obstacle.countdown += OBSTACLE_COUNTDOWN;
            let screen_size = graphics::screen_coordinates(&ctx);
            let new_obstacle = Obstacle {
                x: screen_size.w + 32.0,
                y: self.generator.gen_range(0.0, screen_size.h),
                angle: self.generator.gen_range(0.0, 2.0 * std::f32::consts::PI),
            };
            self.obstacle.objects.push_back(new_obstacle);
        }

        // remove elements that are behind the screen
        loop {
            if let Some(section) = self.wave_section.get(0) {
                if section.x < -32.0 {
                    self.wave_section.pop_front();
                    continue;
                }
            }
            break;
        }

        loop {
            if let Some(obstacle) = self.obstacle.objects.get(0) {
                if obstacle.x < -32.0 {
                    self.obstacle.objects.pop_front();
                    continue;
                }
            }
            break;
        }

        if end_game {
            self.restart(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let mb = &mut graphics::MeshBuilder::new();

        let screen_size = graphics::screen_coordinates(&ctx);
        let width = screen_size.w;

        let mut bar_width = self.life / 100.0 * (width - 10.0);
        if bar_width < 0.0 {
            bar_width = 0.0;
        }

        for section in self.wave_section.iter() {
            mb.circle(
                DrawMode::fill(),
                Point2::new(section.x, section.y),
                WAVE_RADIUS,
                0.2,
                section.color,
            );
        }

        if !self.wave_section.is_empty() {
            if let Some(o) = self.wave_section.get(self.wave_section.len() - 1) {
                mb.rectangle(
                    DrawMode::fill(),
                    graphics::Rect::new(5.0, 5.0, bar_width, 16.0),
                    o.color,
                );
            }
        }

        for o in self.obstacle.objects.iter() {
            graphics::draw(
                ctx,
                &self.obstacle.sprite,
                DrawParam::new()
                    .rotation(o.angle)
                    .dest(Point2::new(o.x, o.y))
                    .offset(Point2::new(0.5, 0.5)),
            )?;
        }

        if let Ok(s) = mb.build(ctx) {
            graphics::draw(ctx, &s, DrawParam::new())?;
        }

        graphics::present(ctx)
    }
}

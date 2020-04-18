use ggez;

use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum PlayMode {
    Pending,
    Running,
    Lost,
    Won,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Bounce {
    Bottom,
    Top,
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, Debug)]
struct Block {
    i: i32,
    j: i32,
    rect: graphics::Rect,
    fill: graphics::Color,
    stroke: graphics::Color
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.j == other.j
    }
}
impl Eq for Block {}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.i.hash(state);
        self.j.hash(state);
    }
}

pub struct PlayState {
    help_text: graphics::Text,
    font: graphics::Font,
    mode: PlayMode,
    blocks: HashSet<Block>,
    paddle: f32,
    ball: (f32, f32),
    speed: (f32, f32),
    ball_speed: (f32, f32),
    score: u32,
}

impl PlayState {
    pub fn new(font: graphics::Font) -> Self {
        let mut blocks = HashSet::new();
        for i in 0..20 {
            for j in 0..5 {
                let minx = (i as f32) * BLOCK_WIDTH;
                let miny = (j as f32) * BLOCK_HEIGHT;
                let color = if i % 2 == j % 2 { GRAY } else { DARKGRAY };
                blocks.insert(Block {
                    i,
                    j,
                    rect: graphics::Rect::new(minx, miny, BLOCK_WIDTH, BLOCK_HEIGHT),
                    fill:color,
                    stroke:graphics::BLACK,
                });
            }
        }

        let help_text = graphics::Text::new(("Press <SPACE> to launch the ball", font, 18.0));

        Self {
            help_text,
            font,
            mode: PlayMode::Pending,
            blocks,
            paddle: WIDTH / 2.0,
            ball: (WIDTH / 2.0, 390.0),
            speed: (1.0, 1.0),
            ball_speed: (0.0, -BALL_SPEED),
            score: 0,
        }
    }

    fn draw_block(mb: &mut graphics::MeshBuilder, block: &Block) {
        mb.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            block.rect,
            block.fill,
        );
        mb.rectangle(
            graphics::DrawMode::Stroke(graphics::StrokeOptions::default()),
            block.rect,
            block.stroke,
        );
    }

    fn collision(&mut self) {
        let (bx, by) = self.ball;
        if bx - BALL_RADIUS <= 0.0 || bx + BALL_RADIUS >= WIDTH {
            self.ball_speed.0 = -self.ball_speed.0;
        } else if by - BALL_RADIUS <= 0.0 {
            self.ball_speed.1 = -self.ball_speed.1;
        } else if by + BALL_RADIUS > 420.0 {
            self.mode = PlayMode::Lost;
        } else if self.ball_speed.1 > 0.0
            && by + BALL_RADIUS > 400.0
            && bx + BALL_RADIUS >= self.paddle - PADDLE_WIDTH / 2.0 - 10.0
            && bx - BALL_RADIUS <= self.paddle + PADDLE_WIDTH / 2.0 + 10.0
        {
            let ratio = (bx - self.paddle) / 20.0;
            self.ball_speed.1 = -self.ball_speed.1;
            if ratio != 0.0 {
                self.ball_speed.0 = (self.ball_speed.0 + ratio)
                    .min(BALL_SPEED - 0.1)
                    .max(-BALL_SPEED + 0.1);
                self.ball_speed.1 =
                    -(BALL_SPEED * BALL_SPEED - self.ball_speed.0 * self.ball_speed.0).sqrt();
            }
        } else {
            let (bsx, bsy) = self.ball_speed;
            let mut bounce = Bounce::None;
            let mut sc = 0;
            let ball_rect = graphics::Rect::new(
                bx - BALL_RADIUS,
                by - BALL_RADIUS,
                BALL_RADIUS * 2.0,
                BALL_RADIUS * 2.0,
            );
            self.blocks.retain(|b| {
                if b.rect.overlaps(&ball_rect) {
                    if bsy < 0.0 && by > b.rect.y + b.rect.h {
                        bounce = Bounce::Bottom;
                    } else if bsy > 0.0 && by < b.rect.y {
                        bounce = Bounce::Top;
                    } else if bsx < 0.0 && bx > b.rect.x + b.rect.w {
                        bounce = Bounce::Right;
                    } else {
                        bounce = Bounce::Left;
                    }
                    sc += 1;
                    return false;
                }
                true
            });
            match bounce {
                Bounce::Bottom => {
                    self.ball_speed.1 = -self.ball_speed.1;
                }
                Bounce::Top => {
                    self.ball_speed.1 = -self.ball_speed.1;
                }
                Bounce::Left => {
                    self.ball_speed.0 = -self.ball_speed.0;
                }
                Bounce::Right => {
                    self.ball_speed.0 = -self.ball_speed.0;
                }
                _ => {}
            }
            self.score += sc;
            if self.blocks.is_empty() {
                self.mode = PlayMode::Won;
            }
        }
    }
}

impl event::EventHandler for PlayState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.mode == PlayMode::Running {
            self.ball.0 += self.ball_speed.0;
            self.ball.1 += self.ball_speed.1;
            self.collision();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, LIGHTGRAY);
        let mb = &mut graphics::MeshBuilder::new();
        for b in self.blocks.iter() {
            PlayState::draw_block(mb, b);
        }
        let rect = graphics::Rect::new(
            self.paddle - PADDLE_INNERWIDTH / 2.0,
            400.0,
            PADDLE_INNERWIDTH,
            PADDLE_HEIGHT,
        );

        mb.circle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            na::Point2::new(rect.x - 10.0, rect.y + PADDLE_HEIGHT / 2.0),
            10.0,
            0.1,
            DARKGRAY,
        );
        mb.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            graphics::Rect::new(rect.x - 10.0, rect.y, 10.0, PADDLE_HEIGHT),
            DARKGRAY,
        );

        mb.circle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            na::Point2::new(rect.x + PADDLE_WIDTH - 10.0, rect.y + PADDLE_HEIGHT / 2.0),
            10.0,
            0.1,
            DARKGRAY,
        );
        mb.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            graphics::Rect::new(rect.x + PADDLE_WIDTH - 20.0, rect.y, 10.0, PADDLE_HEIGHT),
            DARKGRAY,
        );

        mb.rectangle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            rect,
            LIGHTGRAY,
        );
        mb.rectangle(
            graphics::DrawMode::Stroke(graphics::StrokeOptions::default()),
            graphics::Rect::new(rect.x, rect.y + 1.0, rect.w, rect.h - 1.0),
            DARKGRAY,
        );

        if self.mode != PlayMode::Lost {
            mb.circle(
                graphics::DrawMode::Fill(graphics::FillOptions::default()),
                na::Point2::new(self.ball.0, self.ball.1),
                BALL_RADIUS,
                0.1,
                RED,
            );
            mb.circle(
                graphics::DrawMode::Stroke(graphics::StrokeOptions::default()),
                na::Point2::new(self.ball.0, self.ball.1),
                BALL_RADIUS,
                0.1,
                DARKGRAY,
            );
        }

        let m = mb.build(ctx)?;
        graphics::draw(ctx, &m, graphics::DrawParam::new())?;

        if self.mode == PlayMode::Pending {
            let (w, _h) = self.help_text.dimensions(ctx);
            let dest_point = na::Point2::new(WIDTH / 2.0 - (w as f32 / 2.0), 426.0);
            graphics::draw(ctx, &self.help_text, (dest_point, DARKGRAY))?;
        }

        let score_text = graphics::Text::new((format!("Score: {}",self.score), self.font, 18.0));
        graphics::draw(ctx, &score_text, (na::Point2::new(5.0,426.0), DARKGRAY))?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        repeat: bool,
    ) {
        match keycode {
            event::KeyCode::Space if self.mode == PlayMode::Pending => {
                self.mode = PlayMode::Running;
            }
            event::KeyCode::Left if self.mode != PlayMode::Lost => {
                self.speed.1 = 1.0;
                if self.paddle > LIMIT_LEFT {
                    if repeat {
                        self.speed.0 *= 1.2;
                    } else {
                        self.speed.0 = 1.0;
                    }
                    let mut delta = PADDLE_SPEED * self.speed.0;
                    self.paddle -= delta;
                    if self.paddle < LIMIT_LEFT {
                        delta -= LIMIT_LEFT - self.paddle;
                        self.paddle = LIMIT_LEFT;
                    }
                    if self.mode == PlayMode::Pending {
                        self.ball.0 -= delta;
                    }
                }
            }
            event::KeyCode::Right if self.mode != PlayMode::Lost => {
                self.speed.0 = 1.0;
                if self.paddle < LIMIT_RIGHT {
                    if repeat {
                        self.speed.1 *= 1.2;
                    } else {
                        self.speed.1 = 1.0;
                    }

                    let mut delta = PADDLE_SPEED * self.speed.1;
                    self.paddle += delta;
                    if self.paddle > LIMIT_RIGHT {
                        delta -= self.paddle - LIMIT_RIGHT;
                        self.paddle = LIMIT_RIGHT;
                    }
                    if self.mode == PlayMode::Pending {
                        self.ball.0 += delta;
                    }
                }
            }
            _ => (),
        }
    }
}

impl InnerState for PlayState {
    fn transition(&self, font: graphics::Font, keycode: event::KeyCode) -> Transition {
        if self.mode == PlayMode::Running && keycode == event::KeyCode::Space {
            return Transition::Push(Box::new(PauseState::new(font)));
        }
        Transition::None
    }

    fn state_transition(&self, font: graphics::Font) -> Transition {
        if self.mode == PlayMode::Lost {
            return Transition::Replace(Box::new(EndState::new("GAME OVER",font)));
        } else if self.mode == PlayMode::Won {
            return Transition::Replace(Box::new(EndState::new("YOU WIN!",font)));
        }
        Transition::None
    }
}

pub const BLOCK_WIDTH: f32 = 40.0;
pub const BLOCK_HEIGHT: f32 = 20.0;

pub const PADDLE_WIDTH: f32 = 80.0;
pub const PADDLE_HEIGHT: f32 = 20.0;

pub const PADDLE_INNERWIDTH: f32 = 60.0;

pub const LIMIT_LEFT: f32 = PADDLE_WIDTH / 2.0 + 10.0;
pub const LIMIT_RIGHT: f32 = WIDTH - PADDLE_WIDTH / 2.0 - 10.0;

const PADDLE_SPEED: f32 = 8.0;
const BALL_RADIUS: f32 = 10.0;
const BALL_SPEED: f32 = 3.0;

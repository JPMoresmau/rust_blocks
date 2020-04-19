//! Game play
use ggez;

use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use ggez::audio::{Source,SoundSource};

use super::*;
use crate::level::*;

// Different modes this play screen can be in
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum PlayMode {
    Pending,
    Running,
    Lost,
    Won,
}

// Bounce direction on a block
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Bounce {
    Bottom,
    Top,
    Left,
    Right,
    None,
}

// Full play state
pub struct PlayState {
    help_text: graphics::Text,    // help text
    font: graphics::Font,         // font for text
    block_sound: Option<Source>,  // sound when hitting a block
    paddle_sound: Option<Source>, // sound when hitting the paddle
    mode: PlayMode,  // current mode
    level: Level, // level definition
    paddle: f32, // paddle x position
    ball: (f32, f32), // ball position
    speed: (f32, f32), // paddle speed (left/right)
    ball_speed: (f32, f32), // ball speed vector
    score: u32, // user score
}

impl PlayState {
    // new play state, using level at given index
    pub fn new(font: graphics::Font, index: u32, score: u32) -> Self {
        let level = next_level(index);
        let speed = level.ball_speed;
        let help_text = graphics::Text::new(("Press <SPACE> to launch the ball", font, 18.0));
        
        Self {
            help_text,
            font,
            block_sound: Option::None,
            paddle_sound: Option::None,
            mode: PlayMode::Pending,
            level,
            paddle: WIDTH / 2.0,
            ball: (WIDTH / 2.0, 390.0),
            speed: (1.0, 1.0),
            ball_speed: (0.0, -speed),
            score,
        }
    }

    // draw a single block
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

    // calculate collision of ball with walls, blocks and paddle
    fn collision(&mut self) {
        let (bx, by) = self.ball;
        let ball_speed = self.level.ball_speed;
        // vertical wall collision
        if bx - BALL_RADIUS <= 0.0 || bx + BALL_RADIUS >= WIDTH {
            self.ball_speed.0 = -self.ball_speed.0;
        // top wall collision
        } else if by - BALL_RADIUS <= 0.0 {
            self.ball_speed.1 = -self.ball_speed.1;
        // ball falls below paddle, lost!
        } else if by + BALL_RADIUS > 420.0 {
            self.mode = PlayMode::Lost;
        // paddle collision
        } else if self.ball_speed.1 > 0.0
            && by + BALL_RADIUS > 400.0
            && bx + BALL_RADIUS >= self.paddle - PADDLE_WIDTH / 2.0 - 10.0
            && bx - BALL_RADIUS <= self.paddle + PADDLE_WIDTH / 2.0 + 10.0
        {
            if let Some(bs) = &mut self.paddle_sound {
                bs.play().unwrap_or_else(|e| println!("Cannot play sound:{}",e));
            }
            // calculate how to adapt the bounce according to the position of contact
            let ratio = (bx - self.paddle) / 20.0;
            self.ball_speed.1 = -self.ball_speed.1;
            if ratio != 0.0 {
                self.ball_speed.0 = (self.ball_speed.0 + ratio)
                    .min(ball_speed - 0.1)
                    .max(-ball_speed + 0.1);
                self.ball_speed.1 =
                    -(ball_speed * ball_speed - self.ball_speed.0 * self.ball_speed.0).sqrt();
            }
        } else {
            let (bsx, bsy) = self.ball_speed;
            let mut bounce = Bounce::None;
            let mut sc = 0;
            // approximate ball by rectangle
            let ball_rect = graphics::Rect::new(
                bx - BALL_RADIUS,
                by - BALL_RADIUS,
                BALL_RADIUS * 2.0,
                BALL_RADIUS * 2.0,
            );
            // calculate hit blocks
            self.level.blocks.retain(|b| {
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
            if sc>0{
                if let Some(bs) = &mut self.block_sound {
                    bs.play().unwrap_or_else(|e| println!("Cannot play sound:{}",e));
                }
            }
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
            if self.level.blocks.is_empty() {
                self.mode = PlayMode::Won;
            }
        }
    }
}

impl event::EventHandler for PlayState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // load sounds now that we have a context to do so
        if self.block_sound.is_none(){
            self.block_sound = Option::Some(Source::new(ctx, "/321585__waxxman__bip.mp3")?);
            self.paddle_sound = Option::Some(Source::new(ctx, "/399196__spiceprogram__perc-bip.wav")?);
        }
        // update ball and calculate collisions
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
        for b in self.level.blocks.iter() {
            PlayState::draw_block(mb, b);
        }
        // draw the paddle
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

        // draw the ball
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

        // draw help text
        if self.mode == PlayMode::Pending {
            let (w, _h) = self.help_text.dimensions(ctx);
            let dest_point = na::Point2::new(WIDTH / 2.0 - (w as f32 / 2.0), 426.0);
            graphics::draw(ctx, &self.help_text, (dest_point, DARKGRAY))?;
        }

        // draw score
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
                    // repeat press increases speed
                    if repeat {
                        self.speed.0 *= 1.2;
                    } else {
                        self.speed.0 = 1.0;
                    }
                    let mut delta = PADDLE_SPEED * self.speed.0;
                    self.paddle -= delta;
                    // stay in bounds
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
                     // repeat press increases speed
                    if repeat {
                        self.speed.1 *= 1.2;
                    } else {
                        self.speed.1 = 1.0;
                    }

                    let mut delta = PADDLE_SPEED * self.speed.1;
                    self.paddle += delta;
                    // stay in bounds
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
            return Transition::Replace(Box::new(PlayState::new(font,self.level.index+1,self.score)));
        }
        Transition::None
    }
}


pub const PADDLE_WIDTH: f32 = 80.0;
pub const PADDLE_HEIGHT: f32 = 20.0;

pub const PADDLE_INNERWIDTH: f32 = 60.0;

pub const LIMIT_LEFT: f32 = PADDLE_WIDTH / 2.0 + 10.0;
pub const LIMIT_RIGHT: f32 = WIDTH - PADDLE_WIDTH / 2.0 - 10.0;

const PADDLE_SPEED: f32 = 8.0;
const BALL_RADIUS: f32 = 10.0;


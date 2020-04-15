use ggez;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::nalgebra as na;
use std::time::{Duration, Instant};

use std::collections::HashSet;

use super::*;

pub struct PlayState {
    help_text: graphics::Text,
    started: bool,
    blocks: HashSet<(i32,i32)>,
    paddle: f32,
    ball: (f32,f32),
    speed: (f32, f32),
}

impl PlayState {
    pub fn new(font: graphics::Font) -> Self {
        let mut blocks=HashSet::new();
        for i in 0..20 {
            for j in 0..5 {
                blocks.insert((i,j ));
            }
        }
        let help_text = graphics::Text::new(("Press <SPACE> to launch the ball", font, 18.0));

        Self{help_text, started:false, blocks,paddle:WIDTH/2.0,ball:(WIDTH/2.0,390.0),speed:(1.0,1.0)}
    }

    fn draw_block(mb: &mut graphics::MeshBuilder, coords: (f32,f32), color: graphics::Color){
        let rect=graphics::Rect::new(coords.0,coords.1,BLOCK_WIDTH,BLOCK_HEIGHT);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), rect, color);
        mb.rectangle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), rect, graphics::BLACK);
        
    }
}

impl event::EventHandler for PlayState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, LIGHTGRAY);
        
        

        let mb = &mut graphics::MeshBuilder::new();
        for i in 0..20 {
            for j in 0..5 {
                if self.blocks.contains(&(i,j)){
                    let color = if i % 2 == j % 2 {GRAY} else {DARKGRAY};
                    PlayState::draw_block(mb, ((i as f32)*BLOCK_WIDTH,(j as f32)*BLOCK_HEIGHT), color);
                }
            }
        }
        let rect=graphics::Rect::new(self.paddle-PADDLE_INNERWIDTH/2.0,400.0,PADDLE_INNERWIDTH,PADDLE_HEIGHT);

        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(rect.x-10.0,rect.y+PADDLE_HEIGHT/2.0), 10.0, 0.1, DARKGRAY);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), graphics::Rect::new(rect.x-10.0,rect.y,10.0,PADDLE_HEIGHT), DARKGRAY);

        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(rect.x+PADDLE_WIDTH-10.0,rect.y+PADDLE_HEIGHT/2.0), 10.0, 0.1, DARKGRAY);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), graphics::Rect::new(rect.x+PADDLE_WIDTH-20.0,rect.y,10.0,PADDLE_HEIGHT), DARKGRAY);

        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), rect, LIGHTGRAY);
        mb.rectangle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), graphics::Rect::new(rect.x,rect.y+1.0,rect.w,rect.h-1.0), DARKGRAY);
       
        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(self.ball.0,self.ball.1), 10.0, 0.1, RED);
        mb.circle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), na::Point2::new(self.ball.0,self.ball.1), 10.0, 0.1, DARKGRAY);
        
        let m = mb.build(ctx)?;
        graphics::draw(ctx, &m, graphics::DrawParam::new())?;

        if !self.started {
            let (w,_h)=self.help_text.dimensions(ctx);
            let dest_point = na::Point2::new(WIDTH/2.0-(w as f32 / 2.0), 426.0);
            graphics::draw(ctx, &self.help_text, (dest_point,DARKGRAY))?;
        }


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
            event::KeyCode::Space if !self.started => {
                self.started=true;
            },
            event::KeyCode::Left => {
                self.speed.1=1.0;
                if self.paddle>LIMIT_LEFT {
                    if repeat {
                        self.speed.0 *= 1.2;
                    } else {
                        self.speed.0 = 1.0;
                    }
                    
                    let mut delta = 5.0 * self.speed.0;
                    self.paddle -= delta;
                    if self.paddle<LIMIT_LEFT {
                        delta -= LIMIT_LEFT-self.paddle;
                        self.paddle=LIMIT_LEFT;
                    }
                    if !self.started {
                        self.ball.0 -= delta;
                    }
                    
                }
            },
            event::KeyCode::Right => {
                self.speed.0=1.0;
                if self.paddle<LIMIT_RIGHT {
                    if repeat {
                        self.speed.1 *= 1.2;
                    } else {
                        self.speed.1 = 1.0;
                    }

                    let mut delta = 5.0 * self.speed.1;
                    self.paddle += delta;
                    if self.paddle>LIMIT_RIGHT {
                        delta -= self.paddle-LIMIT_RIGHT;
                        self.paddle=LIMIT_RIGHT;
                    }
                    if !self.started {
                        self.ball.0 += delta;
                    }
                    
                }
            },
            _ => (),
        }
        
    }
}


impl InnerState for PlayState {
    fn transition(&self, font: graphics::Font, keycode: event::KeyCode) -> Transition {
        if self.started && keycode == event::KeyCode::Space {
            return Transition::Push(Box::new(PauseState::new(font)));
        }
        Transition::None
    }
}

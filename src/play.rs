use ggez;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::nalgebra as na;
use std::time::{Duration, Instant};

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum PlayMode {
    Pending,
    Running,
    Lost,
}

#[derive(Clone, Copy, Debug)]
struct Brick {
    i: i32,
    j: i32,
    rect: graphics::Rect,
}

impl PartialEq for Brick {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.j == other.j
    }
}
impl Eq for Brick {}

impl Hash for Brick {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.i.hash(state);
        self.j.hash(state);
    }
}

pub struct PlayState {
    help_text: graphics::Text,
    mode: PlayMode,
    blocks: HashSet<Brick>,
    paddle: f32,
    ball: (f32,f32),
    speed: (f32, f32),
    ball_speed: (f32, f32),
}

impl PlayState {
    pub fn new(font: graphics::Font) -> Self {
        let mut blocks=HashSet::new();
        for i in 0..20 {
            for j in 0..5 {
                let minx=(i as f32)*BLOCK_WIDTH;
                let miny=(j as f32)*BLOCK_HEIGHT;
                blocks.insert(Brick{i,j,rect:graphics::Rect::new(minx,miny,BLOCK_WIDTH,BLOCK_HEIGHT )});
            }
        }
        let help_text = graphics::Text::new(("Press <SPACE> to launch the ball", font, 18.0));

        Self{help_text, mode: PlayMode::Pending, blocks,paddle:WIDTH/2.0,ball:(WIDTH/2.0,390.0),
                speed:(1.0,1.0), ball_speed:(0.0,-3.0)}
    }

    fn draw_block(mb: &mut graphics::MeshBuilder, coords: (f32,f32), color: graphics::Color){
        let rect=graphics::Rect::new(coords.0,coords.1,BLOCK_WIDTH,BLOCK_HEIGHT);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), rect, color);
        mb.rectangle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), rect, graphics::BLACK);
        
    }

    fn collision(&mut self) {
        let (bx,by) = self.ball;
        if bx-BALL_RADIUS<=0.0 || bx+BALL_RADIUS>=WIDTH {
            self.ball_speed.0=-self.ball_speed.0;
        } else if by-BALL_RADIUS<=0.0 {
            self.ball_speed.1=-self.ball_speed.1;
        } else if by+BALL_RADIUS>410.0 {
            self.mode=PlayMode::Lost;
        } else if by+BALL_RADIUS>400.0 && bx-BALL_RADIUS>=self.paddle-PADDLE_WIDTH/2.0-10.0 && bx+BALL_RADIUS<=self.paddle+PADDLE_WIDTH/2.0+10.0 {
            let ratio = (bx-self.paddle)/10.0;
            self.ball_speed.0=self.ball_speed.0+ratio;
            self.ball_speed.1=-self.ball_speed.1;
            //println!("ratio: {}, ball_speed:{:?}, sum:{}",ratio,self.ball_speed,self.ball_speed.0+self.ball_speed.1);
        } else {
            let mut rem=vec![];
            for b in self.blocks.iter() {
                let miny=b.rect.y;
                let maxy= miny+b.rect.h;
                let minx=b.rect.x;
                let maxx=minx+b.rect.w;
                
                let ydiff = by-BALL_RADIUS-maxy;
                
                if bx>=minx && bx<=maxx && ydiff<=0.0 && ydiff>self.ball_speed.1 && self.ball_speed.1<0.0{
                    self.ball_speed.1=-self.ball_speed.1;
                    rem.push(b.clone());
                } 
            }
            for r in rem.iter() {
                self.blocks.remove(r);
            }
        }
    }


}

impl event::EventHandler for PlayState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.mode==PlayMode::Running {
            //if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            //    self.last_update = Instant::now();
                self.ball.0+=self.ball_speed.0;
                self.ball.1+=self.ball_speed.1;
                self.collision();
            //}
               
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, LIGHTGRAY);
                

        let mb = &mut graphics::MeshBuilder::new();
        for b in self.blocks.iter() {
            let color = if b.i % 2 == b.j % 2 {GRAY} else {DARKGRAY};
            PlayState::draw_block(mb, (b.rect.x,b.rect.y), color);
        }
        let rect=graphics::Rect::new(self.paddle-PADDLE_INNERWIDTH/2.0,400.0,PADDLE_INNERWIDTH,PADDLE_HEIGHT);

        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(rect.x-10.0,rect.y+PADDLE_HEIGHT/2.0), 10.0, 0.1, DARKGRAY);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), graphics::Rect::new(rect.x-10.0,rect.y,10.0,PADDLE_HEIGHT), DARKGRAY);

        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(rect.x+PADDLE_WIDTH-10.0,rect.y+PADDLE_HEIGHT/2.0), 10.0, 0.1, DARKGRAY);
        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), graphics::Rect::new(rect.x+PADDLE_WIDTH-20.0,rect.y,10.0,PADDLE_HEIGHT), DARKGRAY);

        mb.rectangle(graphics::DrawMode::Fill(graphics::FillOptions::default()), rect, LIGHTGRAY);
        mb.rectangle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), graphics::Rect::new(rect.x,rect.y+1.0,rect.w,rect.h-1.0), DARKGRAY);
       
        mb.circle(graphics::DrawMode::Fill(graphics::FillOptions::default()), na::Point2::new(self.ball.0,self.ball.1), BALL_RADIUS, 0.1, RED);
        mb.circle(graphics::DrawMode::Stroke(graphics::StrokeOptions::default()), na::Point2::new(self.ball.0,self.ball.1), BALL_RADIUS, 0.1, DARKGRAY);
        
        let m = mb.build(ctx)?;
        graphics::draw(ctx, &m, graphics::DrawParam::new())?;

        if self.mode==PlayMode::Pending {
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
            event::KeyCode::Space if self.mode==PlayMode::Pending => {
                self.mode = PlayMode::Running;
            },
            event::KeyCode::Left if self.mode!=PlayMode::Lost => {
                self.speed.1=1.0;
                if self.paddle>LIMIT_LEFT {
                    if repeat {
                        self.speed.0 *= 1.2;
                    } else {
                        self.speed.0 = 1.0;
                    }
                    
                    let mut delta = PADDLE_SPEED * self.speed.0;
                    self.paddle -= delta;
                    if self.paddle<LIMIT_LEFT {
                        delta -= LIMIT_LEFT-self.paddle;
                        self.paddle=LIMIT_LEFT;
                    }
                    if self.mode==PlayMode::Pending {
                        self.ball.0 -= delta;
                    }
                    
                }
            },
            event::KeyCode::Right if self.mode!=PlayMode::Lost => {
                self.speed.0=1.0;
                if self.paddle<LIMIT_RIGHT {
                    if repeat {
                        self.speed.1 *= 1.2;
                    } else {
                        self.speed.1 = 1.0;
                    }

                    let mut delta = PADDLE_SPEED * self.speed.1;
                    self.paddle += delta;
                    if self.paddle>LIMIT_RIGHT {
                        delta -= self.paddle-LIMIT_RIGHT;
                        self.paddle=LIMIT_RIGHT;
                    }
                    if self.mode==PlayMode::Pending {
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
        if self.mode==PlayMode::Running && keycode == event::KeyCode::Space {
            return Transition::Push(Box::new(PauseState::new(font)));
        }
        Transition::None
    }
}

pub const BLOCK_WIDTH: f32= 40.0;
pub const BLOCK_HEIGHT: f32= 20.0;

pub const PADDLE_WIDTH: f32= 80.0;
pub const PADDLE_HEIGHT: f32= 20.0;

pub const PADDLE_INNERWIDTH: f32= 60.0;

pub const LIMIT_LEFT:f32 = PADDLE_WIDTH/2.0+10.0;
pub const LIMIT_RIGHT:f32 =WIDTH-PADDLE_WIDTH/2.0-10.0;

const PADDLE_SPEED: f32 = 8.0;
const BALL_RADIUS: f32= 10.0;

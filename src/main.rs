//! Basic hello world example.

use ggez;

use ggez::event;
use ggez::graphics;
use ggez::conf;
use ggez::{Context, GameResult};
use ggez::nalgebra as na;
use std::env;
use std::path;
use std::time::{Duration, Instant};

mod play;
use play::PlayState;

enum Transition {
    Push(Box<dyn InnerState>),
    Replace(Box<dyn InnerState>),
    Pop,
    None,
}

trait InnerState : event::EventHandler {
    fn transition(&self, font: graphics::Font,  keycode: event::KeyCode) -> Transition;
}

struct StartState {
    title_text: graphics::Text,
    start_text: graphics::Text,
    last_update: Instant,
    show_start: bool,
}

impl StartState {
    fn new(font: graphics::Font) -> Self {
        let title_text = graphics::Text::new(("BLOCKS", font, 72.0));
        let start_text = graphics::Text::new(("Press <SPACE> to start", font, 36.0));
        Self{title_text, start_text, last_update: Instant::now(), show_start: true}
    }   
}

impl event::EventHandler for StartState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
       
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.show_start = !self.show_start;
            self.last_update = Instant::now();
        }
           
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, LIGHTGRAY);
        let (w,_h)=self.title_text.dimensions(ctx);
        let dest_point = na::Point2::new(WIDTH/2.0-(w as f32 / 2.0), 100.0);
        graphics::draw(ctx, &self.title_text, (dest_point,RED))?;
        if self.show_start {
            let (w,_h)=self.start_text.dimensions(ctx);
            let dest_point = na::Point2::new(WIDTH/2.0-(w as f32 / 2.0), 180.0);
            graphics::draw(ctx, &self.start_text, (dest_point,graphics::BLACK))?;
        }

        Ok(())
    }
}

impl InnerState for StartState {
    fn transition(&self, font: graphics::Font, keycode: event::KeyCode) -> Transition {
        if keycode == event::KeyCode::Space {
            return Transition::Replace(Box::new(PlayState::new(font)));
        }
        Transition::None
    }
}


struct PauseState {
    title_text: graphics::Text,
    restart_text: graphics::Text,
}

impl PauseState {
    fn new(font: graphics::Font) -> Self {
        let title_text = graphics::Text::new(("Game Paused", font, 36.0));
        let restart_text = graphics::Text::new(("Press <SPACE> to resume", font, 25.0));
        Self{title_text, restart_text}
    }   
}

impl event::EventHandler for PauseState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
       
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (w,_h)=self.title_text.dimensions(ctx);
        let dest_point = na::Point2::new(WIDTH/2.0-(w as f32 / 2.0), 100.0);
        graphics::draw(ctx, &self.title_text, (dest_point,RED))?;
        let (w,_h)=self.restart_text.dimensions(ctx);
        let dest_point = na::Point2::new(WIDTH/2.0-(w as f32 / 2.0), 180.0);
        graphics::draw(ctx, &self.restart_text, (dest_point,graphics::BLACK))?;
        Ok(())
    }
}


impl InnerState for PauseState {
    fn transition(&self, _font: graphics::Font, keycode: event::KeyCode) -> Transition {
        if keycode == event::KeyCode::Space {
            return Transition::Pop;
        }
        Transition::None
    }
}



struct MainState {
    frames: usize,
    font: graphics::Font,
    inner_state: Vec<Box<dyn InnerState>>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let font = graphics::Font::new(ctx, "/PixelEmulator-xq08.ttf")?;
        
        let s = MainState { frames: 0,font, inner_state: vec![Box::new(StartState::new(font))]};
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.inner_state.last_mut().unwrap().update(ctx)?;
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        
        self.inner_state.last_mut().unwrap().draw(ctx)?;

        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        keymod: event::KeyMods,
        repeat: bool,
    ) {
        let tr=self.inner_state.last_mut().unwrap().transition(self.font, keycode);
        match tr {
            Transition::Replace(ns) => {
                self.inner_state.pop();
                self.inner_state.push(ns);
            },
            Transition::Pop => {
                self.inner_state.pop();
            },
            Transition::Push(ns) => {
                self.inner_state.push(ns);
            },
            _ => {
                self.inner_state.last_mut().unwrap().key_down_event(ctx,keycode,keymod,repeat);
            },
        }
        
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    let ws=conf::WindowSetup {
             title: "Blocks".to_owned(),
             samples: conf::NumSamples::Zero,
             vsync: true,
             icon: "".to_owned(),
             srgb: true,
         };
    let wm = conf::WindowMode::default().dimensions(WIDTH,HEIGHT);

    let cb = ggez::ContextBuilder::new("blocks", "JP Moresmau")
        .window_setup(ws)
        .window_mode(wm)
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}

const UPDATES_PER_SECOND: f32 = 2.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

pub const WIDTH: f32= 800.0;
pub const HEIGHT: f32= 450.0;



pub const LIGHTGRAY: graphics::Color = graphics::Color {
    r: 0.83,
    g: 0.83,
    b: 0.83,
    a: 1.0,
};

pub const GRAY: graphics::Color = graphics::Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

pub const DARKGRAY: graphics::Color = graphics::Color {
    r: 0.33,
    g: 0.33,
    b: 0.33,
    a: 1.0,
};

pub const RED: graphics::Color = graphics::Color {
    r: 0.83,
    g: 0.13,
    b: 0.18,
    a: 1.0,
};

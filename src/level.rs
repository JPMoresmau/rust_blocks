//! Define different levels

use ggez::graphics;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::*;

// A single block
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub i: i32,                  // X position in grid
    pub j: i32,                  // Y position in grid
    pub rect: graphics::Rect,    // position in pixel
    pub fill: graphics::Color,   // fill color
    pub stroke: graphics::Color, // stroke color
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

// Level definition
pub struct Level {
    pub index: u32,             // level number
    pub blocks: HashSet<Block>, // block position
    pub ball_speed: f32,        // speed of the ball
}

// Get the next level, based on the index
pub fn next_level(ix: u32) -> Level {
    let mut blocks = HashSet::new();

    let mut speed = BALL_SPEED;

    let num_levels = 3;

    // get the level disposition
    let lvl = ix % num_levels;
    // speed increases when we go back to first level
    let multi = ix / num_levels;
    if multi > 1 {
        speed *= multi as f32;
    }

    // build proper level
    match lvl {
        0 => level_1(&mut blocks),
        1 => level_2(&mut blocks),
        3 => level_3(&mut blocks),
        _ => panic!("No level for {}", lvl),
    }

    Level {
        index: ix,
        blocks,
        ball_speed: speed,
    }
}

// Level 1
fn level_1(blocks: &mut HashSet<Block>) {
    for j in 0..3 {
        for x in 0..(5 - j * 2) {
            let i = 8 + j + x;
            let minx = (i as f32) * BLOCK_WIDTH;
            let miny = (j as f32) * BLOCK_HEIGHT;
            let color = if i % 2 == j % 2 { GRAY } else { DARKGRAY };
            blocks.insert(Block {
                i,
                j,
                rect: graphics::Rect::new(minx, miny, BLOCK_WIDTH, BLOCK_HEIGHT),
                fill: color,
                stroke: graphics::BLACK,
            });
        }
    }
}

// Level 2
fn level_2(blocks: &mut HashSet<Block>) {
    for (i, j) in &[
        (10, 0),
        (9, 1),
        (11, 1),
        (8, 2),
        (12, 2),
        (9, 3),
        (11, 3),
        (10, 4),
    ] {
        let minx = (*i as f32) * BLOCK_WIDTH;
        let miny = (*j as f32) * BLOCK_HEIGHT;
        let color = if i % 2 == j % 2 { GRAY } else { DARKGRAY };
        blocks.insert(Block {
            i:*i,
            j:*j,
            rect: graphics::Rect::new(minx, miny, BLOCK_WIDTH, BLOCK_HEIGHT),
            fill: color,
            stroke: graphics::BLACK,
        });
    }
}

// Level 3
fn level_3(blocks: &mut HashSet<Block>) {
    for i in 0..20 {
        for j in 0..5 {
            let minx = (i as f32) * BLOCK_WIDTH;
            let miny = (j as f32) * BLOCK_HEIGHT;
            let color = if i % 2 == j % 2 { GRAY } else { DARKGRAY };
            blocks.insert(Block {
                i,
                j,
                rect: graphics::Rect::new(minx, miny, BLOCK_WIDTH, BLOCK_HEIGHT),
                fill: color,
                stroke: graphics::BLACK,
            });
        }
    }
}

// Initial ball speed
const BALL_SPEED: f32 = 5.0;

// Default lock width
const BLOCK_WIDTH: f32 = 40.0;
// Default block height
const BLOCK_HEIGHT: f32 = 20.0;

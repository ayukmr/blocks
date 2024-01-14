use crate::instance::Instance;
use crate::vertex::TEX_H;

use noise::permutationtable::PermutationTable;
use noise::core::open_simplex::open_simplex_2d;

use std::cmp::Ordering;

// chunk size
pub const CHUNK_SIZE: u8 = 16;

// block type
enum Block {
    Air,
    Dirt,
    Grass,
}

impl Block {
    // check if air
    fn is_air(&self) -> bool {
        matches!(self, Self::Air)
    }

    // texture mapping
    fn texture(&self) -> f32 {
        let pos = match self {
            Self::Air   => unreachable!(),
            Self::Dirt  => 2.0,
            Self::Grass => 3.0,
        };

        pos * TEX_H
    }
}

// chunk
pub struct Chunk {
    // position
    pos_x: i32,
    pos_z: i32,

    // blocks
    blocks: Vec<Vec<Vec<Block>>>,
}

impl Chunk {
    // create chunk
    pub fn new(pos_x: i32, pos_z: i32, hashers: &[(u8, PermutationTable)]) -> Self {
        let off_x = pos_x * CHUNK_SIZE as i32;
        let off_z = pos_z * CHUNK_SIZE as i32;

        // get hashers sum
        let divisor =
            hashers
                .iter()
                .map(|(n, _)| 1 / n)
                .sum::<u8>() as f64;

        let blocks = (0..CHUNK_SIZE).map(|z| {
            (0..CHUNK_SIZE).map(move |x| {
                // sample noise for height
                let raw_height =
                    hashers
                        .iter()
                        .map(|(n, hasher)| {
                            open_simplex_2d([
                                ((x as f32 + off_x as f32) * *n as f32 * 0.025) as f64,
                                ((z as f32 + off_z as f32) * *n as f32 * 0.025) as f64,
                            ], hasher) / *n as f64
                        }).sum::<f64>() / divisor;

                let height = ((raw_height + 1.0) * 15.0).powf(0.9) as i32;

                (0..64).map(move |y| {
                    // show dirt if below height
                    match height.cmp(&y) {
                        Ordering::Equal   => Block::Grass,
                        Ordering::Greater => Block::Dirt,
                        _ => Block::Air
                    }
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        Self { pos_x, pos_z, blocks }
    }

    // get instances
    pub fn instances(&self) -> Vec<Instance> {
        // simplify blocks
        let blocks = self.blocks.iter().enumerate().flat_map(|(z, row)| {
            row.iter().enumerate().flat_map(move |(x, col)| {
                col.iter().enumerate().map(move |(y, block)| {
                    ((x, y, z), block)
                })
            })
        });

        // get instances
        blocks
            .filter(|(_, block)| !block.is_air())
            .map(|((x, y, z), block)| {
                Instance::new(
                    // position
                    [
                        x as f32 + (self.pos_x * CHUNK_SIZE as i32) as f32,
                        y as f32 - 7.5,
                        z as f32 + (self.pos_z * CHUNK_SIZE as i32) as f32,
                    ],

                    // get texture
                    block.texture(),

                    // render faces
                    [
                        // front and back
                        z + 1 == self.blocks.len() || self.blocks[z + 1][x][y].is_air(),
                        z == 0 || self.blocks[z - 1][x][y].is_air(),

                        // left and right
                        x == 0 || self.blocks[z][x - 1][y].is_air(),
                        x + 1 == self.blocks[z].len() || self.blocks[z][x + 1][y].is_air(),

                        // top and bottom
                        y + 1 == self.blocks[z][x].len() || self.blocks[z][x][y + 1].is_air(),
                        y == 0 || self.blocks[z][x][y - 1].is_air(),
                    ]
                )
            }).collect()
    }
}

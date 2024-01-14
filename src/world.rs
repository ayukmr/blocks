use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::instance::Instance;

use noise::permutationtable::PermutationTable;

use std::collections::HashMap;
use rayon::prelude::*;

const CHUNKS: u8 = 8;

// world instances
pub struct World {
    // chunks
    chunks: HashMap<(i32, i32), Chunk>,

    // simplex hashers
    hashers: Vec<(u8, PermutationTable)>,

    // loaded chunks
    loaded_x: i32,
    loaded_z: i32,
}

impl World {
    // create world
    pub fn new() -> Self {
        // create hashers
        let hashers = [1, 2, 4, 8, 16]
            .iter()
            .map(|&step| (step, PermutationTable::new(rand::random())))
            .collect::<Vec<(u8, PermutationTable)>>();

        Self {
            hashers,
            chunks: HashMap::new(),

            loaded_x: 0,
            loaded_z: 0,
        }
    }

    // check if refresh is required
    pub fn refresh_required(&self, pos_x: i32, pos_z: i32) -> bool {
        let chunk_x = pos_x / CHUNK_SIZE as i32;
        let chunk_z = pos_z / CHUNK_SIZE as i32;

        // in loaded chunks
        chunk_x < self.loaded_x - 3 ||
           chunk_x > self.loaded_x + 3 ||
           chunk_z < self.loaded_z - 3 ||
           chunk_z > self.loaded_z + 3
    }

    // load chunk
    pub fn load(&mut self, pos_x: i32, pos_z: i32) {
        let chunk = Chunk::new(pos_x, pos_z, &self.hashers);
        self.chunks.insert((pos_x, pos_z), chunk);
    }

    // get instances
    pub fn instances(&mut self, pos_x: i32, pos_z: i32) -> Vec<Instance> {
        // movement direction
        let x_dir = ((pos_x / CHUNK_SIZE as i32) - self.loaded_x).signum();
        let z_dir = ((pos_z / CHUNK_SIZE as i32) - self.loaded_z).signum();

        // chunks center
        let chunk_x = self.loaded_x + x_dir;
        let chunk_z = self.loaded_z + z_dir;

        self.loaded_x = chunk_x;
        self.loaded_z = chunk_z;

        let chunks =
            itertools::iproduct!(
                (chunk_x - CHUNKS as i32..=chunk_x + CHUNKS as i32),
                (chunk_z - CHUNKS as i32..=chunk_z + CHUNKS as i32)
            );

        for (x, z) in chunks.clone() {
            // load if not available
            if !self.chunks.contains_key(&(x, z)) {
                self.load(x, z);
            }
        }

        // get instances
        chunks
            .par_bridge()
            .flat_map(|(x, z)| {
                self.chunks.get(&(x, z)).unwrap().instances()
            }).collect()
    }
}

impl Default for World {
    // default world
    fn default() -> Self {
        Self::new()
    }
}

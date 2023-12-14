use std::error::Error;
use std::ops::{Index, IndexMut};

const MEMORY: usize = 4096;   // 4KB
const LOWER_MEMORY_BOUNDARY: usize = 512;
pub struct Memory {
    memory: [u8; MEMORY],

}

impl Memory {

    pub fn default() -> Self {
        Memory{memory: [0x0;4096]}
    }
    // pub fn load_into_memory(&mut self, instructions: [u8; 6]) {
    //     for (idx, instruction) in instructions.iter().copied().enumerate() {
    //         self.memory[LOWER_MEMORY_BOUNDARY + idx] = instruction
    //     }
    // }

    pub fn load_program(&mut self, program: Vec<u8>) -> Result<(), Box<dyn Error>> {
        for (idx, instruction) in program.iter().copied().enumerate() {
            self.memory[LOWER_MEMORY_BOUNDARY + idx] = instruction
        }
        Ok(())
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.memory[index  as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.memory.get_mut(index as usize).unwrap()
    }
}
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

const MEMORY: usize = 4096;   // 4KB
const LOWER_MEMORY_BOUNDARY: usize = 512;

#[derive(Debug)]
struct ProgramTooLargeError;

impl Display for ProgramTooLargeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Provided program is too large to fit into memory.")
    }
}


impl Error for ProgramTooLargeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct Memory {
    memory: [u8; MEMORY],
}

impl Memory {

    pub fn default() -> Self {
        Memory{memory: [0x0;MEMORY]}
    }

    pub fn load_program(&mut self, program: Vec<u8>) -> Result<(), Box<dyn Error>> {
        if program.len() + LOWER_MEMORY_BOUNDARY > MEMORY {
            return Err(Box::new(ProgramTooLargeError));
        }

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
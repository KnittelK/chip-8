use std::error::Error;
use rand::random;
use crate::memory::Memory;
use crate::screen::Screen;
use crate::keyboard::Keypad;

const REGISTER: usize = 16;
const STACK: usize = 16;
const LOWER_MEMORY_BOUNDARY: u16 = 512;

static CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    memory: Memory,                 // 4KB of memory
    pc: u16,                      // program counter
    i: u16,                       // index register
    register: [u8; REGISTER],      // register of size 16
    stack: [u16; STACK],            // program stack
    sp: usize,                      // stack pointer
    delay_timer: u8,
    sound_timer: u8,

    // ** Peripherals **

    // graphics
    screen: Screen,

    // Keyboard
    keyboard: Keypad,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory =  Memory::default();
        for i in 0..79 {
            memory[i] = CHIP8_FONTSET[i as usize];
        }

        Chip8 {
            memory,
            i: 0,
            pc: LOWER_MEMORY_BOUNDARY,
            register: [0; REGISTER],
            stack: [0; STACK],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::default(),
            keyboard: Keypad::default(),
        }
    }

    pub fn populate_register(&mut self, data: Vec<u8>) {
        for (idx, value) in data.iter().enumerate() {
            self.register[idx] = *value
        }
    }

    pub fn load_into_memory(&mut self, program: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.memory.load_program(program)
    }
    pub fn get_value_at_register_addr(&self, addr: u8) -> Option<u8> {
        self.register.get(addr as usize).copied()
    }

    fn read_opcode(&self) -> u16 {
        let high_byte = self.memory[self.pc] as u16;
        let low_byte = self.memory[self.pc + 1] as u16;

        /*
        since opcodes take up 2 bytes of memory, and each element within our memory only has 1 byte of information,
        we need:
            To take the first entry (the high byte) and cast it as u16.
            Then shift the High Byte by 1 byte to the left.
            Following that, we take the next entry (the Low Byte) in our memory
                and, using an OR operation, combine the 2 bytes to form the 2 byte OPCode.

         */
        high_byte << 8 | low_byte
    }

    fn call_fn_at_addr(&mut self, addr: u16) {
        let sp = self.sp;
        let stack = self.stack;
        if sp > stack.len() {
            panic!("Stack overflow!");
        }

        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn return_from_fn_call(&mut self) {
        if self.sp == 0 {
            panic!("Stack underflow!");
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    pub fn execute_instruction(&mut self, opcode: u16) {
            let opcode_group = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            let n = (opcode & 0x000F) as u8;
            let nnn = opcode & 0x0FFF;
            let nn = (opcode & 0x00FF) as u8;

            match (opcode_group, x, y, n) {
                (0, 0, 0, 0) => {
                    return
                },
                // 0x00E0
                (0x0, 0x0, 0xE, 0x0) => {
                    self.screen.clear_screen();
                }
                (0, 0, 0xE, 0xE) => {
                    self.return_from_fn_call();
                },
                (0x1, _, _, _) => {
                    self.set_pc_to_addr(nnn);
                    return
                },
                (0x2, _, _, _) => {
                    self.call_fn_at_addr(nnn);
                    // skip incrementing program counter.
                    return
                },
                (0x3, _, _, _) => {
                    // Skip the next instruction if register VX is equal to NN
                    if self.register[x] == nn {
                        self.pc += 2;
                    }
                }
                (0x4, _, _, _) => {
                    // Skip the next instruction if register VX is not equal to NN.
                    if self.register[x] != nn {
                        self.pc += 2;
                    }
                }
                (0x5, _, _, 0x0) => {
                    // Skip the next instruction if register VX equals VY.
                    if self.register[x] == self.register[y] {
                        self.pc += 2;
                    }
                }
                (0x6, _, _, _) => {
                    // Load immediate value NN into register VX.
                    self.register[x] = nn;
                }
                (0x7, _, _, _) => {
                    // Add immediate value NN to register VX. Does not effect VF.
                    self.register[x] = ((self.register[x] as u16 + nn as u16) & 0xff) as u8;
                }
                (0x8, _, _, _) => {
                    match n {
                        0x0 => {
                            // Copy the value in register VY into VX
                            self.register[x] = self.register[y];
                        }
                        0x1 => {
                            // Set VX equal to the bitwise or of the values in VX and VY.
                            let vx = self.register[x];
                            let vy = self.register[y];
                            self.register[x] = vx | vy
                        }
                        0x2 => {
                            // Set VX equal to the bitwise and of the values in VX and VY.
                            let vx = self.register[x];
                            let vy = self.register[y];
                            self.register[x] = vx & vy
                        }
                        0x3 => {
                            // Set VX equal to the bitwise xor of the values in VX and VY.
                            let vx = self.register[x];
                            let vy = self.register[y];
                            self.register[x] = vx ^ vy
                        }
                        0x4 => {
                            // Set VX equal to VX plus VY. In the case of an overflow VF is set to 1. Otherwise 0.
                            self.add(x, y)
                        }
                        0x5 => {
                            // Set VX equal to VX minus VY. In the case of an underflow VF is set 0. Otherwise 1. (VF = VX > VY)
                            self.sub(x, y)
                        }
                        0x6 => {
                            // Set VX equal to VX bitshifted right 1. VF is set to the least significant bit of VX prior to the shift.
                            self.register[0xF] = self.register[x] & 0x1;
                            self.register[x] = self.register[x] >> 1
                        }
                        0x7 => {
                            // Set VX equal to VY minus VX. VF is set to 1 if VY > VX. Otherwise 0.
                            let vx = self.register[x];
                            let vy = self.register[y];
                            if vx > vy {
                                // + 1 due to 0 not being counted.
                                self.register[x] = (vy as i16 - vx as i16 + 1).abs() as u8;
                                self.register[0xF] = 0;
                            } else {
                                self.register[x] = (vy as i16 - vx as i16).abs() as u8;
                                self.register[0xF] = 1;
                            }
                        }
                        0xE => {
                            // Set VX equal to VX bitshifted left 1. VF is set to the most significant bit of VX prior to the shift.
                            self.register[0xF] = self.register[x] >> 7;
                            self.register[x] = self.register[x] << 1;
                        }
                        _ => {
                            panic!("Unknown OpCode was provided. OpCode: {}", opcode);
                        }
                    }
                }
                (0x9, _, _, 0x0) => {
                    // Skip the next instruction if VX does not equal VY.
                    if self.register[x] != self.register[y] {
                        self.pc += 2;
                    }
                }
                (0xA, _, _, _) => {
                    // Set I equal to NNN.
                    self.i = nnn;
                }
                (0xB, _, _, _) => {
                    // Set the PC to NNN plus the value in V0.
                    self.pc = nnn + self.register[0] as u16;
                    return
                }
                (0xC, _, _, _) => {
                    // Set VX equal to a random number ranging from 0 to 255 which is logically anded with NN.
                    let r: u8 = random();
                    self.register[x] = r & nn
                }
                (0xD, _, _, _) => {
                    // Display n-byte sprite starting at memory location I at (VX, VY).
                    // Each set bit of xored with what's already drawn.
                    // VF is set to 1 if a collision occurs.
                    // 0 otherwise.

                    let height = n;

                    // Set collision detection to 0
                    self.register[0xF] = 0x0;

                    let x_coord = self.register[x];
                    let y_coord = self.register[y];

                    for yline in 0..height {
                        let pixel = self.memory[self.i + (yline as u16)];
                        let collision = self.screen.draw_sprite_at_location(pixel, x_coord, y_coord + yline);
                        match collision {
                            true => self.register[0xF] = 0x1,
                            false => ()
                        }
                    }
                }
                (0xE, _, 0x9, 0xE) => {
                    // Skips the next instruction if the key stored in VX is pressed
                    // (usually the next instruction is a jump to skip a code block).
                    let key_pressed = self.keyboard.take_keypress();
                    let key_at_x_pressed = key_pressed == Some(x as u8);
                    match key_at_x_pressed {
                        true => self.pc += 2,
                        false => ()
                    }
                }
                (0xE, _, 0xA, 0x1) => {
                    // Skip the following instruction if the key represented by the value in VX is not pressed.
                    let key_at_x_pressed = self.keyboard.was_key_pressed(x as u8);
                    match !key_at_x_pressed {
                        true => self.pc += 2,
                        false => ()
                    }
                }
                (0xF, _, 0x0, 0x7) => {
                    // Set VX equal to the delay timer.
                    self.register[x] = self.delay_timer;
                }
                (0xF, _, 0x0, 0xA) => {
                    // Wait for a key press and store the value of the key into VX.
                    match self.keyboard.take_keypress() {
                        Some(key) => self.register[x] = key,
                        None => return
                    }
                }
                (0xF, _, 0x1, 0x5) => {
                    // Set the delay timer DT to VX.
                    self.delay_timer = self.register[x]
                }
                (0xF, _, 0x1, 0x8) => {
                    // Set the sound timer ST to VX.
                    self.sound_timer = self.register[x]
                }
                (0xF, _, 0x1, 0xE) => {
                    // Add VX to I. VF is set to 1 if I > 0x0FFF. Otherwise set to 0.
                    if self.i + self.register[x] as u16 > 0xFFF {
                        self.register[0xF] = 1
                    }
                    self.i += self.register[x] as u16;
                }
                (0xF, _, 0x2, 0x9) => {
                    // Set I to the address of the CHIP-8 8x5 font sprite representing the value in VX.
                    // TODO: copied
                    self.i = (self.register[x] * 0x5) as u16;
                }
                (0xF, _, 0x3, 0x3) => {
                    // Stores the binary-coded decimal representation of VX, with the most
                    // significant of three digits at the address in I,
                    // the middle digit at I plus 1, and the least significant digit at I plus 2.

                    self.memory[self.i] = self.register[x] / 100;
                    self.memory[self.i + 1] = self.register[x] / 10 % 10;
                    self.memory[self.i + 2] = self.register[x] % 10;
                }
                (0xF, _, 0x5, 0x5) => {
                    // Store registers V0 through VX in memory starting at location I.
                    // I does not change.
                    for idx in 0..=x as u16 {
                        self.memory[self.i + idx] = self.register[idx as usize];
                    }
                }
                (0xF, _, 0x6, 0x5) => {
                    // Copy values from memory location I through I + X into registers V0
                    // through VX. I does not change.

                    for (starting_idx, idx) in (self.i..=self.i+x as u16).enumerate() {
                        // starting_idx starts at 0 for V0 and ends at x thanks to enumerate.
                        self.register[starting_idx] = self.memory[idx];
                    }
                }
                _ => panic!("Unknown opcode was provided {opcode}!")
            }

            self.pc += 2; // move to next instruction


            // Timers are updated every iteration at 60Hz.

            // Update timers
            if self.delay_timer > 0 {
                self.delay_timer-=1;
            }

            if self.sound_timer > 0 {
                if self.sound_timer == 1 {
                    println!("BEEP!\n");

                }
                self.sound_timer-= 1;
            }
    }

    fn execute_single_instruction(&mut self) {
        let opcode = self.read_opcode();
        if opcode != 0x0 {
            self.execute_instruction(opcode)
        } else {
            return;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            if opcode != 0x0 {
                self.execute_instruction(opcode)
            } else {
                return;
            }
        }
    }

    fn set_pc_to_addr(&mut self, addr: u16) {
        self.pc = addr
    }

    fn add(&mut self, x: usize, y: usize) {
        let arg1 = self.register[x];
        let arg2 = self.register[y];
        let (val, overflow) = arg1.overflowing_add(arg2);
        self.register[x] = val;
        // CHIP-8 uses the last register as carry flag, indicating that an operation has overflowed.
        if overflow {
            self.register[0xF] = 1;
        } else {
            self.register[0xF] = 0;
        }
    }

    fn sub(&mut self, x: usize, y: usize) {
        let vx = self.register[x];
        let vy = self.register[y];

        if vy > vx {
            // borrow occurs
            let val = (vx as i16 - vy as i16 + 1).abs() as u8;
            self.register[x] = val;
            self.register[0xF] = 0;
        } else {
            self.register[x] = vx - vy;
            self.register[0xF] = 1;
        }
    }

    fn subn(&mut self, x: usize, y: usize) {
        let arg1 = self.register[x];
        let arg2 = self.register[y];
        let (val, underflow) = arg1.overflowing_sub(arg2);
        self.register[x] = val;
        // CHIP-8 uses the last register as carry flag, indicating that an operation has overflowed.
        if underflow {
            self.register[0xF] = 0;
        } else {
            self.register[0xF] = 1;
        }
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::*;

    #[test]
    fn test_load_program() {
        let program: Vec<u8> = [0; 512].to_vec();

        let chip8 = create_and_load(&program);
        assert!(chip8.is_ok())
    }

    #[test]
    fn test_load_program_that_is_too_big() {
        let program: Vec<u8> = [0; 8192].to_vec();

        let chip8 = create_and_load(&program);
        assert!(chip8.is_err())
    }

    #[test]
    fn test_clear_screen() {
        // 0x00E0; clear the screen
        let program: Vec<u8> = vec![0x0, 0xE0];

        let mut chip8 = create_and_load(&program).unwrap();

        // draw a box for testing
        for y in 0..10 {
            for x in 0..10 {
                chip8.screen.draw_pixel_at_location(x, y);
            }
        }

        chip8.run();

        let all_empty = chip8.screen.how_many_ones();

        assert_eq!(all_empty, 0);
        assert_eq!(chip8.pc, LOWER_MEMORY_BOUNDARY + 2);
    }

    #[test]
    fn test_return_from_subroutine() {
        // 0x00EE; returns from subroutine
        let program: Vec<u8> = vec![
            0x22, 0xA,
            0x0, 0x0,
            0x0, 0xE0,
            0x0, 0xE0,
            0x0, 0xE0,
            0x0, 0xEE,
        ];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.i = LOWER_MEMORY_BOUNDARY;

        assert_eq!(chip8.sp, 0);

        let orig_pc = chip8.pc;

        // the first time through will jump to the beginning
        // of the subroutine
        chip8.run();

        // assert_eq!(chip8.pc, 0x20A);
        // assert_eq!(chip8.sp, 1);
        // assert_eq!(chip8.stack[0], LOWER_MEMORY_BOUNDARY as u16);
        //
        // and the second time through should return from it
        // chip8.run();

        assert_eq!(chip8.pc, orig_pc + 2);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.memory[chip8.pc], 0x0);
    }

    #[test]
    fn test_jump_to_address() {
        // 0x1NNN: jumps to address NNN
        let program: Vec<u8> = vec![
            0x10, 0xDC
        ];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.memory[0xDC], 0);
        chip8.memory[0xDC] = 0x0;
        chip8.memory[0xDD] = 0x0;
        assert_eq!(chip8.memory[0xDC], 0x0);
        assert_eq!(chip8.memory[0xDD], 0x0);

        chip8.run();

        assert_eq!(chip8.pc, 0xDC);
        assert_eq!(chip8.memory[chip8.pc], 0x0);
    }

    #[test]
    fn test_call_subroutine_at_nnn() {
        // 0x2NNN: calls subroutine at NNN
        let program: Vec<u8> = vec![0x20, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.sp, 0);

        chip8.run();

        assert_eq!(chip8.pc, 0xDC);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], 512);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_nn_positive() {
        // 0x3XNN: Skips the next instruction if VX equals NN.
        let program: Vec<u8> = vec![0x34, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x17;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_nn_negative() {
        // 0x3XNN: Skips the next instruction if VX equals NN.
        let program: Vec<u8> = vec![0x34, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x23;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_nn_positive() {
        // 0x4XNN: Skips the next instruction if VX doesn't equal NN.
        let program: Vec<u8> = vec![0x44, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x23;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_nn_negative() {
        // 0x4XNN: Skips the next instruction if VX doesn't equal NN.
        let program: Vec<u8> = vec![0x44, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x17;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_vy_positive() {
        // 0x5XY0: Skips the next instruction if VX equals VY.
        let program: Vec<u8> = vec![0x54, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x17;
        chip8.register[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_vy_negative() {
        // 0x5XY0: Skips the next instruction if VX equals VY.
        let program: Vec<u8> = vec![0x54, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x17;
        chip8.register[6] = 0x23;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_vx_to_nn() {
        // 0x6XNN: Sets VX to NN.
        let program: Vec<u8> = vec![0x64, 0xAA];

        let chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.register[4], 0);

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.run();

        assert_eq!(chip8.register[4], 0xAA);
    }

    #[test]
    fn test_add_nn_to_vx() {
        // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
        let program: Vec<u8> = vec![0x74, 0xAA];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x10;

        chip8.run();

        assert_eq!(chip8.register[4], 0xBA);
    }

    #[test]
    fn test_add_nn_to_vx_wrapping() {
        // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
        let program: Vec<u8> = vec![0x74, 0xAA, 0x0, 0x0];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;

        chip8.run();

        assert_eq!(chip8.register[4], 0x64);
    }

    #[test]
    fn test_set_vx_to_value_of_vy() {
        // 0x8XY0: Sets VX to the value of VY.
        let program: Vec<u8> = vec![0x84, 0x50];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xDD;

        chip8.run();

        assert_eq!(chip8.register[4], 0xDD);
        assert_eq!(chip8.register[5], 0xDD);
    }

    #[test]
    fn test_set_vx_to_vx_or_vy() {
        // 0x8XY1: Sets VX to VX or VY. (Bitwise OR operation)
        let program: Vec<u8> = vec![0x84, 0x51];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xCC;

        chip8.run();

        assert_eq!(chip8.register[4], 0xFE);
        assert_eq!(chip8.register[5], 0xCC);
    }

    #[test]
    fn test_set_vx_to_vx_and_vy() {
        // 0x8XY2: Sets VX to VX and VY. (Bitwise AND operation)
        let program: Vec<u8> = vec![0x84, 0x52];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xCC;

        chip8.run();

        assert_eq!(chip8.register[4], 0x88);
        assert_eq!(chip8.register[5], 0xCC);
    }

    #[test]
    fn test_set_vx_to_vx_xor_vy() {
        // 0x8XY3: Sets VX to VX xor VY.
        let program: Vec<u8> = vec![0x84, 0x53];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xCC;

        chip8.run();

        assert_eq!(chip8.register[4], 0x76);
        assert_eq!(chip8.register[5], 0xCC);
    }

    #[test]
    fn test_add_vy_to_vx_with_carry() {
        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x54];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.register[0xF], 0);

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xCC;

        chip8.run();

        assert_eq!(chip8.register[4], 0x86);
        assert_eq!(chip8.register[0xF], 1);
    }

    #[test]
    fn test_add_vy_to_vx_without_carry() {
        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x54];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.register[0xF], 0);

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0x10;

        chip8.run();

        assert_eq!(chip8.register[4], 0xCA);
        assert_eq!(chip8.register[0xF], 0);
    }

    #[test]
    fn test_subtract_vy_from_vx_with_borrow() {
        // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow,
        // and 1 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x55];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBA;
        chip8.register[5] = 0xCC;

        chip8.run();

        assert_eq!(chip8.register[4], 0x11);
        assert_eq!(chip8.register[0xF], 0);
    }

    #[test]
    fn test_store_least_significant_bit_of_vx_in_vf_and_shift_vx_right_by_1() {
        // 0x8XY6: Stores the least significant bit of VX in VF and then shifts VX to
        // the right by 1.
        let program: Vec<u8> = vec![0x84, 0x56];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xBB;
        chip8.register[0xF] = 0x0;

        chip8.run();

        assert_eq!(chip8.register[4], 0x5D);
        assert_eq!(chip8.register[0xF], 1);
    }

    #[test]
    fn test_set_vx_to_vy_minus_vx_with_borrow() {
        // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1
        // when there isn't.
        let program: Vec<u8> = vec![0x84, 0x57];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xCC;
        chip8.register[5] = 0xBA;

        chip8.run();

        assert_eq!(chip8.register[4], 0x11);
        assert_eq!(chip8.register[0xF], 0);
    }

    #[test]
    fn test_store_most_significant_bit_of_vx_in_vf_and_shift_vx_right_by_1() {
        // 0x8XYE: Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
        let program: Vec<u8> = vec![0x84, 0x5E];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0xF0;
        chip8.register[0xF] = 0x0;

        chip8.run();

        assert_eq!(chip8.register[4], 0xE0);
        assert_eq!(chip8.register[0xF], 1);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_vy_positive() {
        // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next
        // instruction is a jump to skip a code block)
        let program: Vec<u8> = vec![0x94, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x23;
        chip8.register[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_vy_negative() {
        // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next
        // instruction is a jump to skip a code block)
        let program: Vec<u8> = vec![0x94, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 0x17;
        chip8.register[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.run();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_i_to_address_nnn() {
        // 0xANNN: sets I to the address NNN
        let program: Vec<u8> = vec![0xA0, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.i, 0);

        chip8.run();

        assert_eq!(chip8.i, 0xDC);
    }

    #[test]
    fn test_jump_to_nnn_plus_v0() {
        // 0xBNNN: Jumps to the address NNN plus V0.
        let program: Vec<u8> = vec![0xB0, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.i, 0);

        chip8.register[0] = 0x17;

        chip8.run();

        assert_eq!(chip8.pc, 0xF3);
    }

    #[test]
    fn test_draw_sprite_at_x_y_with_height_n_with_no_collision() {
        // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
        // and a height of N pixels.
        let height = 5;
        let start_x = 10;
        let start_y = 10;

        // Draw the 0 pixel at (10, 10)
        let program: Vec<u8> = vec![0xD4, 0x65];

        let mut chip8 = create_and_load(&program).unwrap();

        let how_many_ones = chip8.screen.how_many_ones();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.register[0xF], 0);

        // set i to the first sprite in the font set (the number 0)
        chip8.i = 0;
        chip8.register[4] = start_x;
        chip8.register[6] = start_y;

        chip8.run();

        let x_coord = (start_x % 32) as usize;
        let y_coord = (start_y % 64) as usize;

        let start_pixel = (y_coord * 64) + x_coord;
        let how_many_ones = chip8.screen.how_many_ones();

        assert_eq!(how_many_ones, 14);
        assert_eq!(chip8.register[0xF], 0);
    }

    #[test]
    fn test_draw_sprite_at_x_y_with_height_n_with_collision() {
        // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
        // and a height of N pixels.
        let height = 5;
        let start_x = 10;
        let start_y = 10;

        // Draw the 0 pixel at (10, 10), twice, which should result in
        // 0 pixels being set to 1, and the `chip8.register[0xF]` should be set to 1,
        // indicating a collistion
        let program: Vec<u8> = vec![0xD4, 0x65, 0xD4, 0x65];

        let mut chip8 = create_and_load(&program).unwrap();

        let how_many_ones = chip8.screen.how_many_ones();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.register[0xF], 0);

        // set i to the first sprite in the font set (the number 0)
        chip8.i = 0;
        chip8.register[4] = start_x;
        chip8.register[6] = start_y;

        // This will draw the `0` in the first step, and overwrite said zero at the second step.
        chip8.run();

        let x_coord = (start_x % 32) as usize;
        let y_coord = (start_y % 64) as usize;

        let start_pixel = (y_coord * 64) + x_coord;
        let end_pixel = start_pixel + (64 * height);

        let how_many_ones = chip8.screen.how_many_ones();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.register[0xF], 1);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_pressed_positive() {
        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0x9E];

        let mut chip8 = create_and_load(&program).unwrap();

        let orig_pc = chip8.pc;

        chip8.register[4] = key_index;
        chip8.keyboard.keypress(key_index);

        chip8.execute_single_instruction();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_pressed_negative() {
        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0x9E];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.keyboard.any_key_pressed();

        assert_eq!(keys_pressed, false);

        let orig_pc = chip8.pc;

        chip8.register[4] = key_index;

        let keys_pressed = chip8.keyboard.any_key_pressed();
        assert_eq!(keys_pressed, false);

        chip8.keyboard.keypress(key_index);

        chip8.execute_single_instruction();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_not_pressed_positive() {
        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0xA1];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.keyboard.any_key_pressed();

        assert_eq!(keys_pressed, false);

        let orig_pc = chip8.pc;

        chip8.register[4] = key_index;

        chip8.run();

        let keys_pressed = chip8.keyboard.any_key_pressed();

        assert_eq!(keys_pressed, false);
        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_not_pressed_negative() {
        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0xA1];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.keyboard.any_key_pressed();

        assert_eq!(keys_pressed, false);

        let orig_pc = chip8.pc;

        chip8.register[4] = key_index;
        chip8.keyboard.keypress(key_index);

        chip8.execute_single_instruction();

        let keys_pressed = chip8.keyboard.any_key_pressed();

        assert_eq!(keys_pressed, true);
        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_vx_to_value_of_delay_timer() {
        // 0xFX07: Sets VX to the value of the delay timer.
        let test_value: u8 = 23;
        let program: Vec<u8> = vec![0xF4, 0x07];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.delay_timer, 0);

        chip8.delay_timer = test_value;

        chip8.run();

        assert_eq!(chip8.register[4], test_value);
    }

    #[test]
    fn test_wait_for_keypress() {
        // 0xFX0A: A key press is awaited, and then stored in VX.
        // (Blocking Operation. All instruction halted until next key event)
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![
            0xF4, 0x0A
        ];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.register[4], 0);
        let orig_pc = chip8.pc;

        // After this, everything should be just as it was,
        // since no key has been pressed and program counter wasn't incremented
        chip8.execute_single_instruction();

        assert_eq!(chip8.pc, orig_pc);

        // Now set the key, and go again
        chip8.keyboard.keypress(key_index);

        // After this time, the key index should be in `chip8.register[4]`,
        // the `chip8.key` array should be all `0`, and `self.pc` should have been advanced
        chip8.execute_single_instruction();

        assert_eq!(chip8.register[4], key_index);
        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_delay_timer_to_vx() {
        // 0xFX15: Sets the delay timer to VX.
        let program: Vec<u8> = vec![0xF4, 0x15];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.delay_timer, 0);

        chip8.register[4] = 0x17;

        chip8.run();

        // the value is 1 less than what was set, because
        // the `process_timers` method has been called
        assert_eq!(chip8.delay_timer, 0x17 - 1);
    }

    #[test]
    fn test_set_sound_timer_to_vx() {
        // 0xFX18: Sets the delay timer to VX.
        let program: Vec<u8> = vec![0xF4, 0x18];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.sound_timer, 0);

        chip8.register[4] = 0x17;

        chip8.run();

        // the value is 1 less than what was set, because
        // the `process_timers` method has been called
        assert_eq!(chip8.sound_timer, 0x17 - 1);
    }

    #[test]
    fn test_add_vx_to_i_with_no_overflow() {
        // 0xFX1E: Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF),
        // and to 0 when there isn't.
        let program: Vec<u8> = vec![0xF4, 0x1E];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.i = 0xA;
        chip8.register[4] = 0x17;

        chip8.run();

        assert_eq!(chip8.i, 0x21);
        assert_eq!(chip8.register[0xF], 0);
    }

    #[test]
    fn test_add_vx_to_i_with_overflow() {
        // 0xFX1E: Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF),
        // and to 0 when there isn't.
        let program: Vec<u8> = vec![0xF4, 0x1E];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.i = 0xFFA;
        chip8.register[4] = 0xA;

        chip8.run();

        assert_eq!(chip8.i, 0x1004);
        assert_eq!(chip8.register[0xF], 1);
    }

    #[test]
    fn test_set_i_to_location_of_sprite_for_character_in_vx() {
        // 0xFX29: Sets I to the location of the sprite for the character in VX.
        // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
        let program: Vec<u8> = vec![0xF4, 0x29];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.register[4] = 2;

        chip8.run();

        assert_eq!(chip8.i, 10);
    }

    #[test]
    fn test_store_binary_coded_decimal_representation_of_vx() {
        // 0xFX33: Stores the binary-coded decimal representation of VX, with the most
        // significant of three digits at the address in I, the middle digit at I plus 1,
        // and the least significant digit at I plus 2.
        let program: Vec<u8> = vec![0xF4, 0x33];

        let mut chip8 = create_and_load(&program).unwrap();

        let first_i = LOWER_MEMORY_BOUNDARY;
        chip8.i = first_i;
        chip8.register[4] = 0xDC;

        chip8.run();

        assert_eq!(chip8.memory[first_i], 2);
        assert_eq!(chip8.memory[first_i + 1], 2);
        assert_eq!(chip8.memory[first_i + 2], 0);
    }

    #[test]
    fn test_store_v0_to_vx_in_memory_starting_at_address_i() {
        // 0xFX55: Stores V0 to VX (including VX) in memory starting at address I.
        // The offset from I is increased by 1 for each value written, but I itself
        // is left unmodified.

        let program: Vec<u8> = vec![
            0xF4, 0x55,
            0x0, 0x0 // exit
        ];

        let mut chip8 = create_and_load(&program).unwrap();

        let first_i = LOWER_MEMORY_BOUNDARY + 4;
        chip8.i = first_i;

        for i in 0..5 {
            chip8.register[i as usize] = i + 1;
        }

        chip8.run();

        for i in 0..5 {
            assert_eq!(chip8.memory[first_i + i], (i + 1) as u8);
        }
    }

    #[test]
    fn test_fill_v0_to_vx_with_values_from_memory_starting_at_address_i() {
        // 0xFX65: Fills V0 to VX (including VX) with values from memory
        // starting at address I. The offset from I is increased by 1 for
        // each value written, but I itself is left unmodified.
        let program: Vec<u8> = vec![
            0xF4, 0x65,
            0x0, 0x0 // exit
        ];

        let mut chip8 = create_and_load(&program).unwrap();

        let first_i = (LOWER_MEMORY_BOUNDARY + 4);
        chip8.i = first_i;

        for i in 0..5 {
            chip8.memory[first_i + i] = i as u8 + 1;
        }

        chip8.run();

        for i in 0..5 {
            assert_eq!(chip8.register[i as usize], i + 1);
        }
    }

    fn create_and_load(program: &Vec<u8>) -> Result<Chip8, Box<dyn Error>> {
        let mut chip8 = Chip8::new();

        chip8.load_into_memory(program.clone())?;

        Ok(chip8)
    }
}

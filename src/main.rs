use rand::random;

struct CPU {
    memory: [u8; 4069],
    // 4KB of memory
    pc: usize,
    // usize instead of u16 for easier indexing, program counter
    i: u16,
    // index register
    registers: [u8; 16],
    // register of size 16
    stack: [u16; 16],
    // program stack
    sp: usize,               // stack pointer
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        let p = self.pc;
        let high_byte = self.memory[p] as u16;
        let low_byte = self.memory[p + 1] as u16;

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

        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = addr as usize;
    }

    fn return_from_fn_call(&mut self) {
        if self.sp == 0 {
            panic!("Stack underflow!");
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            let opcode_group = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as usize;
            let y = ((opcode & 0x00F0) >> 4) as usize;
            let n = (opcode & 0x000F) as u8;
            let nnn = opcode & 0x0FFF;
            let nn = (opcode & 0x00FF) as u8;

            match (opcode_group, x, y, n) {
                (0, 0, 0, 0) => return,
                (0, 0, 0xE, 0xE) => self.return_from_fn_call(),
                (0x1, _, _, _) => self.set_pc_to_addr(nnn),
                (0x2, _, _, _) => self.call_fn_at_addr(nnn),
                (0x3, _, _, _) => {
                    // Skip the next instruction if register VX is equal to NN
                    if self.registers[x] == nn {
                        self.pc += 2;
                    }
                }
                (0x4, _, _, _) => {
                    // Skip the next instruction if register VX is not equal to NN.
                    if self.registers[x] != nn {
                        self.pc += 2;
                    }
                }
                (0x5, _, _, 0x0) => {
                    // Skip the next instruction if register VX equals VY.
                    if self.registers[x] == self.registers[y] {
                        self.pc += 2;
                    }
                }
                (0x6, _, _, _) => {
                    // Load immediate value NN into register VX.
                    self.registers[x] = nn;
                }
                (0x7, _, _, _) => {
                    // Add immediate value NN to register VX. Does not effect VF.
                    self.registers[x] += nn;
                }
                (0x8, _, _, _) => {
                    match n {
                        0x0 => {
                            // Copy the value in register VY into VX
                            self.registers[x] = self.registers[y];
                        }
                        0x1 => {
                            // Set VX equal to the bitwise or of the values in VX and VY.
                            let vx = self.registers[x];
                            let vy = self.registers[y];
                            self.registers[x] = vx | vy
                        }
                        0x2 => {
                            // Set VX equal to the bitwise and of the values in VX and VY.
                            let vx = self.registers[x];
                            let vy = self.registers[y];
                            self.registers[x] = vx & vy
                        }
                        0x3 => {
                            // Set VX equal to the bitwise xor of the values in VX and VY.
                            let vx = self.registers[x];
                            let vy = self.registers[y];
                            self.registers[x] = vx ^ vy
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
                            self.registers[0xF] = self.registers[x] & 0x1;
                            self.registers[x] = self.registers[x] >> 1
                        }
                        0x7 => {
                            // Set VX equal to VY minus VX. VF is set to 1 if VY > VX. Otherwise 0.
                            let vx = self.registers[x];
                            let vy = self.registers[y];
                            self.registers[x] = self.registers[y] - self.registers[x];
                            if vy > vx {
                                self.registers[0xF] = 1
                            } else {
                                self.registers[0xF] = 0
                            }
                        }
                        0xE => {
                            // Set VX equal to VX bitshifted left 1. VF is set to the most significant bit of VX prior to the shift.
                            self.registers[0xF] = self.registers[x] >> 7;
                            self.registers[x] = self.registers[x] << 1;
                        }
                        _ => {
                            panic!("Unknown OpCode was provided. OpCode: {}", opcode);
                        }
                    }
                }
                (0x9, _, _, 0x0) => {
                    // Skip the next instruction if VX does not equal VY.
                    if self.registers[x] != self.registers[y] {
                        self.pc += 2;
                    }
                }
                (0xA, _, _, _) => {
                    // Set I equal to NNN.
                    self.i = nnn;
                }
                (0xB, _, _, _) => {
                    // Set the PC to NNN plus the value in V0.
                    self.pc = (nnn + self.registers[0] as u16) as usize
                }
                (0xC, _, _, _) => {
                    // Set VX equal to a random number ranging from 0 to 255 which is logically anded with NN.
                    let r: u8 = random();
                    self.registers[x] = r & nn
                }
                (0xD, _, _, _) => {
                    // Display N-byte sprite starting at memory location I at (VX, VY). Each set bit of xored with what's already drawn. VF is set to 1 if a collision occurs. 0 otherwise.
                    unimplemented!()
                }
                _ => unimplemented!()
            }

            self.pc += 2; // move to next instruction
        }
    }

    fn set_pc_to_addr(&mut self, adr: u16) {
        self.pc = adr as usize
    }

    fn add(&mut self, x: usize, y: usize) {
        let arg1 = self.registers[x];
        let arg2 = self.registers[y];
        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x] = val;
        // CHIP-8 uses the last register as carry flag, indicating that an operation has overflowed.
        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sub(&mut self, x: usize, y: usize) {
        let arg1 = self.registers[x];
        let arg2 = self.registers[y];
        let (val, underflow) = arg1.overflowing_sub(arg2);
        self.registers[x] = val;
        // CHIP-8 uses the last register as carry flag, indicating that an operation has overflowed.
        if underflow {
            self.registers[0xF] = 0;
        } else {
            self.registers[0xF] = 1;
        }
    }

    fn subn(&mut self, x: usize, y: usize) {
        let arg1 = self.registers[x];
        let arg2 = self.registers[y];
        let (val, underflow) = arg1.overflowing_sub(arg2);
        self.registers[x] = val;
        // CHIP-8 uses the last register as carry flag, indicating that an operation has overflowed.
        if underflow {
            self.registers[0xF] = 0;
        } else {
            self.registers[0xF] = 1;
        }
    }
}

fn main() {
    let mut cpu = CPU {
        memory: [0; 4069],
        i: 0,
        pc: 0,
        registers: [0; 16],
        stack: [0; 16],
        sp: 0,
    };
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;
    cpu.registers[2] = 10;
    cpu.registers[3] = 10;

    let mem = &mut cpu.memory;

    mem[0] = 0x80;
    mem[1] = 0x14;   // OpCode 8014
    mem[2] = 0x80;
    mem[3] = 0x24;   // OpCode 8024
    mem[4] = 0x80;
    mem[5] = 0x34;   // OpCode 8034

    cpu.run();

    assert_eq!(cpu.registers[0], 35);

    println!("5 + 10 + 10 + 10 = {}", cpu.registers[0]);
}

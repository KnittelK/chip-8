mod memory;
mod screen;
mod keyboard;
mod cpu;

use crate::cpu::Chip8;

fn main() {
    let instructions_to_store_in_memory: [u8; 6] = [0x80, 0x14, 0x80, 0x24, 0x80, 0x34];

    let mut cpu = Chip8::new();
    cpu.populate_register(Vec::from([5, 10, 10, 10]));
    cpu.load_into_memory(Vec::from(instructions_to_store_in_memory)).unwrap();

    cpu.run();

    assert_eq!(cpu.get_value_at_register_addr(0).unwrap(), 35);

    println!("5 + 10 + 10 + 10 = {}", cpu.get_value_at_register_addr(0).unwrap());
}

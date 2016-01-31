#![allow(dead_code)]
#![allow(unused_imports)]

use std::env::{args};
use std::fs::{File};
use std::io::{Read};
use std::default;

fn main() {
    if args().count() < 2 {
        panic!("Missing argument(s). Call: ./binary <DMG ROM file name>.");
    }

    let dmg_rom_name = args().nth(1).unwrap();
    let mut rom_file = File::open(dmg_rom_name).unwrap();
    let mut rom: Vec<u8> = Vec::new();
    let _ = rom_file.read_to_end(&mut rom);

    let mut gameboy = GameBoy::new(rom);
    gameboy.turn_on()
}

const RAM_SIZE: usize = 0xFFFF;

struct GameBoy {
    cpu: CPU,
    rom: Vec<u8>,
    ram: [u8; RAM_SIZE],

}

impl GameBoy {
    fn new(rom: Vec<u8>) -> GameBoy {
        GameBoy {
            rom: rom,
            cpu: CPU::new(),
            ram: [0; RAM_SIZE],
        }
    }

    fn turn_on(&mut self) {
        self.cpu.boot();
        self.copy_rom_to_memory();

        loop {
            self.cpu.next_instruction(&self.ram);
        }
    }

    fn copy_rom_to_memory(&mut self,) {
        for idx in 0..self.rom.len() {
            self.ram[idx] = self.rom[idx];
        }
    }
}

#[derive(Default)]
struct CPU {
    // Main registers.
    acc_a: u8,
    acc_b: u8,
    acc_d: u8,
    acc_h: u8,

    flag_f: u8,
    flag_c: u8,
    flag_e: u8,
    flag_l: u8,

    // Alternative registers.
    acc_alt_a: u8,
    acc_alt_b: u8,
    acc_alt_d: u8,
    acc_alt_h: u8,

    flag_alt_f: u8,
    flag_alt_c: u8,
    flag_alt_e: u8,
    flag_alt_l: u8,

    // Special purpose registers.
    int_vec_i: u8,
    mem_refresh: u8,

    ix: u16, // Might not exist in LR35902.
    iy: u16, // Might not exist in LR35902.
    sp: u16,
    pc: u16,
}

impl CPU {
    fn new() -> CPU {
        CPU {
            pc: 0x0000,
            ..Default::default()
        }
    }

    fn boot(&self) {

    }

    fn next_instruction(&mut self, mem: &[u8])  {
        let opcode = self.read_opcode(mem);
        self.pc += 1;
        println!("Opcode read: {:#x} ({:#b})", opcode, opcode);

        match opcode {
            // LD SP ?
            0x31 => {
            },
            _ => panic!("Unknown opcode {:#x} ({:#b})", opcode, opcode),
        };
    }

    fn read_opcode(&self, mem: &[u8]) -> u8 {
        mem[self.pc as usize]
    }
}

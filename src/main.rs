#![allow(dead_code)]
#![allow(unused_imports)]

use std::env::{args};
use std::fs::{File};
use std::io::{Read};
use std::default;

fn main() {
    if args().count() < 2 {
        panic!("Missing argument(s). Call: ./binary <DMG_ROM_FILE>.");
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
        self.cpu.reset();
        self.copy_rom_to_memory();

        println!("CPU {:#?}", self.cpu);

        loop {
            self.cpu.next_instruction(&self.ram);
            println!("CPU {:#?}", self.cpu);
        }
    }

    fn copy_rom_to_memory(&mut self,) {
        for idx in 0..self.rom.len() {
            self.ram[idx] = self.rom[idx];
        }
    }
}

#[derive(Default, Debug)]
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
    int_vec: u8,
    mem_refresh: u8,

    ix: u16, // Might not exist in LR35902.
    iy: u16, // Might not exist in LR35902.
    sp: u16,
    pc: u16,
}

impl CPU {
    fn new() -> CPU {
        CPU::default()
    }

    fn reset(&mut self) {
        // Point to first instruction.
        self.pc = 0x0000;
    }

    fn next_instruction(&mut self, mem: &[u8])  {
        let opcode = self.read_opcode(mem);
        println!("Opcode read: {:#x} ({:#b})", opcode, opcode);

        match opcode {
            _ => {
                // LD dd, nn.
                if self.bit_match(0b00000001, 0b11001111, opcode) {
                    let vhigh = self.read_byte(mem);
                    let vlow = self.read_byte(mem);
                    match opcode >> 4 & 0b11 {
                        0b00 => { self.acc_b = vhigh; self.flag_c = vlow; }, // BC
                        0b01 => { self.acc_d = vhigh; self.flag_e = vlow; }, // DE
                        0b10 => { self.acc_h = vhigh; self.flag_l = vlow; }, // HL
                        0b11 => { self.sp = (vhigh as u16) << 8 | (vlow as u16); }, // SP
                        _ => unreachable!(),
                    }

                // XOR s.
                } else if self.bit_match(0b10101000, 0b11111000, opcode) {
                    match opcode & 0b111 {
                        // TODO review if it's properly stored in the same acc reg.
                        000 => { self.acc_b ^= self.acc_b; },
                        001 => { self.flag_c ^= self.flag_c; },
                        010 => { self.acc_d ^= self.acc_d; },
                        011 => { self.flag_e ^= self.flag_e; },
                        100 => { self.acc_h ^= self.acc_h; },
                        101 => { self.flag_l ^= self.flag_l; },
                        111 => { self.acc_a ^= self.acc_a; },
                        reg @ _ => panic!("Xor reg {:#b} should be handled in the strict opcode match section", reg),
                    }
                } else {
                    panic!("Unknown opcode {:#x} ({:#b})", opcode, opcode);
                }
            },
        };
    }

    /// Compare fixed and dynamic bits.
    /// Example requirement:    0b00??0001
    /// Example pattern:        0b00000001
    /// Example mask:           0b11001111
    fn bit_match(&self, pattern: u8, mask: u8, opcode: u8) -> bool {
        (opcode & mask) ^ pattern == 0
    }

    fn read_opcode(&mut self, mem: &[u8]) -> u8 {
        self.read_byte(mem)
    }

    fn read_byte(&mut self, mem: &[u8]) -> u8 {
        let byte = mem[self.pc as usize];
        self.pc += 1;
        byte
    }
}

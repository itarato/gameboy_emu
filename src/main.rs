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

        println!("{:#?}", self.cpu);

        loop {
            self.cpu.next_instruction(&self.ram);
            println!("{:#?}", self.cpu);
        }
    }

    // TODO make sure its copied to the right place (maybe keep separate?)
    fn copy_rom_to_memory(&mut self,) {
        for idx in 0..self.rom.len() {
            self.ram[idx] = self.rom[idx];
        }
    }
}

#[derive(Default, Debug)]
struct Flags {
    z_zero: bool,
    n_substract: bool,
    h_half_carry: bool,
    c_carry: bool,
}

#[derive(Default, Debug)]
struct CPU {
    // Main register set.
    acc_a: u8,
    flag_f: Flags,

    // General purpose registers.
    b: u8,
    d: u8,
    h: u8,

    c: u8,
    e: u8,
    l: u8,

    // Special purpose registers.
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
            // LD (HL-),A.
            0b00110010 => {
                // TODO need to write to memory
            },
            _ => {
                // LD dd, nn.
                if self.bit_match(0b00000001, 0b11001111, opcode) {
                    let (vlow, vhigh) = self.read_low_high(mem);
                    match opcode >> 4 & 0b11 {
                        0b00 => { self.b = vhigh; self.c = vlow; }, // BC
                        0b01 => { self.d = vhigh; self.e = vlow; }, // DE
                        0b10 => { self.h = vhigh; self.l = vlow; }, // HL
                        0b11 => { self.sp = (vhigh as u16) << 8 | (vlow as u16); }, // SP
                        _ => unreachable!(),
                    }

                // XOR s.
                } else if self.bit_match(0b10101000, 0b11111000, opcode) {
                    match opcode & 0b111 {
                        // TODO review if it's properly stored in the same acc reg.
                        0b000 => { self.b ^= self.b; },
                        0b001 => { self.c ^= self.c; },
                        0b010 => { self.d ^= self.d; },
                        0b011 => { self.e ^= self.e; },
                        0b100 => { self.h ^= self.h; },
                        0b101 => { self.l ^= self.l; },
                        0b111 => { self.acc_a ^= self.acc_a; },
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
        self.pc += 1;
        mem[(self.pc - 1) as usize]
    }

    fn read_low_high(&mut self, mem: &[u8]) -> (u8, u8) {
        (self.read_byte(mem), self.read_byte(mem))
    }
}

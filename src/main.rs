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
        self.cpu.run();
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
        CPU::default()
    }

    fn boot(&self) {

    }

    fn run(&mut self) {

    }
}
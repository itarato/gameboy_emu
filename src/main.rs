#![allow(dead_code)]
#![allow(unused_imports)]

mod bus;
mod cpu;
mod gameboy;

use std::env::{args};
use std::fs::{File};
use std::io::{Read};
use std::default;
use std::rc::Rc;
use std::cell::RefCell;
use bus::{Bus};
use cpu::{CPU};
use gameboy::{GameBoy};

fn main() {
    GameBoy
        ::new(fetch_dmg_boot_rom())
        .turn_on();
}

fn fetch_dmg_boot_rom() -> Vec<u8> {
    if args().count() < 2 {
        panic!("Missing argument(s). Call: ./binary <DMG_ROM_FILE>.");
    }

    let dmg_rom_name = args().nth(1).unwrap();
    let mut rom_file = File::open(dmg_rom_name).unwrap();
    let mut rom: Vec<u8> = Vec::new();
    let _ = rom_file.read_to_end(&mut rom);

    rom
}

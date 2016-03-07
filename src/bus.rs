use std::rc::Rc;
use std::cell::RefCell;
use timer::Timer;
use io::IO;
use std::io::prelude::*;
use std::fs::File;
use constants::*;

fn is_in(left_inc: usize, addr: usize, right_inc: usize) -> bool {
    left_inc <= addr && addr <= right_inc
}

pub struct Bus {
    mem: Rc<RefCell<[u8]>>,
    pub timer: Timer,
}

impl Bus {
    pub fn new(mem: Rc<RefCell<[u8]>>, timer: Timer) -> Bus {
        Bus {
            mem: mem,
            timer: timer,
        }
    }

    pub fn read_byte(&self, pos: usize) -> u8 {
        self.mem.borrow()[pos]
    }

    pub fn write_byte(&mut self, addr: usize, byte: u8) {
        // println!("WRITE --> {:#04X}", addr);
        self.mem.borrow_mut()[addr] = byte;

        if
            is_in(MEM_MAP_INTERNAL_RAM_START, addr, MEM_MAP_INTERNAL_RAM_ECHO_END) ||
            is_in(MEM_MAP_ECHO_OF_INTERNAL_RAM_START, addr, MEM_MAP_ECHO_OF_INTERNAL_RAM_END) {
                let diff = MEM_MAP_ECHO_OF_INTERNAL_RAM_START - MEM_MAP_ECHO_OF_INTERNAL_RAM_START;
                let offset: usize = if addr < MEM_MAP_ECHO_OF_INTERNAL_RAM_START {
                    addr.wrapping_add(diff)
                } else {
                    addr.wrapping_sub(diff)
                };
                self.mem.borrow_mut()[offset] = byte
            }
    }

    pub fn register_cycles(&mut self, cycles: u16) {
        self.timer.inc(cycles as u64);
    }

    pub fn mem_dump(&mut self) {
        let mut f = File::create("/tmp/gameboy_emu_memdump.txt").unwrap();
        let bytes = self.mem.borrow();
        let _ = f.write_all(&*bytes);
    }

}

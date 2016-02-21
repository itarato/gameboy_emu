use std::rc::Rc;
use std::cell::RefCell;
use timer::Timer;
use io::IO;

const MEM_MAP_ECHO_OF_INTERNAL_RAM_END: usize =   0xFDFF;
const MEM_MAP_ECHO_OF_INTERNAL_RAM_START: usize = 0xE000;
const MEM_MAP_INTERNAL_RAM_END: usize =           0xDFFF;
const MEM_MAP_INTERNAL_RAM_ECHO_END: usize =      0xDDFF;
const MEM_MAP_INTERNAL_RAM_START: usize =         0xC000;

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
        self.timer.inc(cycles);
    }

}

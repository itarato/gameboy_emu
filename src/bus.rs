use std::rc::Rc;
use std::cell::RefCell;
use timer::Timer;

pub struct Bus {
    mem: Rc<RefCell<[u8]>>,
    timer: Rc<RefCell<Timer>>,
}

impl Bus {
    pub fn new(mem: Rc<RefCell<[u8]>>, timer: Rc<RefCell<Timer>>) -> Bus {
        Bus {
            mem: mem,
            timer: timer,
        }
    }

    pub fn read_byte(&self, pos: usize) -> u8 {
        self.mem.borrow()[pos]
    }

    pub fn write_byte(&mut self, pos: usize, byte: u8) {
        self.mem.borrow_mut()[pos] = byte;
    }

    pub fn register_cycles(&mut self, cycles: u16) {
        self.timer.borrow_mut().inc(cycles);
    }
}

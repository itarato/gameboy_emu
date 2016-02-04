use std::rc::Rc;
use std::cell::RefCell;

pub struct Bus {
    mem: Rc<RefCell<[u8]>>,
}

impl Bus {
    pub fn new(mem: Rc<RefCell<[u8]>>) -> Bus {
        Bus {
            mem: mem
        }
    }

    pub fn read_byte(&self, pos: usize) -> u8 {
        self.mem.borrow()[pos]
    }

    pub fn write_byte(&mut self, pos: usize, byte: u8) {
        self.mem.borrow_mut()[pos] = byte;
    }
}

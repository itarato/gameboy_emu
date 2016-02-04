use cpu::{CPU};
use bus::{Bus};
use std::rc::Rc;
use std::cell::RefCell;

const RAM_SIZE: usize = 0xFFFF;

pub struct GameBoy {
    cpu: CPU,
    boot_rom: Vec<u8>,
    ram: Rc<RefCell<[u8; RAM_SIZE]>>,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom: Vec<u8>) -> GameBoy {
        let ram = Rc::new(RefCell::new([0; RAM_SIZE]));

        GameBoy {
            boot_rom: boot_rom,
            cpu: CPU::new(),
            ram: ram.clone(),
            bus: Bus::new(ram.clone()),
        }
    }

    pub fn turn_on(&mut self) {
        self.cpu.reset();
        self.copy_rom_to_memory();

        loop {
            self.cpu.next_instruction(&mut self.bus);
            println!("{:#?}", self.cpu);
        }
    }

    // TODO make sure its copied to the right place (maybe keep separate?)
    // TODO think about moving this operation to the bus so GameBoy does not need a mutable refcell of mem.
    fn copy_rom_to_memory(&mut self) {
        for idx in 0..self.boot_rom.len() {
            self.ram.borrow_mut()[idx] = self.boot_rom[idx];
        }
    }
}

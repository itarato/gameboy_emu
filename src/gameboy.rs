use cpu::CPU;
use cpu;
use bus::Bus;
use timer::Timer;
use io::IO;
use io;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use constants::*;

pub struct GameBoy {
    cpu: CPU,
    boot_rom: Vec<u8>,
    ram: Rc<RefCell<[u8; RAM_SIZE]>>,
    io: IO,
    bus: Bus,
}

impl GameBoy {
    pub fn new(boot_rom: Vec<u8>) -> GameBoy {
        let ram = Rc::new(RefCell::new([0; RAM_SIZE]));
        let timer = Timer::default();
        let io = IO;

        GameBoy {
            boot_rom: boot_rom,
            cpu: CPU::new(),
            ram: ram.clone(),
            io: io,
            bus: Bus::new(ram.clone(), timer),
        }
    }

    pub fn turn_on(&mut self) {
        self.cpu.reset();
        self.copy_rom_to_memory();
        self.io.init(&mut self.bus.timer);

        loop {
            self.cpu.next_instruction(&mut self.bus);
            self.cpu.check_interrupt(&mut self.bus);
            self.io.operate(&mut self.bus);
            println!("{:#?}", self);
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

impl fmt::Debug for GameBoy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let div = self.bus.read_byte(REG_DIV as usize);
        let if_reg = self.bus.read_byte(REG_IF as usize);
        let lcdc = self.bus.read_byte(REG_LCDC as usize);
        let stat = self.bus.read_byte(REG_STAT as usize);
        let ie = self.bus.read_byte(REG_IE as usize);
        let ly = self.bus.read_byte(REG_LY as usize);
        write!(f, "{:#?}
DIV:  {:#010b}
IF:   {:#010b}
LCDC: {:#010b}
STAT: {:#010b}
IE:   {:#010b}
LY:   {:#010b}",
        self.cpu,
        div,
        if_reg,
        lcdc,
        stat,
        ie,
        ly)
    }
}

use timer::Timer;
use bus::Bus;
use cpu;
use cpu::CPU;

const TICK_DIV_REG: u16 = 255;
const TICK_VBLANK: u16 = 200;

pub const REG_DIV: u16 = 0xFF04;

pub struct IO;

impl IO {
    pub fn new() -> IO {
        IO
    }

    pub fn init(&self, timer: &mut Timer) {
        timer.register_tick(TICK_DIV_REG);
        timer.register_tick(TICK_VBLANK);
    }

    pub fn operate(&self, bus: &mut Bus) {
        if bus.timer.did_tick(TICK_DIV_REG) {
            let div = bus.read_byte(REG_DIV as usize);
            bus.write_byte(REG_DIV as usize, div.wrapping_add(1));
        }

        if bus.timer.did_tick(TICK_VBLANK) {
            let if_val = bus.read_byte(cpu::IF_ADDR as usize);
            bus.write_byte(cpu::IF_ADDR as usize, if_val | 1);
        }
    }
}

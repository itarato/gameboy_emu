use std::collections::HashMap;

#[derive(Debug, Default)]
struct Ticker {
    cycle: u16,
    did_tick: bool,
}

#[derive(Default)]
pub struct Timer {
    // Not sure it's necessary.
    // cycle: u64,
    ticks: HashMap<u16, Ticker>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn register_tick(&mut self, cycle: u16) {
        self.ticks.insert(cycle, Ticker::default());
    }

    pub fn inc(&mut self, curr_cycles: u16) {
        // self.cycle = self.cycle.wrapping_add(cycles as u64);
        for (cycle, ticker) in self.ticks.iter_mut() {
            ticker.cycle += curr_cycles;
            if ticker.cycle >= *cycle {
                ticker.cycle = ticker.cycle - cycle + 1;
                ticker.did_tick = true;
                println!("TICK {:?}", cycle);
            }
        }
    }

    pub fn did_tick(&mut self, cycle: u16) -> bool {
        let mut ticker = self.ticks.get_mut(&cycle).unwrap();
        let did_tick = ticker.did_tick;
        ticker.did_tick = false;
        did_tick
    }
}

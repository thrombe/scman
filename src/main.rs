#![allow(unused_imports)]
#![allow(dead_code)]

use rdev::{listen, Event};
use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

#[derive(Debug, Default)]
struct Handle {
    prev_scroll_ts: f64,
    seperator_del_t: f64, // this much gap between scrolls to be considered seperate scrolls
    scroll_type: ScrollType,
}

#[derive(Debug, Clone)]
enum ScrollType {
    FlatMultiplier{m: i64},
    SpeedDependence{
        combo_del_t: f64, // if scrolled faster than this, combo++
        combo_num: i64,
        clamp_max: i64, 
    },
}
impl Default for ScrollType {
    fn default() -> Self {
        Self::FlatMultiplier {m: 1}
    }
}

impl Handle {
    fn new() -> Self {
        Self {
            seperator_del_t: 0.01,
            // scroll_type: ScrollType::FlatMultiplier { m: 6 },
            scroll_type: ScrollType::SpeedDependence {
                combo_del_t: 0.06, // a casual fast scroll is like 0.01 sec apart
                clamp_max: 6,

                combo_num: 0,
            },
            ..Default::default()
        }
    }
    
    fn callback(&mut self, event: Event) {
        self.scroll_callback(&event);
    }
    
    fn scroll_callback(&mut self, event: &Event) {
        let now = self.now();
        let now_del = now-self.prev_scroll_ts;
        if now_del < self.seperator_del_t {
            return;
        }

        let mut multiplier = match event.event_type {
            EventType::Wheel { delta_x: 0, delta_y: 1 } => 1,
            EventType::Wheel { delta_x: 0, delta_y: -1 } => -1,
            _ => return,
        };

        multiplier *= match &mut self.scroll_type {
            ScrollType::FlatMultiplier { m } => *m,
            ScrollType::SpeedDependence {combo_del_t, combo_num, clamp_max} => {
                if now_del < *combo_del_t {
                    *combo_num += 1;
                    *combo_num = (*combo_num).clamp(0, *clamp_max);
                } else {
                    *combo_num = 0;
                }
                // dbg!(&combo_num);
                *combo_num
            },
        };

        for _ in 0..multiplier.abs() {
            send(&EventType::Wheel { delta_x: 0, delta_y: multiplier.signum() });
        }
        self.prev_scroll_ts = self.now();
    }

    fn now(&self) -> f64 {
        time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
    }

    fn event_ts(&self, e: Event) -> f64 {
        e.time.duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
    }
}


fn main() {
    // This will block.
    let mut h = Handle::new();
    if let Err(error) = listen(move |e| {
        h.callback(e);
    }) {
        println!("Error: {:?}", error)
    }
}


fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(5);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

#![allow(unused_imports)]
#![allow(dead_code)]

use rdev::{listen, Event};
use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

#[derive(Debug)]
struct Handle {
    ts: f64,
}

impl Handle {
    fn callback(&mut self, event: Event) {
        // dbg!(&event);
        // println!("{:?} {:?}", &event, self.val);
        // dbg!(self.ts);

        let del = 5;
        let tdel = 0.01;
        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64();
        match event.event_type {
            EventType::Wheel { delta_x: 0, delta_y: 1 } => {
                if now-self.ts > tdel {
                    for _ in 0..del {
                        send(&EventType::Wheel { delta_x: 0, delta_y: 1 });
                    }
                    // println!("triggered");
                    self.ts = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64();
                }
            }
            EventType::Wheel { delta_x: 0, delta_y: -1 } => {
                if now-self.ts > tdel {
                    for _ in 0..del {
                        send(&EventType::Wheel { delta_x: 0, delta_y: -1 });
                    }
                    // println!("triggered");
                    self.ts = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64();
                }
            }
            _ => ()
        }
    }
}


fn main() {
    // This will block.
    let mut h = Handle {ts: 0.0};
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

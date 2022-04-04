#![allow(unused_imports)]
#![allow(dead_code)]

use rdev::{listen, grab, Event};
use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

#[derive(Debug, Default)]
struct ScrollHandle {
    prev_scroll_ts: f64,
    seperator_del_t: f64, // this much gap between scrolls to be considered seperate scrolls
    scroll_mode: ScrollMode,
}

#[derive(Debug, Clone)]
enum ScrollMode {
    FlatMultiplier{m: i64},
    LinearIncline{
        combo_del_t: f64, // if scrolled faster than this, combo++
        combo_num: i64,
        clamp_max: i64, 
    },
}
impl Default for ScrollMode {
    fn default() -> Self {
        Self::FlatMultiplier {m: 1}
    }
}

impl ScrollHandle {
    fn new() -> Self {
        Self {
            seperator_del_t: 0.01,
            // scroll_type: ScrollType::FlatMultiplier { m: 6 },
            scroll_mode: ScrollMode::LinearIncline {
                combo_del_t: 0.06, // a casual fast scroll is like 0.01 sec apart
                clamp_max: 6,

                combo_num: 0,
            },
            ..Default::default()
        }
    }
    
    /// scrolling cannot be blocked for some reason (rdev 0.5.0)
    fn callback(&mut self, event: &Event) -> EventStatus {
        let now = now_ts();
        let now_del = now-self.prev_scroll_ts;
        if now_del < self.seperator_del_t {
            return EventStatus::UnHandled;
        }

        let mut multiplier = match event.event_type {
            EventType::Wheel { delta_x: 0, delta_y: 1 } => 1,
            EventType::Wheel { delta_x: 0, delta_y: -1 } => -1,
            _ => return EventStatus::UnHandled,
        };

        multiplier *= match &mut self.scroll_mode {
            ScrollMode::FlatMultiplier { m } => *m,
            ScrollMode::LinearIncline {combo_del_t, combo_num, clamp_max} => {
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
        self.prev_scroll_ts = now_ts();
        EventStatus::Block
    }

}

fn now_ts() -> f64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
}

fn event_ts(e: Event) -> f64 {
    e.time.duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
}

#[derive(Debug, Default)]
struct MouseClickHandle {
    held: bool,
    moved: bool,
}

impl MouseClickHandle {
    fn new() -> Self {
        Self {
            held: false,
            moved: false,
            ..Default::default()
        }
    }

    /// remaps rightclick+drag to meta+leftclick+drag
    fn callback(&mut self, event: &Event) -> EventStatus {
        match event.event_type {
            EventType::ButtonPress(Button::Right) => {
                if !self.held {
                    self.held = true;
                } else if self.moved {
                    send(&EventType::KeyPress(Key::MetaLeft));
                    send(&EventType::ButtonPress(Button::Left));
                }
            },
            EventType::ButtonRelease(Button::Right) => {
                if !self.moved {
                    send(&EventType::ButtonPress(Button::Right));
                    send(&EventType::ButtonRelease(Button::Right));
                } else {
                    send(&EventType::ButtonRelease(Button::Left));
                    send(&EventType::KeyRelease(Key::MetaLeft));
                }
                self.reset();
            }
            EventType::MouseMove { x: _, y: _ } => {
                if self.held {
                    self.moved = true;
                }
                return EventStatus::NoBlock;
            }
            _ => return EventStatus::UnHandled,
        }

        EventStatus::Block
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}


enum EventStatus {
    Block,
    NoBlock,
    UnHandled,
}

impl EventStatus {
    fn not_handled(&self) -> bool {
        if let Self::UnHandled = self {
            true
        } else {
            false
        }
    }

    fn get_event(&self, e: &Event) -> Option<Event> {
        match self {
            Self::Block => None,
            Self::NoBlock => Some(e.clone()),
            Self::UnHandled => panic!(),
        }
    }
}


fn main() {
    // grabb();
    listenn();
}

fn listenn() {
    let mut scroll_handle = ScrollHandle::new();
    if let Err(error) = listen(move |e| {
        // dbg!(&e);
        
        scroll_handle.callback(&e);
    }) {
        println!("Error: {:?}", error)
    }

}

// if grabbing
//   trackpad edge scroll does not work
//   trackpad scroll defaults to 2 finger
//   trackpad events do not come here
//   smol lag when starting and stopping the program
#[cfg(feature = "custom_rdev")]
fn grabb() {
    let mut scroll_handle = ScrollHandle::new();
    let mut mouse_click_handle = MouseClickHandle::new();
    if let Err(error) = grab(move |e| -> Option<Event> {
        // dbg!(&e);

        let shc = scroll_handle.callback(&e);
        if !shc.not_handled() {
            return shc.get_event(&e);
        }

        let mchc = mouse_click_handle.callback(&e);
        if !mchc.not_handled() {
            return mchc.get_event(&e);
        }

        Some(e)
    }) {
            println!("Error: {:?}", error)
    }
}


fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }

    // Let ths OS catchup (at least MacOS)
    // let delay = time::Duration::from_millis(5);
    // thread::sleep(delay);
}

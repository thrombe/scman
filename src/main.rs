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
    dbg: bool,
}

#[derive(Debug, Clone)]
enum ScrollMode {
    FlatMultiplier{m: i64},
    LinearIncline{
        combo_del_t: f64, // if scrolled faster than this, combo++
        combo_num: i64,
        clamp_max: i64, 
    },
    DelTimeInverse{
        multiplier_bias: f64,
        clamp_max: i64,
    },
    DelTimeInvMap{
        max_scroll_speed: f64, // max scroll ticks per second that the user can do
        mappers: Vec<DelTimeInvMapElement>,
    },
}

#[derive(Debug, Clone, Copy)]
struct DelTimeInvMapElement {
    /// value where this gets triggered. start from 0.0 (<=1.0)
    trigger_val: f64,
    m: i64,
}
impl Default for ScrollMode {
    fn default() -> Self {
        Self::FlatMultiplier {m: 1}
    }
}

impl ScrollHandle {
    fn new() -> Self {
        let _fm = ScrollMode::FlatMultiplier { m: 6 };
        let _li = ScrollMode::LinearIncline {
            combo_del_t: 0.06, // a casual fast scroll is like 0.01 sec apart
            clamp_max: 6,
            combo_num: Default::default(),
        };
        let _dti = ScrollMode::DelTimeInverse {
            // multiplier_bias: 10.0,
            // clamp_max: 9,
            multiplier_bias: 20.0,
            clamp_max: 4,
        };
        let _dtim = ScrollMode::DelTimeInvMap {
            max_scroll_speed: 1.0/0.015,
            mappers: vec![
                DelTimeInvMapElement {trigger_val: 0.0, m: 0},
                DelTimeInvMapElement {trigger_val: 0.07, m: 1},
                DelTimeInvMapElement {trigger_val: 0.12, m: 2},
                DelTimeInvMapElement {trigger_val: 0.17, m: 3},
                DelTimeInvMapElement {trigger_val: 0.3, m: 4},
                DelTimeInvMapElement {trigger_val: 0.6, m: 5},
                // DelTimeInvMapElement {trigger_val: 0.8, m: 6},
            ],
        };

        // just to allow quick switching these for testing
        let scroll_mode = 
            // _fm
            // _li
            // _dti
            _dtim
        ;

        Self {
            seperator_del_t: 0.01,
            scroll_mode,
            // dbg: true,
            ..Default::default()
        }
    }
    
    /// scrolling cannot be blocked for some reason (rdev 0.5.0)
    fn callback(&mut self, event: &Event) -> EventStatus {
        let now = event_ts(event);
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
                if self.dbg {
                    dbg!(&combo_num);
                }
                *combo_num
            },
            ScrollMode::DelTimeInverse { multiplier_bias, clamp_max } => {
                let now_del_inv = 1.0/now_del;
                let m = now_del_inv*(*multiplier_bias)*0.01;
                if self.dbg {
                    dbg!(m);
                }
                m.clamp(0.0, *clamp_max as f64) as i64
            },
            ScrollMode::DelTimeInvMap { max_scroll_speed, mappers } => {
                let now_del_inv = 1.0/now_del;
                let scaled_speed = now_del_inv / *max_scroll_speed; // hopefully 0.0..1.0
                let index = mappers.iter()
                .position(|m| m.trigger_val > scaled_speed)
                .unwrap_or(mappers.len())
                -1;
                let map = mappers[index];
                if self.dbg {
                    dbg!(scaled_speed, map.m);
                }
                map.m
            },
        };

        for _ in 0..multiplier.abs() {
            send(&EventType::Wheel { delta_x: 0, delta_y: multiplier.signum() });
        }
        self.prev_scroll_ts = event_ts(event);
        EventStatus::Block
    }

}

fn now_ts() -> f64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
}

fn event_ts(e: &Event) -> f64 {
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

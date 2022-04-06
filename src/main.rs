#![allow(unused_imports)]
#![allow(dead_code)]

mod mouse_click_handle;
mod scroll_handle;

use rdev::{listen, grab, Event};
use rdev::{simulate, Button, EventType, Key, SimulateError};
use std::{thread, time};

use crate::{
    scroll_handle::{ScrollHandle, ScrollMode},
    mouse_click_handle::{},
};

fn main() {
    // grabb().unwrap();
    listenn().unwrap();
}

fn now_ts() -> f64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
}

fn event_ts(e: &Event) -> f64 {
    e.time.duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64()
}

pub enum EventStatus {
    Block,
    NoBlock,
    UnHandled,
}

impl EventStatus {
    pub fn not_handled(&self) -> bool {
        if let Self::UnHandled = self {
            true
        } else {
            false
        }
    }

    pub fn get_event(&self, e: &Event) -> Option<Event> {
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

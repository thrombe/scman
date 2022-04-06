
use crate::{
    EventStatus, now_ts, event_ts, Event, EventType, send, Button, Key,
};


#[derive(Debug, Default)]
pub struct MouseClickHandle {
    held: bool,
    moved: bool,
}

impl MouseClickHandle {
    pub fn new() -> Self {
        Self {
            held: false,
            moved: false,
            ..Default::default()
        }
    }

    /// remaps rightclick+drag to meta+leftclick+drag
    pub fn callback(&mut self, event: &Event) -> EventStatus {
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

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}


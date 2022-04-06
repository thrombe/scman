
use crate::{
    EventStatus, now_ts, event_ts, Event, EventType, send,
};


#[derive(Debug, Default)]
pub struct ScrollHandle {
    prev_scroll_ts: f64,
    
    /// this much gap between scrolls to be considered seperate scrolls
    seperator_del_t: f64,
    scroll_mode: ScrollMode,
    
    /// turns on debug prints
    dbg: bool,
}

#[derive(Debug, Clone)]
pub enum ScrollMode {
    FlatMultiplier{m: i64},
    LinearIncline{
        /// if scrolled faster than this, combo++
        combo_del_t: f64,
        combo_num: i64,
        clamp_max: i64, 
        clamp_min: i64,
    },
    DelTimeInverse{
        multiplier_bias: f64,
        clamp_max: i64,
        clamp_min: i64,
    },
    DelTimeInvMap{
        max_scroll_speed: f64, // max scroll ticks per second that the user can do
        mappers: Vec<DelTimeInvMapElement>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct DelTimeInvMapElement {
    /// value where this gets triggered. start from 0.0 (<=1.0)
    trigger_val: f64,
    scroll_val: i64,
}
impl Default for ScrollMode {
    fn default() -> Self {
        Self::FlatMultiplier {m: 1}
    }
}

impl ScrollHandle {
    pub fn new() -> Self {
        let _flat_multiplier = ScrollMode::FlatMultiplier { m: 6 };
        let _linear_incline = ScrollMode::LinearIncline {
            combo_del_t: 0.06, // a casual fast scroll is like 0.01 sec apart
            clamp_max: 6,
            clamp_min: 1,
            combo_num: Default::default(),
        };
        let _del_time_inverse = ScrollMode::DelTimeInverse {
            // multiplier_bias: 10.0,
            // clamp_max: 9,
            multiplier_bias: 20.0,
            clamp_max: 4,
            clamp_min: 1,
        };
        let _del_time_inv_map = ScrollMode::DelTimeInvMap {
            max_scroll_speed: 1.0/0.015,
            mappers: vec![
                DelTimeInvMapElement {trigger_val: 0.0, scroll_val: 1},
                DelTimeInvMapElement {trigger_val: 0.10, scroll_val: 2},
                DelTimeInvMapElement {trigger_val: 0.17, scroll_val: 3},
                DelTimeInvMapElement {trigger_val: 0.3, scroll_val: 4},
                DelTimeInvMapElement {trigger_val: 0.6, scroll_val: 5},
                // DelTimeInvMapElement {trigger_val: 0.8, m: 6},
            ],
        };

        // just to allow quick switching these for testing
        let scroll_mode = 
            // _flat_multiplier
            // _linear_incline
            // _del_time_inverse
            _del_time_inv_map
        ;

        Self {
            seperator_del_t: 0.01,
            scroll_mode,
            // dbg: true,
            ..Default::default()
        }
    }
    
    /// scrolling cannot be blocked for some reason (rdev 0.5.0)
    pub fn callback(&mut self, event: &Event) -> EventStatus {
        let now = event_ts(event);
        let now_del = now-self.prev_scroll_ts;
        if now_del < self.seperator_del_t {
            return EventStatus::UnHandled;
        }
        self.prev_scroll_ts = now;

        let mut multiplier = match event.event_type {
            EventType::Wheel { delta_x: 0, delta_y: 1 } => 1,
            EventType::Wheel { delta_x: 0, delta_y: -1 } => -1,
            _ => return EventStatus::UnHandled,
        };

        multiplier *= match &mut self.scroll_mode {
            ScrollMode::FlatMultiplier { m } => *m,
            ScrollMode::LinearIncline {combo_del_t, combo_num, clamp_max, clamp_min} => {
                if now_del < *combo_del_t {
                    *combo_num += 1;
                    *combo_num = (*combo_num).clamp(*clamp_min, *clamp_max);
                } else {
                    *combo_num = 0;
                }
                if self.dbg {
                    dbg!(&combo_num);
                }
                *combo_num
            },
            ScrollMode::DelTimeInverse { multiplier_bias, clamp_max, clamp_min } => {
                let now_del_inv = 1.0/now_del;
                let m = now_del_inv*(*multiplier_bias)*0.01;
                if self.dbg {
                    dbg!(m);
                }
                m.clamp(*clamp_min as f64, *clamp_max as f64) as i64
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
                    dbg!(scaled_speed, map.scroll_val);
                }
                map.scroll_val
            },
        };

        for _ in 0..multiplier.abs() {
            send(&EventType::Wheel { delta_x: 0, delta_y: multiplier.signum() });
        }
        EventStatus::Block
    }
}

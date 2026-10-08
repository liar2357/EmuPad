use crate::input::frame::{TouchFrame, TouchPoint};
use evdev::EventSummary;
use std::collections::HashMap;

pub struct TouchState {
    current_slot: usize,
    slots: HashMap<usize, TouchPoint>,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            current_slot: 0,
            slots: HashMap::new(),
        }
    }

    /// evdevイベントを処理する。
    ///
    /// SYN_REPORTが来た場合は、その時点のTouchFrameを返す。
    pub fn process_event(&mut self, event: EventSummary) -> Option<TouchFrame> {
        match event {
            EventSummary::AbsoluteAxis(_, axis, value) => {
                match axis {
                    evdev::AbsoluteAxisCode::ABS_MT_SLOT => {
                        self.current_slot = value as usize;
                    }

                    evdev::AbsoluteAxisCode::ABS_MT_TRACKING_ID => {
                        if value < 0 {
                            // 現在のslotから指が離れた
                            self.slots.remove(&self.current_slot);
                        } else {
                            // 新しい指
                            self.slots.insert(
                                self.current_slot,
                                TouchPoint {
                                    slot: self.current_slot,
                                    tracking_id: value,
                                    x: 0,
                                    y: 0,
                                },
                            );
                        }
                    }

                    evdev::AbsoluteAxisCode::ABS_MT_POSITION_X => {
                        if let Some(point) = self.slots.get_mut(&self.current_slot) {
                            point.x = value;
                        }
                    }

                    evdev::AbsoluteAxisCode::ABS_MT_POSITION_Y => {
                        if let Some(point) = self.slots.get_mut(&self.current_slot) {
                            point.y = value;
                        }
                    }

                    // その他のABSイベントは現時点では無視
                    _ => {}
                }
            }

            EventSummary::Synchronization(_, _, _) => {
                return Some(self.frame());
            }

            // KEY / MISCなどは現時点では無視
            _ => {}
        }

        None
    }

    /// 現在のタッチ状態からTouchFrameを生成する。
    pub fn frame(&self) -> TouchFrame {
        let mut fingers: Vec<_> = self.slots.values().copied().collect();

        fingers.sort_by_key(|finger| finger.slot);

        TouchFrame { fingers }
    }
}

impl Default for TouchState {
    fn default() -> Self {
        Self::new()
    }
}

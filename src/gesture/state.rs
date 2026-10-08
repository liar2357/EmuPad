use std::time::Instant;

#[derive(Debug, Clone)]
pub enum GestureState {
    Idle,

    OneFinger {
        started_at: Instant,
        start: (f64, f64),
    },

    TwoFinger {
        started_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
        action: TwoFingerAction,
    },

    ThreeFinger {
        started_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
        action: ThreeFingerAction,
    },

    ThreeFingerReleasing {
        started_at: Instant,
        released_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
    },

    SuppressUntilRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoFingerAction {
    Pending,
    Moving,
    LeftClick,
    RightClick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreeFingerAction {
    Pending,
    Scrolling,
    MiddleClick,
}

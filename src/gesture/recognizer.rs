#![allow(clippy::too_many_arguments)]

use crate::{
    config::structs::GestureConfig,
    coordinate::transform::{Transform, transform_delta},
    error::Error,
    event::log::Logger,
    gesture::state::{GestureState, ThreeFingerAction, TwoFingerAction},
    input::frame::TouchFrame,
    output::event::{MouseButton, MouseEvent},
};
use std::time::{Duration, Instant};

const MOVE_RATE: f64 = 0.75;
const SCROLL_RATE: f64 = 0.2;

pub struct GestureRecognizer {
    state: GestureState,

    tap_threshold: Duration,
    long_press_threshold: Duration,
    movement_threshold: f64,
    release_grace_threshold: Duration,

    transform_direction: Transform,

    move_sensitivity: f64,
    scroll_sensitivity: f64,
}
impl GestureRecognizer {
    pub fn new(config: &GestureConfig) -> Result<Self, Error> {
        Ok(Self {
            state: GestureState::Idle,

            tap_threshold: Duration::from_millis(config.tap_threshold_ms),
            long_press_threshold: Duration::from_millis(config.long_press_threshold_ms),
            movement_threshold: config.movement_threshold,
            release_grace_threshold: Duration::from_millis(config.release_grace_threshold_ms),
            transform_direction: config.default_direction,
            move_sensitivity: config.move_sensitivity_percent as f64 / 100.0,
            scroll_sensitivity: config.scroll_sensitivity_percent as f64 / 100.0,
        })
    }

    pub fn set_transform_direction(&mut self, new_direction: Transform) {
        self.transform_direction = new_direction;
    }

    pub fn reload_config(&mut self, config: &GestureConfig) -> Result<(), Error> {
        self.state = GestureState::Idle;
        self.tap_threshold = Duration::from_millis(config.tap_threshold_ms);
        self.long_press_threshold = Duration::from_millis(config.long_press_threshold_ms);
        self.movement_threshold = config.movement_threshold;
        self.release_grace_threshold = Duration::from_millis(config.release_grace_threshold_ms);
        self.transform_direction = config.default_direction;
        self.move_sensitivity = config.move_sensitivity_percent as f64 / 100.0;
        self.scroll_sensitivity = config.scroll_sensitivity_percent as f64 / 100.0;

        Ok(())
    }

    pub fn process(&mut self, frame: &TouchFrame, logger: &Logger) -> Vec<MouseEvent> {
        let finger_count = frame.finger_count();
        let now = Instant::now();

        let mut dx = 0.0;
        let mut dy = 0.0;

        /*
         * 現在のフレームに指が存在する場合のみ重心を取得する。
         *
         * 0本の場合は None になる。
         */
        let current_center = frame.centroid();

        logger.trace(format!(
            "FRAME: fingers={}, state={:?}",
            finger_count, self.state
        ));

        let next_state = match &self.state {
            GestureState::Idle => match finger_count {
                0 => GestureState::Idle,

                1 => GestureState::OneFinger {
                    started_at: now,
                    start: current_center.unwrap(),
                },

                2 => GestureState::TwoFinger {
                    started_at: now,
                    start_center: current_center.unwrap(),
                    last_center: current_center.unwrap(),
                    action: TwoFingerAction::Pending,
                },

                3 => GestureState::ThreeFinger {
                    started_at: now,
                    start_center: current_center.unwrap(),
                    last_center: current_center.unwrap(),
                    action: ThreeFingerAction::Pending,
                },

                _ => GestureState::Idle,
            },

            GestureState::OneFinger {
                started_at: _,
                start: _,
            } => {
                match finger_count {
                    0 => GestureState::Idle,

                    1 => {
                        // 1本指操作は本プログラムではジェスチャとして処理しない。
                        GestureState::OneFinger {
                            started_at: now,
                            start: current_center.unwrap(),
                        }
                    }

                    2 => {
                        /*
                         * 2本目が追加された瞬間。
                         *
                         * このフレームの重心を2本指ジェスチャの開始位置とする。
                         * 1本→2本になったことによる見かけ上のdeltaは
                         * ジェスチャ移動量として扱わない。
                         */
                        let center = current_center.unwrap();

                        GestureState::TwoFinger {
                            started_at: now,
                            start_center: center,
                            last_center: center,
                            action: TwoFingerAction::Pending,
                        }
                    }

                    3 => {
                        let center = current_center.unwrap();

                        GestureState::ThreeFinger {
                            started_at: now,
                            start_center: center,
                            last_center: center,
                            action: ThreeFingerAction::Pending,
                        }
                    }

                    _ => GestureState::Idle,
                }
            }

            GestureState::TwoFinger {
                started_at,
                start_center,
                last_center,
                action,
            } => {
                /*
                 * 2本指のときだけ重心の差分を移動量として扱う。
                 *
                 * 2 -> 1 のように指本数が変化したフレームでは、
                 * 2本指重心と1本指座標の差分が巨大な移動量になるため、
                 * dx/dyを計算しない。
                 */
                if finger_count == 2
                    && let Some(current) = current_center
                {
                    dx = current.0 - last_center.0;
                    dy = current.1 - last_center.1;
                }

                self.process_two_finger(
                    finger_count,
                    now,
                    *started_at,
                    *start_center,
                    *last_center,
                    *action,
                    current_center,
                    logger,
                )
            }

            GestureState::ThreeFinger {
                started_at,
                start_center,
                last_center,
                action,
            } => {
                /*
                 * 3本指のときだけ重心の差分をスクロール量として扱う。
                 *
                 * 3 -> 2 / 1 の指本数変化フレームでは、
                 * スクロール量を生成しない。
                 */
                if finger_count == 3
                    && let Some(current) = current_center
                {
                    dx = current.0 - last_center.0;
                    dy = current.1 - last_center.1;
                }

                self.process_three_finger(
                    finger_count,
                    now,
                    *started_at,
                    *start_center,
                    *last_center,
                    *action,
                    current_center,
                    logger,
                )
            }

            GestureState::ThreeFingerReleasing {
                started_at,
                released_at,
                start_center,
                last_center,
            } => self.process_three_finger_releasing(
                finger_count,
                now,
                *started_at,
                *released_at,
                *start_center,
                *last_center,
                logger,
            ),

            GestureState::SuppressUntilRelease => {
                if finger_count == 0 {
                    GestureState::Idle
                } else {
                    GestureState::SuppressUntilRelease
                }
            }
        };

        let current_state = self.state.clone();

        self.state = next_state.clone();

        let events = self.to_mouse_events(&next_state, &current_state, dx, dy);

        // クリック状態は1フレームだけ有効
        match next_state {
            GestureState::TwoFinger {
                action: TwoFingerAction::LeftClick,
                ..
            }
            | GestureState::TwoFinger {
                action: TwoFingerAction::RightClick,
                ..
            }
            | GestureState::ThreeFinger {
                action: ThreeFingerAction::MiddleClick,
                ..
            } => {
                self.state = GestureState::Idle;
            }
            _ => {}
        }

        events
    }

    fn process_two_finger(
        &self,
        finger_count: usize,
        now: Instant,
        started_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
        action: TwoFingerAction,
        current_center: Option<(f64, f64)>,
        logger: &Logger,
    ) -> GestureState {
        //eprintln!("process two finger");

        /*
         * 2本指状態から3本指へ遷移。
         */
        if finger_count == 3 {
            logger.trace("finger_count: 2 -> 3");

            let center = current_center.unwrap();

            return GestureState::ThreeFinger {
                started_at: now,
                start_center: center,
                last_center: center,
                action: ThreeFingerAction::Pending,
            };
        }

        if finger_count == 1 {
            logger.trace("finger_count: 2 -> 1");

            /*
             * Pending中なら、2本指タップの途中として保持する。
             *
             * 2 -> 1 -> 0 のリリースを許容するため。
             */
            if action == TwoFingerAction::Pending {
                return GestureState::TwoFinger {
                    started_at,
                    start_center,
                    last_center,
                    action,
                };
            }

            /*
             * Moving / RightClick成立後に1本になった場合は、
             * 2本指ジェスチャーを終了する。
             *
             * 1本になったフレームではMoveを生成しない。
             */
            return GestureState::Idle;
        }

        /*
         * 指がすべて離された。
         */
        if finger_count == 0 {
            logger.trace("finger_count: 2 -> 0");

            let elapsed = now.duration_since(started_at);

            let distance = Self::distance(start_center, last_center);

            if action == TwoFingerAction::Moving {
                return GestureState::Idle;
            }

            /*
             * Pending状態で離された場合のみタップ判定。
             *
             * 短時間 + 移動小 → LeftClick
             */
            if elapsed <= self.tap_threshold && distance <= self.movement_threshold {
                logger.trace(format!("LeftClick trrigered/action:{:?}", action));

                logger.trace(format!(
                    "2F: elapsed={:?}, start=({:.2},{:.2}), last=({:.2},{:.2}), \
                current={:?}, distance={:.2}, threshold={:.2}, action={:?}",
                    elapsed,
                    start_center.0,
                    start_center.1,
                    last_center.0,
                    last_center.1,
                    current_center,
                    distance,
                    self.movement_threshold,
                    action,
                ));

                return GestureState::TwoFinger {
                    started_at,
                    start_center,
                    last_center,
                    action: TwoFingerAction::LeftClick,
                };
            }

            /*
             * RightClickは長押し成立済みの場合。
             */
            if elapsed >= self.long_press_threshold && distance <= self.movement_threshold {
                return GestureState::TwoFinger {
                    started_at,
                    start_center,
                    last_center,
                    action: TwoFingerAction::RightClick,
                };
            }

            return GestureState::Idle;
        }

        /*
         * 2本指状態でない場合はIdleへ戻す。
         */
        if finger_count != 2 {
            return GestureState::Idle;
        }

        let center = current_center.unwrap();

        /*
         * ジェスチャ開始位置から現在位置までの移動量。
         */
        let distance = Self::distance(start_center, center);

        let elapsed = now.duration_since(started_at);

        /*
         * すでにMovingへ確定した場合。
         */
        if action == TwoFingerAction::Moving {
            return GestureState::TwoFinger {
                started_at,
                start_center,
                last_center: center,
                action: TwoFingerAction::Moving,
            };
        }
        /*
         * 移動量が閾値を超えたらMovingへ確定。
         */
        if distance > self.movement_threshold {
            return GestureState::TwoFinger {
                started_at,
                start_center,
                last_center: center,
                action: TwoFingerAction::Moving,
            };
        }

        /*
         * 長押し判定。
         *
         * 500ms経過 + 移動小
         * → RightClick
         */
        if elapsed >= self.long_press_threshold && distance <= self.movement_threshold {
            return GestureState::TwoFinger {
                started_at,
                start_center,
                last_center: center,
                action: TwoFingerAction::RightClick,
            };
        }

        //eprintln!("Pending @TowFinger");

        /*
         * まだタップなのか長押しなのか判断できない。
         */
        GestureState::TwoFinger {
            started_at,
            start_center,
            last_center: center,
            action: TwoFingerAction::Pending,
        }
    }

    fn process_three_finger(
        &self,
        finger_count: usize,
        now: Instant,
        started_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
        action: ThreeFingerAction,
        current_center: Option<(f64, f64)>,
        logger: &Logger,
    ) -> GestureState {
        // ----------------------------------------
        // 3本指ではなくなった
        // ----------------------------------------
        if finger_count == 0 {
            // 全指が離れた
            let elapsed = now.duration_since(started_at);
            let distance = Self::distance(start_center, last_center);

            if action == ThreeFingerAction::Scrolling {
                return GestureState::SuppressUntilRelease;
            }

            if action == ThreeFingerAction::Pending
                && elapsed <= self.tap_threshold
                && distance <= self.movement_threshold
            {
                logger.trace(format!(
                    "3F TAP: elapsed={:?}, distance={:.2}",
                    elapsed, distance,
                ));

                return GestureState::ThreeFinger {
                    started_at,
                    start_center,
                    last_center,
                    action: ThreeFingerAction::MiddleClick,
                };
            }

            return GestureState::Idle;
        }

        // 3本指から2本/1本になった
        if finger_count < 3 {
            match action {
                ThreeFingerAction::Pending => {
                    return GestureState::ThreeFingerReleasing {
                        started_at,
                        released_at: now,
                        start_center,
                        last_center,
                    };
                }

                ThreeFingerAction::Scrolling => {
                    return GestureState::SuppressUntilRelease;
                }

                ThreeFingerAction::MiddleClick => {
                    return GestureState::Idle;
                }
            }
        }

        // ----------------------------------------
        // ここから先は finger_count == 3
        // ----------------------------------------

        let center = current_center.unwrap();
        let distance = Self::distance(start_center, center);

        if action == ThreeFingerAction::Scrolling {
            return GestureState::ThreeFinger {
                started_at,
                start_center,
                last_center: center,
                action: ThreeFingerAction::Scrolling,
            };
        }

        if distance > self.movement_threshold {
            return GestureState::ThreeFinger {
                started_at,
                start_center,
                last_center: center,
                action: ThreeFingerAction::Scrolling,
            };
        }

        GestureState::ThreeFinger {
            started_at,
            start_center,
            last_center: center,
            action: ThreeFingerAction::Pending,
        }
    }

    fn process_three_finger_releasing(
        &self,
        finger_count: usize,
        now: Instant,
        started_at: Instant,
        released_at: Instant,
        start_center: (f64, f64),
        last_center: (f64, f64),
        logger: &Logger,
    ) -> GestureState {
        // 全指が離れた
        if finger_count == 0 {
            let elapsed = now.duration_since(started_at);
            let distance = Self::distance(start_center, last_center);

            if elapsed <= self.tap_threshold && distance <= self.movement_threshold {
                logger.trace(format!(
                    "3F TAP after release: elapsed={:?}, distance={:.2}",
                    elapsed, distance,
                ));

                return GestureState::ThreeFinger {
                    started_at,
                    start_center,
                    last_center,
                    action: ThreeFingerAction::MiddleClick,
                };
            }

            return GestureState::Idle;
        }

        // まだ指が残っている場合は、
        // 一定時間だけ「3本指を離している途中」として待つ
        let release_elapsed = now.duration_since(released_at);

        if release_elapsed <= self.release_grace_threshold {
            return GestureState::ThreeFingerReleasing {
                started_at,
                released_at,
                start_center,
                last_center,
            };
        }

        GestureState::Idle
    }

    fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;

        (dx * dx + dy * dy).sqrt()
    }

    fn to_mouse_events(
        &self,
        state: &GestureState,
        prev_state: &GestureState,
        dx: f64,
        dy: f64,
    ) -> Vec<MouseEvent> {
        match state {
            GestureState::TwoFinger {
                action: TwoFingerAction::Moving,
                ..
            } if dx != 0.0 || dy != 0.0 => {
                let (transformed_dx, transformed_dy) =
                    transform_delta(dx, dy, self.transform_direction);

                let (dx, dy) = (
                    transformed_dx * MOVE_RATE * self.move_sensitivity,
                    transformed_dy * MOVE_RATE * self.move_sensitivity,
                );
                vec![MouseEvent::Move { dx, dy }]
            }

            GestureState::TwoFinger {
                action: TwoFingerAction::LeftClick,
                ..
            } => {
                vec![
                    MouseEvent::Button {
                        button: MouseButton::Left,
                        pressed: true,
                    },
                    MouseEvent::Button {
                        button: MouseButton::Left,
                        pressed: false,
                    },
                ]
            }

            GestureState::TwoFinger {
                action: TwoFingerAction::RightClick,
                ..
            } => {
                if matches!(
                    prev_state,
                    GestureState::TwoFinger {
                        action: TwoFingerAction::Pending,
                        ..
                    }
                ) {
                    vec![
                        MouseEvent::Button {
                            button: MouseButton::Right,
                            pressed: true,
                        },
                        MouseEvent::Button {
                            button: MouseButton::Right,
                            pressed: false,
                        },
                    ]
                } else {
                    Vec::new()
                }
            }

            GestureState::ThreeFinger {
                action: ThreeFingerAction::Scrolling,
                ..
            } => {
                let (fixed_dx, fixed_dy) = transform_delta(dx, dy, self.transform_direction);
                let (fixed_dx, fixed_dy) = (fixed_dx / 10.0, fixed_dy / 10.0);
                vec![MouseEvent::Scroll {
                    dx: fixed_dx * SCROLL_RATE * self.scroll_sensitivity,
                    dy: fixed_dy * SCROLL_RATE * self.scroll_sensitivity,
                }]
            }

            GestureState::ThreeFinger {
                action: ThreeFingerAction::MiddleClick,
                ..
            } => {
                vec![
                    MouseEvent::Button {
                        button: MouseButton::Middle,
                        pressed: true,
                    },
                    MouseEvent::Button {
                        button: MouseButton::Middle,
                        pressed: false,
                    },
                ]
            }

            _ => Vec::new(),
        }
    }
}

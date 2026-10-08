use crate::{
    error::Error,
    output::event::{MouseButton, MouseEvent},
};
use evdev::{
    AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode, uinput::VirtualDevice,
};

pub struct VirtualMouse {
    device: VirtualDevice,

    move_remainder_x: f64,
    move_remainder_y: f64,

    scroll_remainder_x: f64,
    scroll_remainder_y: f64,
}
impl VirtualMouse {
    pub fn new() -> Result<Self, Error> {
        let mut axes = AttributeSet::<RelativeAxisCode>::new();

        axes.insert(RelativeAxisCode::REL_X);
        axes.insert(RelativeAxisCode::REL_Y);
        axes.insert(RelativeAxisCode::REL_WHEEL);
        axes.insert(RelativeAxisCode::REL_HWHEEL);

        let mut buttons = AttributeSet::<KeyCode>::new();

        buttons.insert(KeyCode::BTN_LEFT);
        buttons.insert(KeyCode::BTN_MIDDLE);
        buttons.insert(KeyCode::BTN_RIGHT);

        let device = VirtualDevice::builder()?
            .name(env!("CARGO_PKG_NAME"))
            .with_relative_axes(&axes)?
            .with_keys(&buttons)?
            .build()?;

        Ok(Self {
            device,
            move_remainder_x: 0.0,
            move_remainder_y: 0.0,
            scroll_remainder_x: 0.0,
            scroll_remainder_y: 0.0,
        })
    }

    pub fn emit(&mut self, event: MouseEvent) -> Result<(), Error> {
        match event {
            MouseEvent::Move { dx, dy } => {
                self.emit_move(dx, dy)?;
            }

            MouseEvent::Button { button, pressed } => {
                self.emit_button(button, pressed)?;
            }

            MouseEvent::Scroll { dx, dy } => {
                self.emit_scroll(dx, dy)?;
            }
        }

        Ok(())
    }

    fn emit_move(&mut self, dx: f64, dy: f64) -> Result<(), Error> {
        self.move_remainder_x += dx;
        self.move_remainder_y += dy;

        let x = self.move_remainder_x.trunc() as i32;
        let y = self.move_remainder_y.trunc() as i32;

        self.move_remainder_x -= x as f64;
        self.move_remainder_y -= y as f64;

        let mut events = Vec::with_capacity(2);

        if x != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_X.0,
                x,
            ));
        }

        if y != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_Y.0,
                y,
            ));
        }

        if !events.is_empty() {
            self.device.emit(&events)?;
        }

        Ok(())
    }

    fn emit_button(&mut self, button: MouseButton, pressed: bool) -> Result<(), Error> {
        let code = match button {
            MouseButton::Left => KeyCode::BTN_LEFT,
            MouseButton::Middle => KeyCode::BTN_MIDDLE,
            MouseButton::Right => KeyCode::BTN_RIGHT,
        };

        let value = if pressed { 1 } else { 0 };

        let event = InputEvent::new(EventType::KEY.0, code.0, value);

        self.device.emit(&[event])?;

        Ok(())
    }

    fn emit_scroll(&mut self, dx: f64, dy: f64) -> Result<(), Error> {
        self.scroll_remainder_x += dx;
        self.scroll_remainder_y += dy;

        let x = self.scroll_remainder_x.trunc() as i32;
        let y = self.scroll_remainder_y.trunc() as i32;

        self.scroll_remainder_x -= x as f64;
        self.scroll_remainder_y -= y as f64;

        let mut events = Vec::with_capacity(2);

        if x != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_HWHEEL.0,
                x,
            ));
        }

        if y != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_WHEEL.0,
                y,
            ));
        }

        if !events.is_empty() {
            self.device.emit(&events)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MouseEvent {
    Move { dx: f64, dy: f64 },

    Scroll { dx: f64, dy: f64 },

    Button { button: MouseButton, pressed: bool },
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

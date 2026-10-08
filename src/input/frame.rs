#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub slot: usize,
    pub tracking_id: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct TouchFrame {
    pub fingers: Vec<TouchPoint>,
}

impl TouchFrame {
    /// 現在触れている指の重心を返す。
    pub fn centroid(&self) -> Option<(f64, f64)> {
        if self.fingers.is_empty() {
            return None;
        }

        let x = self
            .fingers
            .iter()
            .map(|finger| finger.x as f64)
            .sum::<f64>()
            / self.fingers.len() as f64;

        let y = self
            .fingers
            .iter()
            .map(|finger| finger.y as f64)
            .sum::<f64>()
            / self.fingers.len() as f64;

        Some((x, y))
    }

    pub fn finger_count(&self) -> usize {
        self.fingers.len()
    }
}

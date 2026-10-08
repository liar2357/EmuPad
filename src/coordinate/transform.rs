use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
}

impl Transform {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Rotate90),
            2 => Some(Self::Rotate180),
            3 => Some(Self::Rotate270),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}
impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::from_u8(value).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid transform value: {value} (expected 0..=3)"))
        })
    }
}

pub fn transform_delta(dx: f64, dy: f64, transform: Transform) -> (f64, f64) {
    match transform {
        Transform::Normal => (dx, dy),

        Transform::Rotate90 => (-dy, dx),

        Transform::Rotate180 => (-dx, -dy),

        Transform::Rotate270 => (dy, -dx),
    }
}


pub enum Direction {
    Forward,
    Backward,
    Stopped,
}

pub trait Invert {
    fn invert(self) -> Self;
}

impl Invert for Direction {
    fn invert(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
            Direction::Stopped => Direction::Stopped,
        }
    }
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            1 => Direction::Forward,
            2 => Direction::Backward,
            _ => Direction::Stopped,
        }
    }
}
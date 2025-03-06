use crate::{GameStatus, Team};

impl GameStatus {
    pub fn valid_from_status(self) -> Self {
        match self {
            GameStatus::Started => GameStatus::Reset,
            GameStatus::Stopped => GameStatus::Started,
            GameStatus::Reset => GameStatus::Stopped,
        }
    }
}

impl Team {
    pub const TEAM_COUNT: usize = 3;
    pub fn all() -> [Self; Self::TEAM_COUNT] {
        [Self::Rust, Self::Js, Self::Vtl]
    }
}

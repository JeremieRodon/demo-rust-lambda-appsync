use crate::GameStatus;

impl GameStatus {
    pub fn valid_from_status(self) -> Self {
        match self {
            GameStatus::Started => GameStatus::Reset,
            GameStatus::Stopped => GameStatus::Started,
            GameStatus::Reset => GameStatus::Stopped,
        }
    }
}

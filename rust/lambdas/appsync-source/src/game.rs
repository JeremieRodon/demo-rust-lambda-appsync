use crate::GameStatus;

impl GameStatus {
    /// Returns the allowed current game status when transitioning to a new status.
    ///
    /// The game status can only transition in a specific order:
    /// - Reset -> Started -> Stopped -> Reset
    ///
    /// This method returns what the current status must be to allow transitioning
    /// to the target status (self).
    pub fn valid_from_status(self) -> Self {
        match self {
            GameStatus::Started => GameStatus::Reset,
            GameStatus::Stopped => GameStatus::Started,
            GameStatus::Reset => GameStatus::Stopped,
        }
    }
}

use crate::chess::Color;
use tokio::sync::mpsc;

/// Channel types for move communication between TUI and Coordinator
pub type MoveSender = mpsc::Sender<crate::chess::Move>;
pub type MoveReceiver = mpsc::Receiver<crate::chess::Move>;

/// Player enum with static dispatch
///
/// This enum allows different player types to be handled without trait objects.
/// Adding new player types (e.g., NetworkPlayer, EnginePlayer) just requires
/// adding a new variant - the compiler will flag all places that need updating.
pub enum Player {
    Hotseat(HotseatPlayer),
    // Future variants:
    // Network(NetworkPlayer),
    // Engine(EnginePlayer),
}

impl Player {
    /// Get the player's color
    pub fn color(&self) -> Color {
        match self {
            Player::Hotseat(p) => p.color,
        }
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::Hotseat(_) => write!(f, "Hotseat({:?})", self.color()),
        }
    }
}

/// Hotseat player for local two-player games
///
/// For hotseat play, both players share the same move_rx channel.
/// The Coordinator trusts the TUI to enforce turn order based on the
/// `GameSnapshot.turn` field.
#[derive(Debug, Clone)]
pub struct HotseatPlayer {
    pub color: Color,
}

impl HotseatPlayer {
    /// Create a new hotseat player
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

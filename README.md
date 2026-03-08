# borrow-checkmate

A terminal-based chess game built with [Ratatui], featuring hotseat multiplayer and a clean interface for future network play.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen/badge.svg)

## Features

- **Hotseat Multiplayer** - Play chess locally with a friend on the same terminal
- **Intuitive Controls** - Arrow key navigation with Enter to select/confirm moves
- **Visual Feedback** - Unicode chess pieces, move highlighting, legal move hints
- **Undo Support** - Take back moves with `u`
- **Pawn Promotion** - Choose queen, rook, bishop, or knight when pawns reach the last rank
- **Game State Display** - Turn indicator, captured pieces, check/checkmate status
- **Clean Architecture** - Pure TUI, game logic in background coordinator - ready for network play

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd borrow-checkmate

# Run the game
cargo run --release
```

## How to Play

### Basic Navigation

The game uses arrow keys to move the cursor and Enter to interact:

| Key | Action |
|-----|---------|
| `↑` `↓` `←` `→` | Move the cursor around the board |
| `Enter` | Select a piece / Confirm a move |
| `Esc` | Deselect piece / Cancel promotion |
| `u` | Undo last move |
| `n` | Start a new game |
| `q` | Quit the game |
| `Ctrl-C` | Force quit |

### Making a Move

1. **Move the cursor** to the piece you want to move using arrow keys
2. **Select the piece** - Press `Enter` when the cursor is on your piece
   - The piece will be highlighted in yellow
   - Legal move destinations will be shown as dots (`·`) on squares
3. **Choose destination** - Move the cursor to the target square
4. **Confirm the move** - Press `Enter` again to complete the move

### Pawn Promotion

When a pawn reaches the last rank (rank 1 for white, rank 8 for black):

1. A promotion dialog appears with four options: Q, R, B, N
2. Use `←` and `→` to select the piece type
3. Press `Enter` to confirm your choice
4. The full move is then sent to the game coordinator

### Visual Indicators

- **Cursor** - Green highlight on the current square
- **Selected Piece** - Yellow highlight on the piece you've picked up
- **Legal Moves** - Small dots (`·`) on squares where the selected piece can move
- **Last Move** - Highlighted squares showing the previous move (from → to)
- **Check** - King turns red when in check
- **Illegal Move** - "Illegal move!" flashes in red when you attempt an invalid move

### Game Over

The game ends when:
- **Checkmate** - One player's king is in check and has no legal moves
- **Stalemate** - No player has a legal move but no king is in check
- **Insufficient Material** - Neither player has enough pieces to checkmate

The result is displayed in the info panel, and you can start a new game with `n`.

## Game Rules

Standard chess rules apply:

- **Movement** - All pieces move according to standard chess rules
- **Captures** - Landing on an opponent's piece removes it from the board
- **Check** - Your king cannot move into check
- **Checkmate** - Win by placing the opponent's king in check with no escape
- **Castling** - Kings and rooks can castle (available when conditions met)
- **En Passant** - Pawns can capture en passant on the first move after the opponent advances two squares
- **Promotion** - Pawns reaching the last rank must be promoted

### Turn Order

- White always moves first
- Turns alternate between White and Black
- The info panel shows whose turn it is ("White's turn" / "Black's turn")
- You can only move pieces of the color whose turn it is

## Building for Release

The project is optimized for release builds:

```bash
cargo build --release
```

Release builds are smaller and faster due to LTO and codegen optimizations.

## Architecture

The game is built with a clean separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│  TUI Thread (main)                               │
│                                                     │
│  • Renders board and UI                             │
│  • Handles keyboard input                          │
│  • Forwards confirmed moves to Coordinator            │
└─────────────────────────────────────────────────────────┘
                     ▲ mpsc::channel
                     │
┌─────────────────────────────────────────────────────────┐
│  Game Coordinator (tokio::spawn)                  │
│                                                     │
│  • Owns game state (shakmaty wrapper)          │
│  • Validates moves                                 │
│  • Detects check/checkmate                         │
│  • Broadcasts state updates back to TUI             │
└─────────────────────────────────────────────────────────┘
```

This design allows:
- **Future network play** - Add `NetworkPlayer` variant without changing game logic
- **Chess engine integration** - Add `EnginePlayer` for AI opponents
- **Hotseat-first** - Clean, simple gameplay for immediate use

## Dependencies

- [ratatui](https://ratatui.rs) - Terminal UI framework
- [shakmaty](https://docs.rs/shakmaty) - Chess rules and move generation
- [crossterm](https://github.com/crossterm/crossterm) - Terminal handling
- [tokio](https://tokio.rs) - Async runtime

## License

Copyright (c) G36maid <G36_maid@proton.me>

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>)

## Contributing

The architecture is designed for extensibility. To add features:

- **Network Play** - Implement `NetworkPlayer` in `src/player.rs` and wire WebSocket/HTTP client
- **Engine Player** - Implement `EnginePlayer` that communicates via UCI protocol
- **Sound Effects** - Add audio feedback for captures and game events
- **Move History** - Enhance the info panel to show full move list with algebraic notation

The game coordinator and UI are completely separate - you can add these features without touching each other's code.

---

Built with [Ratatui](https://ratatui.rs) using the [event-driven async template](https://github.com/ratatui/templates/tree/main/event-driven-async).

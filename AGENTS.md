# AGENTS.md

AI Agent Guidelines for `borrow-checkmate` - A TUI Chess Game

---

## Project Overview

**Language**: Rust 2024 Edition
**Domain**: Terminal UI (TUI) chess game using ratatui
**Core Dependencies**:
- `shakmaty` 0.30 - Chess engine (position, moves, rules)
- `ratatui` 0.30 - Terminal UI framework
- `tokio` 1.40.0 - Async runtime
- `crossterm` 0.28.1 - Terminal input handling
- `color-eyre` 0.6.3 - Error handling
- `futures` 0.3.31 - Async utilities

**Architecture**: 
- **Coordinator Pattern**: Async tokio task manages game state
- **Channel-based**: TUI sends `CoordinatorCommand` via mpsc to coordinator
- **Separation of Concerns**: TUI (input/render), Coordinator (game logic), UI modules (presentation)
- **No Arc<Mutex<>>**: All state communication via channels

---

## Build Commands

### Standard Development
```bash
# Check code compiles
cargo check

# Format code (uses rustfmt.toml if exists, or rustfmt defaults)
cargo fmt

# Run linter
cargo clippy

# Build for development
cargo build

# Build optimized release (LTO enabled)
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run clippy with all targets
cargo clippy --all-targets
```

### Release Profile

The project uses a `release` profile with optimizations:
```toml
[profile.release]
codegen-units = 1  # Better inlining
lto = true          # Link-time optimization
opt-level = "s"      # Size over speed
strip = true          # Remove debug symbols
```

**Note**: The optimization guideline comment in `Cargo.toml` references: https://ratatui.rs/recipes/apps/release-your-app/#optimizations

---

## Code Style Guidelines

### Import Organization

**Pattern**: External crate imports first, then local modules, organized by concern

```rust
// Standard ordering
use std::fmt;

use tokio::sync::mpsc;

// External crates
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RColor, Style},
    text::Line,
    widgets::Widget,
};

// Local modules (alphabetical within each layer)
use crate::chess::{Board, Color, Move, Piece, Role, Square};
use crate::event::{AppEvent, Event};
use crate::player::{HotseatPlayer, Player};
use crate::ui;
```

**Style aliases**:
```rust
// Import with alias for colors to avoid namespace conflicts
use ratatui::style::{Color as RColor, Style};
```

### Error Handling

**Pattern**: Use `color_eyre::Result` at boundaries, propagate errors up

```rust
// Main entry point - application boundary
pub async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // ... application code
    Ok(())
}

// Game logic - uses shakmaty's PlayError internally
pub fn make_move(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
    self.history.push(self.pos.clone());
    let new_pos = self.pos.clone().play(mv)?;
    self.pos = new_pos;
    Ok(())
}

// Channel operations - ignore send errors (TUI side)
let _ = self.cmd_tx.try_send(CoordinatorCommand::MakeMove(mv));
```

**Rules**:
1. Use `?` operator to propagate `Result` errors at boundaries
2. Use `unwrap()` only when you're certain the value is `Some` or `Ok`
3. Use `unwrap_or_default()` for providing defaults from `Option`
4. Don't suppress errors with `if let Err(_) = {}` - log them instead
5. Use `color_eyre::eyre!()` or similar for constructing error variants

### Async Patterns

**Pattern**: Tokio runtime with channels for async coordination

```rust
// Bounded channel with capacity 1 (enforces backpressure)
let (cmd_tx, cmd_rx) = mpsc::channel::<CoordinatorCommand>(1);

// Unbounded channel for events (no backpressure needed)
let events = EventHandler::new();
let app_tx = events.sender();

// Spawn coordinator task
tokio::spawn(coordinator::run(cmd_rx, app_tx));

// Coordinator loop - receives commands and broadcasts state
loop {
    match self.cmd_rx.recv().await {
        None => break,
        Some(cmd) => match cmd {
            CoordinatorCommand::MakeMove(mv) => {
                // Process move
                self.broadcast_state();
            }
            CoordinatorCommand::Undo => {
                // Undo last move
                self.game.undo();
                self.broadcast_state();
            }
        }
    }
}
```

**Rules**:
1. Use `await` for all async operations
2. Prefer bounded channels for command patterns (capacity 1)
3. Prefer unbounded channels for events (no blocking)
4. Always spawn coordinator in main before running App
5. Use `.await` on `tokio::spawn()` for tasks that need to run to completion
6. Channel send operations use `try_send()` or block with `.send().await`

### Naming Conventions

**Types**: PascalCase
- `Game`, `Player`, `HotseatPlayer`, `Coordinator`, `GameScreen`
- `GameSnapshot`, `CoordinatorCommand`, `ChessBoard`, `InfoPanel`, `PromotionDialog`

**Enums**: PascalCase
- `AppEvent`, `CoordinatorCommand`, `Color`, `Role`, `Square`

**Functions**: snake_case
- `new()`, `render()`, `make_move()`, `undo()`, `broadcast_state()`
- `legal_moves_from()`, `handle_key()`, `handle_crossterm()`, `route()`

**Constants**: SCREAMING_SNAKE_CASE (all caps)
- N/A in this codebase (mostly uses literals directly)

**Variables**: snake_case
- `rank`, `file`, `sq`, `mv`, `game`, `snapshot`, `cmd_tx`, `cmd_rx`
- `turn_piece`, `turn_text`, `white_move`, `black_move`

**Type Aliases**: PascalCase for clarity
- `App`, `Screen`, `Square`, `Color`, `Role`, `Move`, `Piece`, `Outcome`

### Type Annotations

**Pattern**: Explicit type annotations in function signatures, inferred in bodies

```rust
// Function signature - explicit types
pub fn new(events: EventHandler, cmd_tx: mpsc::Sender<CoordinatorCommand>) -> Self {
    Self {
        running: true,
        events,
        cmd_tx,
        screen: Screen::Game(GameScreen::new()),
    }
}

// Method signature - explicit return type
pub fn make_move(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
    self.history.push(self.pos.clone());
    // ... method body
}

// Local variables - type inferred where obvious
let rank_height = area.height.saturating_sub(2) / 8;
let file_width = area.width.saturating_sub(3) / 8;

// Use `as` for casts when type is obvious
let sq = Square::new((rank * 8 + file) as u32);
```

**Rules**:
1. Always annotate function signatures with parameter and return types
2. Annotate struct fields
3. Use `as` casts for numeric-to-numeric conversions where the target type is clear
4. Use `::<Type>` turbofish syntax for type parameters when needed
5. Don't use unnecessary `: <Type>` type annotations

### Memory Management

**Pattern**: Clone when ownership needs to be transferred, prefer references otherwise

```rust
// Clone when storing for undo history
self.history.push(self.pos.clone());

// Clone for snapshot (sent to TUI)
GameSnapshot {
    pub board: Board,           // Clone cheap (reference)
    pub legal_moves: MoveList,   // Stack-allocated ArrayVec (Copy)
    pub move_history: Vec<Move>, // Clone to send
}

// Reference for local iteration (no clone needed)
for sq in Square::ALL {
    // sq is Copy, so this is cheap
}

// Use `&` borrows where possible
pub fn render(self, area: Rect, buf: &mut Buffer) {
    let Some(snapshot) = self.game.snapshot() else {
        return;
    };
    // Use snapshot by reference
}
```

**Rules**:
1. Use `Clone` trait when you need to copy data
2. Prefer `&` references when borrowing for reads
3. Understand which shakmaty types are `Copy` vs `Clone`:
   - `Square`, `Color`, `Role`, `Piece`, `File`, `Rank` are `Copy`
   - `Move`, `Board`, `Chess`, `Position` are `Clone`
4. Avoid unnecessary clones in hot paths (render loops, event handlers)

### Constants and Configuration

**Pattern**: Use literals or const constructors, avoid magic numbers

```rust
// Square construction - use shakmaty API
let sq = Square::new((rank * 8 + file) as u32);

// Color scheme - use explicit RGB values
RColor::Rgb(240, 217, 181)  // Light square
RColor::Rgb(181, 136, 99)   // Dark square

// Channel capacity - use constant
let (cmd_tx, cmd_rx) = mpsc::channel::<CoordinatorCommand>(1);

// Unicode piece symbols - use literal strings
"♔", "♕", "♖", "♗", "♘", "♙"
```

**Rules**:
1. Use shakmaty's `Square::new()` instead of manual 64-arm match
2. Define color constants as `RColor::Rgb()` for RGB values
3. Don't use magic numbers for board dimensions or colors
4. Use named constants for channel capacities
5. Unicode literals are fine for chess pieces

---

## Project Structure

```
src/
├── main.rs              # Entry point: channel setup, coordinator spawn
├── event.rs             # AppEvent enum + EventHandler
├── app.rs               # Screen router: TUI → events
├── app/
│   └── screen.rs       # GameScreen state + input handling
├── coordinator.rs       # Game state: async task, command handling
├── chess.rs             # shakmaty wrapper: Game + GameSnapshot
├── player.rs            # Player enum + HotseatPlayer
└── ui/
    ├── mod.rs           # UI dispatcher
    ├── board.rs         # Board widget (8x8 grid)
    ├── info.rs           # Info panel (turn, captures, history)
    └── promotion.rs      # Promotion dialog overlay
```

**Architecture Principles**:
1. **No game logic in TUI** - App and GameScreen only route/render
2. **Single source of truth** - Coordinator owns `Game`, broadcasts state
3. **Pure presentation** - UI modules only render, no logic
4. **Channel boundaries** - TUI→Coordinator is bounded (capacity 1), Coordinator→TUI is unbounded

---

## Common Patterns

### Board Rendering

**Pattern**: 8x8 grid with rank/file labels, unicode pieces, color coding

```rust
// Coordinate calculation (shakmaty Square)
let sq = Square::new((rank * 8 + file) as u32);

// Color calculation (light/dark checkerboard)
fn square_color(rank: u8, file: u8) -> RColor {
    if (rank + file) % 2 != 0 {
        RColor::Rgb(240, 217, 181)  // Light
    } else {
        RColor::Rgb(181, 136, 99)   // Dark
    }
}

// Unicode piece mapping
fn piece_char(piece: Piece) -> &'static str {
    match (piece.color, piece.role) {
        (Color::White, Role::King) => "♔",
        (Color::White, Role::Queen) => "♕",
        // ... etc
    }
}
```

### Event Routing

**Pattern**: App receives events, routes to screen, sends commands to coordinator

```rust
// Main loop in App::run()
while self.running {
    terminal.draw(|f| ui::render(&self.screen, f))?;

    match self.events.next().await? {
        Event::Tick => {}
        Event::Crossterm(e) => self.handle_crossterm(e),
        Event::App(e) => self.route(e),
    }
}

// Event routing in route()
fn route(&mut self, event: AppEvent) {
    match event {
        AppEvent::StateUpdate(snap) => self.screen.apply_snapshot(snap),
        AppEvent::GameOver(outcome) => self.screen.set_game_over(outcome),
        AppEvent::Quit => self.running = false,
        AppEvent::NewGame => self.reset(),
    }
}
```

### State Broadcasts

**Pattern**: Coordinator sends `StateUpdate` after each state change

```rust
impl Coordinator {
    fn broadcast_state(&self) {
        self.inject(AppEvent::StateUpdate(self.game.snapshot()));
    }

    fn run(mut self) {
        self.broadcast_state();  // Initial state

        loop {
            match self.cmd_rx.recv().await {
                None => break,
                Some(cmd) => {
                    // Process command
                    self.broadcast_state();  // Always broadcast after change
                }
            }
        }
    }
}
```

---

## Anti-Patterns to Avoid

### ❌ Don't Suppress Errors

```rust
// Bad
if let Err(_) = self.cmd_tx.try_send(cmd) {
    // Error silently ignored
}

// Good
if let Err(e) = self.cmd_tx.try_send(cmd) {
    eprintln!("Failed to send command: {}", e);
}
```

### ❌ Don't Clone Unnecessarily

```rust
// Bad - cloning when reference works
fn render(&self, buf: &mut Buffer) {
    let snapshot = self.game.snapshot().unwrap().clone();
    // ... use snapshot
}

// Good - use reference
fn render(&self, buf: &mut Buffer) {
    let Some(snapshot) = self.game.snapshot() else {
        return;
    };
    // ... use snapshot by reference
}
```

### ❌ Don't Use Unsafe Without Justification

```rust
// Bad - unsafe transmute (avoided in recent fixes)
let sq = unsafe { std::mem::transmute((rank * 8 + file) as u8) };

// Good - use shakmaty API
let sq = Square::new((rank * 8 + file) as u32);
```

### ❌ Don't Mix Concerns

```rust
// Bad - UI module doing game logic
pub fn render(&self, buf: &mut Buffer) {
    if self.game.is_check() {
        // Game logic in UI module
    }
}

// Good - UI only renders state
pub fn render(&self, buf: &mut Buffer) {
    if snapshot.is_check {
        // Display check indicator
    }
}
```

---

## Cursor Rules & Configuration

**Status**: No `.cursorrules` or `.cursorrules` file found in this repository.

This means agents should follow the standard guidelines in this AGENTS.md without additional custom cursor rules.

---

## Testing Guidelines

### Test Organization

The project structure does not currently include a `tests/` directory. Tests should be added as:

```
tests/
├── integration/    # End-to-end tests
├── unit/           # Unit tests for individual modules
└── fixtures/        # Test data and helper functions
```

### Writing Tests

```rust
// Example unit test for chess.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_new() {
        let game = Game::new();
        let snapshot = game.snapshot();
        assert_eq!(snapshot.turn, Color::White);
    }

    #[test]
    fn test_square_construction() {
        let sq = Square::new(0);  // A1
        assert_eq!(sq, Square::A1);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

---

## Shakmaty Usage Notes

### Square Construction

**ALWAYS use shakmaty API** instead of manual matches:

```rust
// Correct
let sq = Square::new((rank * 8 + file) as u32);

// NOT this (removed in recent fixes)
let sq = unsafe { std::mem::transmute((rank * 8 + file) as u8) };
fn from_index(index: u8) -> Square { /* 64-arm match */ }
```

**Rationale**: `Square::new()` is idiomatic, safe, and optimized. It panics on invalid indices (which is fine since we guarantee valid indices).

### Board Iteration

Use `(0..64)` or `Square::ALL` when iterating:

```rust
// Good - iterator over all squares
for sq in Square::ALL {
    // Process each square
}

// Alternative - numeric range
for rank in 0..8u8 {
    for file in 0..8u8 {
        let sq = Square::new((rank * 8 + file) as u32);
    }
}
```

### Legal Moves

```rust
// Get all legal moves
let legal_moves = snapshot.legal_moves();  // Returns MoveList (ArrayVec)

// Filter by source square
let from_selected: MoveList = game
    .legal_moves_from(selected_square);
```

### Type Information

**Copy types** (cheap to clone): `Square`, `Color`, `Role`, `Piece`, `File`, `Rank` are `Copy`
**Clone types** (must be cloned): `Move`, `Board`, `Chess`, `Position`, `GameSnapshot`, `Game`, `MoveList` are `Clone`

**MoveList**: Stack-allocated `ArrayVec` (no heap allocation)

---

## Common Patterns

### Board Rendering

**Pattern**: 8x8 grid with rank/file labels, unicode pieces, color coding

```rust
// Coordinate calculation (shakmaty Square)
let sq = Square::new((rank * 8 + file) as u32);

// Color calculation (light/dark checkerboard)
fn square_color(rank: u8, file: u8) -> RColor {
    if (rank + file) % 2 != 0 {
        RColor::Rgb(240, 217, 181) // Light
    } else {
        RColor::Rgb(181, 136, 99)   // Dark
    }
}

// Unicode piece mapping
fn piece_char(piece: Piece) -> &'static str {
    match (piece.color, piece.role) {
        (Color::White, Role::King) => "♔",
        (Color::White, Role::Queen) => "♕",
        (Color::White, Role::Rook) => "♖",
        // ... etc
    }
}
```

**Solution**: For bounded channel with capacity 1, this is acceptable. If you need guaranteed delivery:

```rust
self.cmd_tx.send(CoordinatorCommand::MakeMove(mv)).await?;
```

### Issue: Clone in Hot Path

**Problem**: Cloning `Board` in render loop is expensive

```rust
for rank in 0..8u8 {
    for file in 0..8u8 {
        let board = snapshot.board().clone();  // Expensive!
    }
}
```

**Solution**: Borrow when possible

```rust
let snapshot = self.game.snapshot() else { return };

for rank in 0..8u8 {
    for file in 0..8u8 {
        // Use snapshot.board directly - no clone needed
    }
}
```

### Issue: Missing Error Context

**Problem**: Silent error handling makes debugging hard

```rust
if let Err(_) = self.game.make_move(mv) {
    // What went wrong?
}
```

**Solution**: Log or report errors

```rust
if let Err(e) = self.game.make_move(mv) {
    self.inject(AppEvent::IllegalMove);
}
```

---

## Version Compatibility Notes

**Current shakmaty version**: 0.30.0

The codebase uses shakmaty 0.30. If upgrading:
- Check breaking changes in shakmaty changelog
- `Square::new()` API is stable
- `Position::legal_moves()` API is stable
- `Board::piece_at()` API is stable

---

## Git Conventions

### Commit Message Style

Follow Conventional Commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `docs:` - Documentation changes
- `test:` - Test changes
- `chore:` - Maintenance tasks

Examples:
```
fix(ui/board): correct rank label alignment and square color parity
feat(coordinator): add CoordinatorCommand enum with Undo and NewGame
refactor(app): wire CoordinatorCommand channel; add undo key handling
```

### Commit Frequency

- Atomic commits - one logical change per commit
- Don't batch unrelated changes
- Test after each significant change
- Build passes before committing

---

## Agent-Specific Guidelines

### When Working on UI

1. **Always check** `snapshot.is_some()` before using
2. **Don't add game logic** - UI is purely presentational
3. **Use shakmaty types directly** - no rewrapping needed
4. **Follow color scheme** - consistent RGB values
5. **Handle promotion** - check `promotion_pending()` state

### When Working on Game Logic

1. **Always clone before modification** - `self.pos.clone()`
2. **Update history consistently** - push to both `history` and `move_history`
3. **Recompute state on undo** - use undo logic in chess.rs
4. **Check outcomes** - `is_game_over()`, `outcome()`, `is_check()`

### When Working on Coordinator

1. **Broadcast state changes** - always after `make_move`, `undo`, `new_game`
2. **Handle errors gracefully** - inject `IllegalMove` on play errors
3. **Use bounded channels** - capacity 1 for backpressure
4. **Don't block** - use `try_send()` for fire-and-forget

---

## Tools and Commands Reference

### Cargo Commands

```bash
cargo check            # Quick compile check
cargo build            # Debug build
cargo build --release # Optimized build
cargo test             # Run tests
cargo clippy           # Run linter
cargo fmt              # Format code
cargo update            # Update dependencies
cargo clean            # Clean build artifacts
```

### Useful Aliases

Consider adding to `~/.bashrc` or shell configuration:

```bash
alias cb='cargo build --release'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt && cargo clippy'
```

---

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Ratatui Docs**: https://docs.rs/ratatui/
- **Shakmaty Docs**: https://docs.rs/shakmaty/
- **Tokio Docs**: https://tokio.rs/tokio/
- **Crossterm Docs**: https://docs.rs/crossterm/
- **Color-eyre Docs**: https://docs.rs/color-eyre/

---

*Last Updated*: 2026-03-09
*Maintained By*: G36maid <G36_maid@proton.me>

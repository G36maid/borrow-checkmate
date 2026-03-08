# borrow-checkmate — Implementation TODO

A ratatui TUI chess game. Hotseat (local two-player) first. Architecture is
designed for future network play without changing the game engine or coordinator.

---

## Workflow

**Complete each task, then commit before moving to the next.**

```
# After finishing each TODO item:
git add -A
git commit -m "<scope>: <what was done>"
```

Suggested commit scopes that map to the task list below:

| Task | Suggested commit message |
|------|--------------------------|
| 1. Cargo.toml | `deps: add shakmaty 0.30` |
| 2. chess.rs | `feat: add chess.rs shakmaty wrapper with undo history` |
| 3. player.rs | `feat: add player.rs enum with HotseatPlayer` |
| 4. coordinator.rs | `feat: add game coordinator tokio task` |
| 5. event.rs | `refactor: extend AppEvent for chess state updates` |
| 6. app.rs | `refactor: app.rs as pure screen router` |
| 7. main.rs | `refactor: wire channels and spawn coordinator in main` |
| 8. ui/mod.rs | `feat: add ui module with screen dispatch` |
| 9. ui/board.rs | `feat: add board widget with unicode pieces and highlights` |
| 10. ui/info.rs | `feat: add info panel (turn, captures, move history)` |
| 11. ui/promotion.rs | `feat: add promotion overlay dialog` |

Keep commits atomic — one task per commit. Do not batch multiple tasks into one commit.

---

## Concurrency Model

```
┌──────────────────────────────────────────────────────────┐
│  TUI Thread (tokio main)                                  │
│                                                           │
│  crossterm events                                         │
│       │                                                   │
│       ▼                                                   │
│  App::handle_crossterm()                                  │
│       │  confirmed Move                                   │
│       └─────────────────────────────► move_tx.send(mv)   │
│                                                           │
│  app_event_rx ◄── Event::App(AppEvent::StateUpdate)      │
│       │        ◄── Event::App(AppEvent::GameOver)         │
│       │        ◄── Event::App(AppEvent::IllegalMove)      │
│       ▼                                                   │
│  App::route(event) → update Screen state                  │
│       │                                                   │
│  terminal.draw(|f| ui::render(&screen, f))               │
└──────────────────────────────────────────────────────────┘
                        ▲ Event injection (app_event_tx clone)
                        │
┌──────────────────────────────────────────────────────────┐
│  Game Coordinator (tokio::spawn)                          │
│                                                           │
│  loop {                                                   │
│      mv = move_rx.recv().await                            │
│      game.make_move(mv)                                   │
│      broadcast StateUpdate snapshot                       │
│      if game_over → inject GameOver, break                │
│  }                                                        │
└──────────────────────────────────────────────────────────┘
```

**Rules:**
- TUI never contains game logic. It only renders state and forwards input.
- Coordinator never touches the terminal. It only injects `AppEvent` via channel.
- No `Arc<Mutex<>>`. All communication is via channels.
- Moves sent to Coordinator are always legal (TUI filters by snapshot's `legal_moves`).
- Promotion role is resolved in TUI before the `Move` is sent. Coordinator never stalls.

---

## Channel Topology

```rust
// TUI → Coordinator  (bounded, capacity 1 — backpressure prevents double-send)
move_tx: mpsc::Sender<Move>
move_rx: mpsc::Receiver<Move>

// Coordinator → TUI  (reuse EventHandler's unbounded sender)
app_event_tx: mpsc::UnboundedSender<Event>   // clone of EventHandler::sender
```

---

## File Structure

```
src/
├── main.rs              // wire channels, spawn coordinator, run TUI
├── event.rs             // Event + AppEvent enums + EventHandler
├── app.rs               // Screen router — render + input dispatch only
├── coordinator.rs       // Game Coordinator (tokio task)
├── chess.rs             // shakmaty wrapper + move history Vec<Chess>
├── player.rs            // Player enum + HotseatPlayer (static dispatch)
└── ui/
    ├── mod.rs           // ui::render(screen, frame) dispatch
    ├── board.rs         // 8×8 board Widget, highlights, unicode pieces
    ├── info.rs          // turn indicator, captured pieces, move history panel
    └── promotion.rs     // promotion role selection overlay
```

---

## Tasks

### [ ] 1. `Cargo.toml` — Add shakmaty

```toml
[dependencies]
shakmaty = "0.30"
# existing deps unchanged:
# crossterm, futures, ratatui, tokio, color-eyre
```

**Notes:**
- No extra feature flags needed for standard chess.
- `shakmaty::MoveList` is `arrayvec::ArrayVec` — stack allocated, no heap cost.

---

### [ ] 2. `chess.rs` — Shakmaty Wrapper

**Purpose:** Thin wrapper around `shakmaty::Chess`. Adds move history for undo
(shakmaty has no built-in undo). Exposes a clean API so the rest of the codebase
never imports shakmaty directly.

**Re-exports (no wrapping needed):**
```rust
pub use shakmaty::{Board, Color, Move, Outcome, Piece, Role, Square};
pub use shakmaty::MoveList;
```

**`GameSnapshot` — sent to TUI on every state change:**
```rust
#[derive(Clone, Debug)]
pub struct GameSnapshot {
    pub board: Board,
    pub turn: Color,
    pub is_check: bool,
    pub legal_moves: MoveList,
    pub last_move: Option<Move>,
    pub outcome: Option<Outcome>,
    pub move_history: Vec<Move>,         // for info panel display
    pub captured_white: Vec<Role>,       // pieces white has captured
    pub captured_black: Vec<Role>,       // pieces black has captured
}
```

**`Game` struct:**
```rust
pub struct Game {
    pos: Chess,
    history: Vec<Chess>,    // cloned snapshots for undo (Chess: Clone)
    move_history: Vec<Move>,
    captured_white: Vec<Role>,
    captured_black: Vec<Role>,
}

impl Game {
    pub fn new() -> Self;
    pub fn snapshot(&self) -> GameSnapshot;

    pub fn legal_moves_from(&self, sq: Square) -> MoveList;
    // filters legal_moves() to only moves originating from `sq`

    pub fn make_move(&mut self, mv: Move) -> Result<(), PlayError<Chess>>;
    // 1. push self.pos.clone() onto history
    // 2. track capture if mv.capture().is_some()
    // 3. self.pos = self.pos.clone().play(mv)?
    // 4. push mv to move_history

    pub fn undo(&mut self) -> bool;
    // pop history stack, restore pos; returns false if nothing to undo
    // also pop move_history and recompute captured lists

    pub fn outcome(&self) -> Option<Outcome>;
    // None if game still in progress
}
```

**Error handling:** Return `color_eyre::Result` at the boundary; internally use
`shakmaty::PlayError`.

---

### [ ] 3. `player.rs` — Player Enum (Static Dispatch)

**Purpose:** Represent players without trait objects. Enum variants are exhaustive
at compile time. Adding `NetworkPlayer` later just adds one variant and the
compiler flags every unhandled match arm.

```rust
use crate::chess::{Color, Move};
use tokio::sync::mpsc;

pub type MoveSender   = mpsc::Sender<Move>;
pub type MoveReceiver = mpsc::Receiver<Move>;

pub enum Player {
    Hotseat(HotseatPlayer),
    // Future:
    // Network(NetworkPlayer),
    // Engine(EnginePlayer),
}

impl Player {
    pub fn color(&self) -> Color {
        match self {
            Player::Hotseat(p) => p.color,
        }
    }
}

pub struct HotseatPlayer {
    pub color: Color,
}

impl HotseatPlayer {
    pub fn new(color: Color) -> Self { Self { color } }
}
```

**Notes:**
- `HotseatPlayer` has no `get_move()` — moves arrive via `move_rx` in the
  Coordinator. The Coordinator trusts the TUI's turn enforcement.
- A future `NetworkPlayer` would hold a connection handle and implement
  `async fn get_move(&mut self, move_rx: &mut MoveReceiver) -> Move` using
  `tokio::select!` between the socket and a local override channel.

---

### [ ] 4. `coordinator.rs` — Game Coordinator

**Purpose:** Owns the `Game` and both `Player`s. Runs as a detached tokio task.
Receives moves from TUI via `move_rx`, applies them, broadcasts state snapshots
back by injecting `AppEvent` into the TUI's event loop.

```rust
use crate::chess::{Game, Move};
use crate::event::{AppEvent, Event};
use crate::player::{MoveReceiver, Player};
use tokio::sync::mpsc;

pub struct Coordinator {
    game: Game,
    white: Player,
    black: Player,
    move_rx: MoveReceiver,
    app_tx: mpsc::UnboundedSender<Event>,
}

impl Coordinator {
    pub fn new(
        move_rx: MoveReceiver,
        app_tx: mpsc::UnboundedSender<Event>,
    ) -> Self {
        Self {
            game: Game::new(),
            white: Player::Hotseat(HotseatPlayer::new(Color::White)),
            black: Player::Hotseat(HotseatPlayer::new(Color::Black)),
            move_rx,
            app_tx,
        }
    }

    pub async fn run(mut self) {
        self.broadcast_state();   // send initial board position to TUI

        loop {
            match self.move_rx.recv().await {
                None => break,    // move_tx dropped (TUI quit)
                Some(mv) => {
                    match self.game.make_move(mv) {
                        Ok(()) => {
                            self.broadcast_state();
                            if self.game.outcome().is_some() {
                                self.inject(AppEvent::GameOver(
                                    self.game.outcome().unwrap()
                                ));
                                break;
                            }
                        }
                        Err(_) => {
                            self.inject(AppEvent::IllegalMove);
                        }
                    }
                }
            }
        }
    }

    fn broadcast_state(&self) {
        self.inject(AppEvent::StateUpdate(self.game.snapshot()));
    }

    fn inject(&self, event: AppEvent) {
        let _ = self.app_tx.send(Event::App(event));
    }
}

/// Spawn entry point called from main.rs
pub async fn run(move_rx: MoveReceiver, app_tx: mpsc::UnboundedSender<Event>) {
    Coordinator::new(move_rx, app_tx).run().await;
}
```

---

### [ ] 5. `event.rs` — Extend AppEvent

**Changes:** Remove the template's `Increment`/`Decrement`. Add chess-specific
events. Expose `EventHandler::sender()` so Coordinator can inject events.

```rust
use crate::chess::{GameSnapshot, Outcome};

#[derive(Clone, Debug)]
pub enum AppEvent {
    Quit,
    NewGame,
    // Coordinator → TUI
    StateUpdate(GameSnapshot),
    GameOver(Outcome),
    IllegalMove,
}
```

**`EventHandler` additions:**
```rust
impl EventHandler {
    /// Returns a clone of the internal sender.
    /// Used by Coordinator to inject AppEvent into the TUI loop.
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }
}
```

**`GameSnapshot` must be `Clone`** (it is — all shakmaty types implement Clone).

---

### [ ] 6. `app.rs` — Pure Router

**Purpose:** Two jobs only — render the active screen, dispatch events to it.
Zero game logic.

```rust
pub enum Screen {
    Game(GameScreen),
    // Future: MainMenu, NetworkLobby, GameOver
}

pub struct App {
    running: bool,
    events: EventHandler,
    move_tx: MoveSender,
    screen: Screen,
}

impl App {
    pub fn new(events: EventHandler, move_tx: MoveSender) -> Self {
        Self {
            running: true,
            events,
            move_tx,
            screen: Screen::Game(GameScreen::new()),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            // Job 1: Render
            terminal.draw(|f| ui::render(&self.screen, f))?;

            // Job 2: Route events
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(e) => self.handle_crossterm(e),
                Event::App(e) => self.route(e),
            }
        }
        Ok(())
    }

    fn handle_crossterm(&mut self, event: crossterm::event::Event) {
        if let crossterm::event::Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return; }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    // Only quit if not mid-selection
                    if !self.screen.wants_esc() {
                        let _ = self.events.send(AppEvent::Quit);
                    } else {
                        self.screen.handle_esc();
                    }
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit);
                }
                KeyCode::Char('n') => self.events.send(AppEvent::NewGame),
                _ => {
                    // Delegate all other keys to screen; screen returns Option<Move>
                    if let Some(mv) = self.screen.handle_key(key) {
                        let _ = self.move_tx.try_send(mv);
                    }
                }
            }
        }
    }

    fn route(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit                 => self.running = false,
            AppEvent::NewGame              => self.reset(),
            AppEvent::StateUpdate(snap)    => self.screen.apply_snapshot(snap),
            AppEvent::GameOver(outcome)    => self.screen.set_game_over(outcome),
            AppEvent::IllegalMove          => self.screen.flash_illegal(),
        }
    }

    fn reset(&mut self) {
        // Re-wire: drop old move_tx, create new channel, spawn new coordinator
        // (Implementation detail: may need to restructure channels slightly)
        self.screen = Screen::Game(GameScreen::new());
    }
}
```

**`GameScreen` — owns UI input state:**
```rust
pub struct GameScreen {
    snapshot: Option<GameSnapshot>,
    cursor: Square,                   // current board cursor position
    selected: Option<Square>,         // piece the user has picked up
    legal_from_selected: MoveList,    // moves available from `selected`
    promotion_pending: Option<(Square, Square)>,  // (from, to) awaiting role
    game_over: Option<Outcome>,
    illegal_flash: u8,                // countdown ticks for red flash
}

impl GameScreen {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Move>;
    // Arrow keys → move cursor
    // Enter → select / confirm move / confirm promotion
    // u     → send UndoMove (requires coordinator support)
    // Returns Some(Move) only when a complete move is confirmed

    pub fn apply_snapshot(&mut self, snap: GameSnapshot);
    pub fn set_game_over(&mut self, outcome: Outcome);
    pub fn flash_illegal(&mut self);
    pub fn wants_esc(&self) -> bool;  // true if promotion dialog is open
    pub fn handle_esc(&mut self);     // cancel promotion, deselect piece
}
```

---

### [ ] 7. `main.rs` — Channel Wiring + Spawn

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Channel: TUI → Coordinator (bounded 1 = no double-send)
    let (move_tx, move_rx) = mpsc::channel::<Move>(1);

    // EventHandler owns the TUI event loop sender
    let events = EventHandler::new();
    let app_tx = events.sender();    // clone for Coordinator

    // Spawn game coordinator as background task
    tokio::spawn(coordinator::run(move_rx, app_tx));

    let terminal = ratatui::init();
    let result = App::new(events, move_tx).run(terminal).await;
    ratatui::restore();
    result
}
```

---

### [ ] 8. `ui/mod.rs` — Screen Dispatch

```rust
pub fn render(screen: &Screen, frame: &mut Frame) {
    match screen {
        Screen::Game(s) => board::render(s, frame),
    }
}
```

---

### [ ] 9. `ui/board.rs` — Board Widget

**Layout:**
```
┌─────────────────────────────────────────────┐
│  8 ♜  ♞  ♝  ♛  ♚  ♝  ♞  ♜                 │
│  7 ♟  ♟  ♟  ♟  ♟  ♟  ♟  ♟                 │
│  6  ·  ·  ·  ·  ·  ·  ·  ·                 │  ← · = legal move hint
│  5  ·  ·  ·  ·  ·  ·  ·  ·                 │
│  4  ·  ·  ·  ·  ♙  ·  ·  ·                 │  ← highlighted legal dest
│  3  ·  ·  ·  ·  ·  ·  ·  ·                 │
│  2 ♙  ♙  ♙  ♙  ·  ♙  ♙  ♙                 │
│  1 ♖  ♘  ♗  ♕  ♔  ♗  ♘  ♖                 │
│     a  b  c  d  e  f  g  h                  │
└─────────────────────────────────────────────┘
```

**Unicode piece map:**
```rust
fn piece_char(piece: Piece) -> &'static str {
    match (piece.color, piece.role) {
        (Color::White, Role::King)   => "♔",
        (Color::White, Role::Queen)  => "♕",
        (Color::White, Role::Rook)   => "♖",
        (Color::White, Role::Bishop) => "♗",
        (Color::White, Role::Knight) => "♘",
        (Color::White, Role::Pawn)   => "♙",
        (Color::Black, Role::King)   => "♚",
        (Color::Black, Role::Queen)  => "♛",
        (Color::Black, Role::Rook)   => "♜",
        (Color::Black, Role::Bishop) => "♝",
        (Color::Black, Role::Knight) => "♞",
        (Color::Black, Role::Pawn)   => "♟",
    }
}
```

**Color scheme:**
| State              | Background       | Foreground |
|--------------------|------------------|------------|
| Light square       | `Color::Rgb(240, 217, 181)` | piece color |
| Dark square        | `Color::Rgb(181, 136, 99)`  | piece color |
| Cursor             | `Color::Rgb(100, 111, 64)`  | bright      |
| Selected piece     | `Color::Rgb(130, 151, 105)` | bright      |
| Legal move hint    | dot `·` in center, same bg  | dimmed      |
| Last move from/to  | `Color::Rgb(205, 210, 106)` | —           |
| Check (king sq)    | `Color::Red`                | bright      |

**Render approach:** Each square is rendered as a 2-wide, 1-tall cell
(`"♟ "` or `"· "` or `"  "`). Rank labels left, file labels bottom.
Use `ratatui::buffer::Buffer::set_string()` for direct buffer writes,
or compose as a `Canvas` / custom `Widget`.

---

### [ ] 10. `ui/info.rs` — Info Panel

**Layout (right of board or below):**
```
┌─────────────────────┐
│  ♟  Black's turn    │
├─────────────────────┤
│  Captured by White  │
│  ♟ ♟ ♞             │
│  Captured by Black  │
│  ♙ ♗               │
├─────────────────────┤
│  Move history       │
│  1. e4  e5          │
│  2. Nf3 Nc6         │
│  ...                │
├─────────────────────┤
│  [n] New game       │
│  [u] Undo           │
│  [q] Quit           │
└─────────────────────┘
```

**Data source:** `GameSnapshot` fields: `turn`, `captured_white`,
`captured_black`, `move_history`.

**Move history format:** Convert `shakmaty::Move` to standard algebraic
notation. shakmaty's `Move` implements `Display` as long algebraic (e.g.,
`e2e4`). Use that initially; SAN can be added later.

---

### [ ] 11. `ui/promotion.rs` — Promotion Overlay

Rendered as a centered popup over the board when `GameScreen::promotion_pending`
is `Some`.

```
┌──────────────────────┐
│   Promote pawn to:   │
│                      │
│   Q   R   B   N      │
│   ♕   ♖   ♗   ♘      │
│       ^              │
│  (arrow keys, Enter) │
└──────────────────────┘
```

**State:** `GameScreen` tracks `promotion_cursor: usize` (0=Q,1=R,2=B,3=N).
Arrow left/right moves cursor. Enter confirms → constructs `Move::Normal {
promotion: Some(role), ... }` → sends via `move_tx`.

**`Esc`** cancels promotion, clears `promotion_pending`, deselects piece.

---

## Controls Reference

| Key         | Context         | Action                              |
|-------------|-----------------|-------------------------------------|
| Arrow keys  | Board           | Move cursor                         |
| `Enter`     | Board           | Select piece / confirm move         |
| `Enter`     | Promotion popup | Confirm promotion role              |
| `Esc`       | Promotion popup | Cancel, deselect piece              |
| `u`         | Board           | Undo last move (both plies)         |
| `n`         | Anywhere        | New game                            |
| `q`         | Anywhere        | Quit                                |
| `Ctrl-C`    | Anywhere        | Quit                                |

---

## Future Extension Points

### Network Play (`NetworkPlayer`)

1. Add `Player::Network(NetworkPlayer)` variant to the enum.
2. `NetworkPlayer` holds a tokio connection handle.
3. Coordinator `tokio::select!`s between `move_rx` (local TUI) and the network
   stream. For the local player's turn: use `move_rx`. For the remote player's
   turn: await the network.
4. No changes to `chess.rs`, `ui/`, or `app.rs`.

### Game Server

1. Extract `coordinator.rs` logic into a library crate.
2. Server wraps `Coordinator` over WebSocket/gRPC instead of mpsc channels.
3. `chess.rs` `Game` struct is already pure — fully portable.

### Engine Player

1. Add `Player::Engine(EnginePlayer)`.
2. `EnginePlayer` runs stockfish/berserk via process stdio or UCI protocol.
3. On the engine's turn, Coordinator spawns a task that talks UCI and sends
   the response move back via a local `mpsc`.

---

## Dependencies

```toml
[dependencies]
shakmaty    = "0.30"
crossterm   = { version = "0.28.1", features = ["event-stream"] }
futures     = "0.3.31"
ratatui     = "0.30.0"
tokio       = { version = "1.40.0", features = ["full"] }
color-eyre  = "0.6.3"
```

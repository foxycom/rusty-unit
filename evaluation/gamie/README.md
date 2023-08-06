# gamie
A Rust library provides abstractions for several classic tiny games.

[![Version](https://img.shields.io/crates/v/gamie.svg?style=flat)](https://crates.io/crates/gamie)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/gamie)
[![License](https://img.shields.io/crates/l/gamie.svg?style=flat)](https://github.com/EAimTY/gamie/blob/master/LICENSE)

gamie provides simple yet adequate abstractions for several classic tiny games.

gamie only came with a few dependencies, it can be easily integrated into your projects.

## Usage
To use gamie, you should enable modules in `Cargo.toml`. For example `tictactoe`:

```toml
[dependencies]
gamie = { version = "*", features = ["tictactoe"] }
```

Now you can use the `TicTacToe` game abstraction:

```rust
use gamie::tictactoe::{TicTacToe, Player as TicTacToePlayer, GameState as TicTacToeGameState};

let mut game = TicTacToe::new().unwrap();
game.place(TicTacToePlayer::Player0, 1, 1).unwrap();
game.place(TicTacToePlayer::Player1, 0, 0).unwrap();
game.place(TicTacToePlayer::Player0, 0, 2).unwrap();
game.place(TicTacToePlayer::Player1, 2, 0).unwrap();
game.place(TicTacToePlayer::Player0, 1, 0).unwrap();
game.place(TicTacToePlayer::Player1, 1, 2).unwrap();
game.place(TicTacToePlayer::Player0, 2, 1).unwrap();
game.place(TicTacToePlayer::Player1, 0, 1).unwrap();
game.place(TicTacToePlayer::Player0, 2, 2).unwrap();
assert!(game.is_ended());
assert_eq!(game.status(), &TicTacToeGameState::Tie);
```

Check the [docs](https://docs.rs/gamie) for further information.

## Modules
Currently, the following modules are available:

- [connect_four](https://docs.rs/gamie/*/gamie/connect_four)
- [gomoku](https://docs.rs/gamie/*/gamie/gomoku)
- [minesweeper](https://docs.rs/gamie/*/gamie/minesweeper)
- [reversi](https://docs.rs/gamie/*/gamie/reversi)
- [tictactoe](https://docs.rs/gamie/*/gamie/tictactoe)

## Serialize / Deserialize
Bring in the `serde` feature to enable serialization and deserialization for structs

```toml
[dependencies]
gamie = { version = "*", features = ["serde", "tictactoe"] }
```

## no_std
This crate runs flawlessly on bare metal.
To remove the Rust standard library dependency, opt out the `std` feature by disabling `default-features` in `Cargo.toml`:

```toml
[dependencies]
gamie = { version = "*", features = ["tictactoe"], default-features = false }
```

## License
GNU General Public License v3.0

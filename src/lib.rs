//! A full Pong game, implemented with the [Amethyst](https://amethyst.rs/) game
//! engine.
//!
//! Intended as an example of the architecture of complete game. This Pong
//! demonstrates:
//!
//! - `State` transitions.
//! - Entity hiding / clearing between `State`s.
//! - Game and `System` pausing via `State`-specific `Dispatcher`s.
//! - UI interaction with buttons.

pub mod audio;
pub mod core;
pub mod states;
pub mod systems;

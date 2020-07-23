use crate::core;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::ui::{Anchor, FontHandle};

/// The final `State` before the game exits.
pub struct GameOver {
    pub font: FontHandle,
}

impl SimpleState for GameOver {
    fn on_start(&mut self, data: StateData<GameData>) {
        core::generic_message(
            data.world,
            self.font.clone(),
            Anchor::Middle,
            "Game Over",
            None,
        );
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::KeyPressed { .. }) => Trans::Quit,
            _ => Trans::None,
        }
    }
}

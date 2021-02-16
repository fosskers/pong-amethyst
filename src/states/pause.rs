use crate::core;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::ui::{Anchor, FontHandle};

/// The "paused" `State`.
pub struct Pause {
    text: Option<Entity>,
    font: FontHandle,
}

impl Pause {
    pub fn new(font: FontHandle) -> Pause {
        Pause { text: None, font }
    }
}

impl SimpleState for Pause {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let entity = initialize_pause_message(world, self.font.clone());
        self.text.replace(entity);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => Trans::Pop,
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        self.text.take().iter_mut().for_each(|entity| {
            let _ = data.world.delete_entity(*entity);
        });
    }
}

fn initialize_pause_message(world: &mut World, font: FontHandle) -> Entity {
    core::generic_message(world, font, Anchor::Middle, "Paused", None)
}

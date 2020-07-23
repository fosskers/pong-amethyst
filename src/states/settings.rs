use crate::audio;
use crate::core;
use crate::states::playing::Pong;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::SpriteRender;
use amethyst::ui::{Anchor, FontHandle, UiButtonBuilder, UiEvent, UiEventType, UiImage};

pub struct Settings {
    font: FontHandle, // TODO Put this in a global resource instead?
    button: Option<Entity>,
    entities: Vec<Entity>,
}

impl Settings {
    pub fn new(font: FontHandle) -> Self {
        Settings {
            font,
            button: None,
            entities: vec![],
        }
    }
}

impl SimpleState for Settings {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        let button_sheet = core::load_sprite_sheet(world, "button");
        let unpressed_button = SpriteRender {
            sprite_sheet: button_sheet.clone(),
            sprite_number: 0,
        };
        let pressed_button = SpriteRender {
            sprite_sheet: button_sheet,
            sprite_number: 1,
        };

        // Music Button.
        let (_, button) = UiButtonBuilder::<(), u32>::new("")
            .with_anchor(Anchor::Middle)
            .with_image(UiImage::Sprite(unpressed_button))
            .with_press_image(UiImage::Sprite(pressed_button))
            .build_from_world(&world);
        self.button.replace(button.image_entity);

        // Header text.
        let header = core::generic_message(
            world,
            self.font.clone(),
            Anchor::TopMiddle,
            "Settings",
            Some(50.0),
        );

        // Usage instructions.
        let instructions = core::generic_message(
            world,
            self.font.clone(),
            Anchor::BottomMiddle,
            "Esc to Pause, Q to Quit",
            Some(25.0),
        );
        self.entities = vec![
            header,
            instructions,
            button.text_entity,
            button.image_entity,
        ];
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let _ = data.world.delete_entities(&self.entities);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::KeyPressed { .. }) => {
                Trans::Replace(Box::new(Pong::new(self.font.clone())))
            }
            StateEvent::Ui(UiEvent {
                target,
                event_type: UiEventType::Click,
            }) if Some(target) == self.button => {
                println!("[HANDLE_EVENT] You clicked the button!");
                audio::toggle_bgm(data.world);
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

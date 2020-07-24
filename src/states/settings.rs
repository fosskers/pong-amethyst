use crate::audio;
use crate::core;
use crate::states::playing::Pong;
use amethyst::assets::Handle;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{SpriteRender, Texture};
use amethyst::ui::{Anchor, FontHandle, UiButton, UiButtonBuilder, UiEvent, UiEventType, UiImage};

struct Button {
    ui_button: UiButton,
    unpressed: SpriteRender,
    pressed: SpriteRender,
    is_pressed: bool,
}

pub struct Settings {
    font: FontHandle, // TODO Put this in a global resource instead?
    button: Option<Button>,
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
        let (_, ui_button) = UiButtonBuilder::<(), u32>::new("")
            .with_anchor(Anchor::Middle)
            // .with_image(UiImage::Sprite(unpressed_button))
            .with_image(UiImage::Sprite(pressed_button.clone()))
            .build_from_world(&world);
        let button = Button {
            ui_button: ui_button.clone(),
            unpressed: unpressed_button,
            pressed: pressed_button,
            is_pressed: true,
        };
        self.button.replace(button);

        // TODO
        // UiButtonActionRetrigger?
        // UiButtonAction?
        //
        // UiButtonActionType has a `SetImage(UiImage)` variant.
        //
        // Or do I just `get_texture_mut`?

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
            ui_button.text_entity,
            ui_button.image_entity,
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
            }) if self
                .button
                .as_ref()
                .map(|b| b.ui_button.image_entity == target)
                .unwrap_or(false) =>
            {
                // if let Some(button) = &self.button {
                //     let mut storage = data.world.write_storage::<Handle<Texture>>();
                //     let mut texture = button.ui_button.get_texture_mut(&mut storage);
                //     *texture = button.unpressed;
                // }
                println!("[HANDLE_EVENT] You clicked the button!");
                audio::toggle_bgm(data.world);
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

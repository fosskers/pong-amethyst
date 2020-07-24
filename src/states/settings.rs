use crate::audio;
use crate::core;
use crate::states::playing::Pong;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::SpriteRender;
use amethyst::ui::*;

/// A UI button that can be toggled, maintaining its up/down sprite until the
/// next time it is clicked.
struct Button {
    ui_button: UiButton,
    activation: UiButtonActionRetrigger,
    deactivation: UiButtonActionRetrigger,
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
        let button = music_button(world);

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

        // Reset state fields.
        self.entities = vec![
            header,
            instructions,
            button.ui_button.text_entity,
            button.ui_button.image_entity,
        ];
        self.button.replace(button);
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
                if let Some(button) = self.button.as_mut() {
                    toggle_button(data.world, button);
                }
                println!("[HANDLE_EVENT] You clicked the button!");
                audio::toggle_bgm(data.world);
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

/// When a button is pressed, its preinstalled `UiButtonActionRetrigger` will
/// run. Then, here, that retrigger is swapped so that the next time the button
/// is pressed, a different effect will occur.
fn toggle_button(world: &mut World, button: &mut Button) {
    button.is_pressed = !button.is_pressed;
    let mut storage = world.write_storage::<UiButtonActionRetrigger>();
    storage.remove(button.ui_button.image_entity);
    if button.is_pressed {
        let _ = storage.insert(button.ui_button.image_entity, button.deactivation.clone());
    } else {
        let _ = storage.insert(button.ui_button.image_entity, button.activation.clone());
    }
}

fn music_button(world: &mut World) -> Button {
    let button_sheet = core::load_sprite_sheet(world, "button");
    let unpressed_button = SpriteRender {
        sprite_sheet: button_sheet.clone(),
        sprite_number: 0,
    };
    let pressed_button = SpriteRender {
        sprite_sheet: button_sheet,
        sprite_number: 1,
    };

    let (_, ui_button) = UiButtonBuilder::<(), u32>::new("")
        .with_anchor(Anchor::Middle)
        .with_image(UiImage::Sprite(pressed_button))
        .build_from_world(&world);

    // Register button reactions.
    let mut storage = world.write_storage::<UiButtonActionRetrigger>();
    let deactivation = retrigger(
        ui_button.image_entity,
        UiButtonActionType::SetImage(UiImage::Sprite(unpressed_button.clone())),
    );
    let activation = retrigger(
        ui_button.image_entity,
        UiButtonActionType::UnsetTexture(UiImage::Sprite(unpressed_button)),
    );
    let _ = storage.insert(ui_button.image_entity, deactivation.clone());

    Button {
        ui_button: ui_button.clone(),
        activation,
        deactivation,
        is_pressed: true,
    }
}

fn retrigger(entity: Entity, event: UiButtonActionType) -> UiButtonActionRetrigger {
    UiButtonActionRetrigger {
        on_click_stop: vec![UiButtonAction {
            target: entity,
            event_type: event,
        }],
        ..Default::default()
    }
}

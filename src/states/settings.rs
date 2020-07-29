use crate::audio;
use crate::core;
use crate::states::playing::Pong;
use amethyst::core::Transform;
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
    label: Entity,
    parent: Entity,
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
        let button = music_button(world, self.font.clone());

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
            button.label,
            button.parent,
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
                event_type: UiEventType::ClickStop,
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

fn music_button(world: &mut World, font: FontHandle) -> Button {
    // A parent entity to align the Button and Text relative to.
    let parent = {
        let transform = UiTransform::new(
            "parent".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        world.create_entity().with(transform).build()
    };

    let button_sheet = core::load_sprite_sheet(world, "button");
    let unpressed_button = SpriteRender {
        sprite_sheet: button_sheet.clone(),
        sprite_number: 0,
    };
    let pressed_button = SpriteRender {
        sprite_sheet: button_sheet,
        sprite_number: 1,
    };

    // I suppose you just have to know how big the image is? It doesn't seem
    // like you can otherwise query the size of the image from `SpriteRender`,
    // etc.
    let (_, ui_button) = UiButtonBuilder::<(), u32>::new("")
        .with_size(36.0 * 3.0, 25.0 * 3.0)
        // .with_stretch(Stretch::XY {
        //     x_margin: 200.0,
        //     y_margin: 200.0,
        //     keep_aspect_ratio: true,
        // })
        .with_anchor(Anchor::Middle)
        .with_image(UiImage::Sprite(pressed_button))
        .with_parent(parent)
        .build_from_world(&world);

    // Register button reactions.
    let (activation, deactivation) = {
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
        (activation, deactivation)
    };

    // let label = core::generic_message(world, font, Anchor::Middle, "Music", None);
    let label = {
        let transform = UiTransform::new(
            "Music".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            -175.0,
            0.0,
            0.0,
            50.0 * 5.0,
            50.0,
        );

        world
            .create_entity()
            .with(transform)
            .with(UiText::new(
                font,
                "Music".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                50.0,
                LineMode::Single,
                Anchor::Middle,
            ))
            .build()
    };

    Button {
        ui_button: ui_button.clone(),
        activation,
        deactivation,
        is_pressed: true,
        label,
        parent,
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

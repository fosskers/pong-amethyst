use crate::audio;
use crate::core::{self, ButtonPair, Pressed, SizedSprite};
use crate::states::playing::Pong;
use amethyst::assets::Handle;
use amethyst::core::Parent;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{SpriteRender, SpriteSheet};
use amethyst::ui::*;

const BUTTON_SCALING: f32 = 3.0;

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
    music_button: Option<Button>,
    control_buttons: Option<ButtonPair>,
    entities: Vec<Entity>,
}

impl Settings {
    pub fn new(font: FontHandle) -> Self {
        Settings {
            font,
            music_button: None,
            control_buttons: None,
            entities: vec![],
        }
    }
}

impl SimpleState for Settings {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let button_sheet = all_buttons(world);
        let music_button = music_button(world, button_sheet.clone(), self.font.clone());
        let controls = control_buttons(world, button_sheet);

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
            music_button.ui_button.text_entity,
            music_button.ui_button.image_entity,
            music_button.label,
            music_button.parent,
            controls.left_button.text_entity,
            controls.left_button.image_entity,
            controls.right_button.text_entity,
            controls.right_button.image_entity,
            controls.parent,
        ];
        self.music_button.replace(music_button);
        self.control_buttons.replace(controls);
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
            }) => {
                // Was the music button pressed?
                if let Some(button) = self.music_button.as_mut() {
                    if button.ui_button.image_entity == target {
                        println!("[HANDLE_EVENT] You clicked the music button!");
                        audio::toggle_bgm(data.world);
                        toggle_button(data.world, button);
                    }
                }

                // Were the control buttons pressed?
                if let Some(button) = self.control_buttons.as_mut() {
                    match button.pressed_side {
                        Pressed::Right if button.left_button.image_entity == target => {
                            println!("[HANDLE_EVENT] You clicked the WS button!");
                            button.pressed_side = Pressed::Left;
                        }
                        Pressed::Left if button.right_button.image_entity == target => {
                            println!("[HANDLE_EVENT] You clicked the WR button!");
                            button.pressed_side = Pressed::Right;
                        }
                        _ => {}
                    }
                }

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

fn all_buttons(world: &mut World) -> Handle<SpriteSheet> {
    core::load_sprite_sheet(world, "button")
}

fn control_buttons(world: &mut World, button_sheet: Handle<SpriteSheet>) -> ButtonPair {
    let left_up = sized_button(button_sheet.clone(), 2);
    let left_down = sized_button(button_sheet.clone(), 3);
    let right_up = sized_button(button_sheet.clone(), 4);
    let right_down = sized_button(button_sheet, 5);

    let parent = {
        let transform = UiTransform::new(
            "controls_parent".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            0.0,
            -100.0,
            0.0,
            200.0,
            100.0,
        );
        world.create_entity().with(transform).build()
    };

    let button_parent = {
        let transform = UiTransform::new(
            "controls_parent".to_string(),
            Anchor::MiddleRight,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            10.0,
            10.0,
        );
        world
            .create_entity()
            .with(transform)
            .with(Parent { entity: parent })
            .build()
    };

    ButtonPair::new(
        world,
        left_up,
        left_down,
        right_up,
        right_down,
        button_parent,
    )
}

fn sized_button(sprite_sheet: Handle<SpriteSheet>, sprite_number: usize) -> SizedSprite {
    SizedSprite {
        sprite: SpriteRender {
            sprite_sheet,
            sprite_number,
        },
        width: 36.0 * BUTTON_SCALING,
        height: 25.0 * BUTTON_SCALING,
    }
}

fn music_button(world: &mut World, button_sheet: Handle<SpriteSheet>, font: FontHandle) -> Button {
    // A parent entity to align the Button and Text relative to.
    let parent = {
        let transform = UiTransform::new(
            "parent".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            200.0,
            100.0,
        );
        world.create_entity().with(transform).build()
    };

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
        .with_size(36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::MiddleRight)
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
            Anchor::MiddleLeft,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            50.0 * 5.0,
            50.0,
        );

        world
            .create_entity()
            .with(transform)
            .with(Parent::new(parent))
            .with(UiText::new(
                font,
                "Music".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                40.0,
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

use crate::audio;
use crate::core::{self, ButtonPair, Pressed, SizedSprite, Sprite, BUTTON_SCALING};
use crate::states::playing::Pong;
use amethyst::assets::Handle;
use amethyst::core::Parent;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::SpriteSheet;
use amethyst::ui::{UiButtonActionType::*, *};

/// A UI button that can be toggled, maintaining its up/down sprite until the
/// next time it is clicked.
struct Button {
    button: UiButton,
    activate: UiButtonActionRetrigger,
    deactivate: UiButtonActionRetrigger,
    is_pressed: bool,
    label: Entity,
    parent: Entity,
}

pub struct Settings {
    font: FontHandle,
    sheet: Handle<SpriteSheet>,
    music: Option<Button>,
    control: Option<ButtonPair>,
    start: Option<UiButton>,
    entities: Vec<Entity>,
}

impl Settings {
    pub fn new(font: FontHandle, sheet: Handle<SpriteSheet>) -> Self {
        Settings {
            font,
            sheet,
            music: None,
            control: None,
            start: None,
            entities: vec![],
        }
    }
}

impl SimpleState for Settings {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let music = music_button(world, &self.sheet, &self.font);
        let (controls, c_label) = control_buttons(world, &self.sheet, &self.font);
        let start = start_button(world, &self.sheet);

        // Header text.
        let header = core::generic_message(
            world,
            self.font.clone(),
            Anchor::TopMiddle,
            "Settings",
            Some(50.0),
        );

        // Usage instructions.
        let usage = core::generic_message(
            world,
            self.font.clone(),
            Anchor::BottomMiddle,
            "Esc to Pause, Q to Quit",
            Some(25.0),
        );

        // Reset state fields.
        self.entities = vec![
            header,
            usage,
            music.button.text_entity,
            music.button.image_entity,
            music.label,
            music.parent,
            controls.left_button.text_entity,
            controls.left_button.image_entity,
            controls.right_button.text_entity,
            controls.right_button.image_entity,
            controls.parent,
            c_label,
            start.text_entity,
            start.image_entity,
        ];
        self.music.replace(music);
        self.control.replace(controls);
        self.start.replace(start);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let _ = data.world.delete_entities(&self.entities);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        let world = data.world;
        let font = &self.font;

        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Ui(UiEvent {
                target,
                event_type: UiEventType::ClickStop,
            }) => match (
                self.music.as_mut(),
                self.control.as_mut(),
                self.start.as_ref(),
            ) {
                (Some(mb), Some(cb), Some(sb)) => click(world, target, mb, cb, sb, font),
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

/// Handle all click events on the screen.
fn click(
    world: &mut World,
    target: Entity,
    mb: &mut Button,
    cb: &mut ButtonPair,
    sb: &UiButton,
    font: &FontHandle,
) -> SimpleTrans {
    if mb.button.image_entity == target {
        println!("[HANDLE_EVENT] You clicked the music button!");
        audio::toggle_bgm(world);
        toggle_button(world, mb);
        Trans::None
    } else if sb.image_entity == target {
        Trans::Replace(Box::new(Pong::new(font.clone())))
    } else {
        match cb.pressed_side {
            Pressed::Right if cb.left_button.image_entity == target => {
                println!("[HANDLE_EVENT] You clicked the WS button!");
                cb.pressed_side = Pressed::Left;
            }
            Pressed::Left if cb.right_button.image_entity == target => {
                println!("[HANDLE_EVENT] You clicked the WR button!");
                cb.pressed_side = Pressed::Right;
            }
            _ => {}
        }
        Trans::None
    }
}

/// When a button is pressed, its preinstalled `UiButtonActionRetrigger` will
/// run. Then, here, that retrigger is swapped so that the next time the button
/// is pressed, a different effect will occur.
fn toggle_button(world: &mut World, button: &mut Button) {
    button.is_pressed = !button.is_pressed;
    let mut storage = world.write_storage::<UiButtonActionRetrigger>();
    storage.remove(button.button.image_entity);
    if button.is_pressed {
        let _ = storage.insert(button.button.image_entity, button.deactivate.clone());
    } else {
        let _ = storage.insert(button.button.image_entity, button.activate.clone());
    }
}

fn start_button(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) -> UiButton {
    let up = Sprite::new(sprite_sheet.clone(), 6);
    let down = Sprite::new(sprite_sheet.clone(), 7);

    UiButtonBuilder::<(), u32>::new("")
        .with_size(72.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::Middle)
        .with_position(0.0, -100.0)
        .with_image(UiImage::Sprite(up.0))
        .with_press_image(UiImage::Sprite(down.0))
        .build_from_world(&world)
        .1
}

fn control_buttons(
    world: &mut World,
    button_sheet: &Handle<SpriteSheet>,
    font: &FontHandle,
) -> (ButtonPair, Entity) {
    let left_up = sized_button(button_sheet.clone(), 2);
    let left_down = sized_button(button_sheet.clone(), 3);
    let right_up = sized_button(button_sheet.clone(), 4);
    let right_down = sized_button(button_sheet.clone(), 5);

    let parent = {
        let transform = UiTransform::new(
            "controls_parent".to_string(),
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
            .with(Parent::new(parent))
            .build()
    };

    let label = {
        let transform = UiTransform::new(
            "P1 Controls".to_string(),
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
                font.clone(),
                "P1 Controls".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                25.0,
                LineMode::Single,
                Anchor::MiddleLeft,
            ))
            .build()
    };

    let button_pair = ButtonPair::new(
        world,
        left_up,
        left_down,
        right_up,
        right_down,
        button_parent,
    );

    (button_pair, label)
}

fn sized_button(sprite_sheet: Handle<SpriteSheet>, sprite_number: usize) -> SizedSprite {
    SizedSprite {
        sprite: Sprite::new(sprite_sheet, sprite_number),
        width: 36.0 * BUTTON_SCALING,
        height: 25.0 * BUTTON_SCALING,
    }
}

fn music_button(
    world: &mut World,
    button_sheet: &Handle<SpriteSheet>,
    font: &FontHandle,
) -> Button {
    // A parent entity to align the Button and Text relative to.
    let parent = {
        let transform = UiTransform::new(
            "parent".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            0.0,
            100.0,
            0.0,
            200.0,
            100.0,
        );
        world.create_entity().with(transform).build()
    };

    let unpressed = Sprite::new(button_sheet.clone(), 0);
    let pressed = Sprite::new(button_sheet.clone(), 1);

    // I suppose you just have to know how big the image is? It doesn't seem
    // like you can otherwise query the size of the image from `SpriteRender`,
    // etc.
    let (_, ui_button) = UiButtonBuilder::<(), u32>::new("")
        .with_size(36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::MiddleRight)
        .with_image(UiImage::Sprite(pressed.0))
        .with_parent(parent)
        .build_from_world(&world);

    // Register button reactions.
    let (activate, deactivate) = {
        let mut storage = world.write_storage::<UiButtonActionRetrigger>();
        let deactivation = retrigger(
            ui_button.image_entity,
            SetImage(UiImage::Sprite(unpressed.0.clone())),
        );
        let activation = retrigger(
            ui_button.image_entity,
            UnsetTexture(UiImage::Sprite(unpressed.0)),
        );
        let _ = storage.insert(ui_button.image_entity, deactivation.clone());
        (activation, deactivation)
    };

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
                font.clone(),
                "Music".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                25.0,
                LineMode::Single,
                Anchor::MiddleLeft,
            ))
            .build()
    };

    Button {
        button: ui_button.clone(),
        activate,
        deactivate,
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

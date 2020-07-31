//! Core types and functions used across other modules.

use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use amethyst::ecs::Entity;
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};
use amethyst::ui::*;

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 65.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

pub const BUTTON_SCALING: f32 = 3.0;

/// A `SpriteRender` that knows its pixel dimensions.
pub struct SizedSprite {
    pub sprite: SpriteRender,
    pub width: f32,
    pub height: f32,
}

/// A pair of buttons. Clicking one will toggle some change in both.
pub struct ButtonPair {
    pub left_button: UiButton,
    pub right_button: UiButton,
    pub pressed_side: Pressed,
    pub parent: Entity,
}

impl ButtonPair {
    /// Generates `UiButton`s into the `World`, and registers custom
    /// `UiButtonActionRetrigger`s for them as well for the toggling animation.
    pub fn new(
        world: &mut World,
        left_up: SizedSprite,
        left_down: SizedSprite,
        right_up: SizedSprite,
        right_down: SizedSprite,
        parent: Entity,
    ) -> ButtonPair {
        let (_, left_button) = UiButtonBuilder::<(), u32>::new("")
            .with_size(left_down.width, left_down.height)
            .with_anchor(Anchor::Middle)
            .with_image(UiImage::Sprite(left_down.sprite.clone()))
            .with_parent(parent)
            .build_from_world(&world);

        let (_, right_button) = UiButtonBuilder::<(), u32>::new("")
            .with_size(right_down.width, right_down.height)
            .with_position(right_down.width, 0.0)
            .with_anchor(Anchor::Middle)
            .with_image(UiImage::Sprite(right_up.sprite.clone()))
            .with_parent(parent)
            .build_from_world(&world);

        // An event triggered by one button can cause an Action to occur on
        // another. We use this to change the image of the second button when
        // the first is clicked (and vice versa).
        let click_left = UiButtonActionRetrigger {
            on_click_start: vec![
                UiButtonAction {
                    target: left_button.image_entity,
                    event_type: UiButtonActionType::SetImage(UiImage::Sprite(left_down.sprite)),
                },
                UiButtonAction {
                    target: right_button.image_entity,
                    event_type: UiButtonActionType::SetImage(UiImage::Sprite(right_up.sprite)),
                },
            ],
            ..Default::default()
        };

        let click_right = UiButtonActionRetrigger {
            on_click_start: vec![
                UiButtonAction {
                    target: left_button.image_entity,
                    event_type: UiButtonActionType::SetImage(UiImage::Sprite(left_up.sprite)),
                },
                UiButtonAction {
                    target: right_button.image_entity,
                    event_type: UiButtonActionType::SetImage(UiImage::Sprite(right_down.sprite)),
                },
            ],
            ..Default::default()
        };

        let mut storage = world.write_storage::<UiButtonActionRetrigger>();
        let _ = storage.insert(left_button.image_entity, click_left);
        let _ = storage.insert(right_button.image_entity, click_right);

        ButtonPair {
            left_button,
            right_button,
            pressed_side: Pressed::Left,
            parent,
        }
    }
}

pub enum Pressed {
    Left,
    Right,
}

/// A component for Entities whose activity can be halted.
pub struct Active {
    /// `None` implies that the entity is active.
    pub countdown: Option<f32>,
}

impl Component for Active {
    type Storage = DenseVecStorage<Active>;
}

#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: u32,
    pub score_right: u32,
}

pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}

/// The "Ready?" message before the ball begins to move.
pub struct ServeText(pub Entity);

/// The FPS counter.
pub struct FPS(pub Entity);

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Ball>;
}

pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Paddle>;
}

/// Given the name of a texture png/ron pair, read its `SpriteSheet`.
pub fn load_sprite_sheet(world: &mut World, path: &str) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    let sprite_sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();

    let texture_handle = loader.load(
        format!("texture/{}.png", path),
        ImageFormat::default(),
        (),
        &texture_storage,
    );

    loader.load(
        format!("texture/{}.ron", path),
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_storage,
    )
}

/// Conveniently create a Text `Entity`.
pub fn generic_message(
    world: &mut World,
    font: FontHandle,
    anchor: Anchor,
    msg: &str,
    size: Option<f32>,
) -> Entity {
    let m1 = msg.to_string();
    let m2 = msg.to_string();
    let text_size = size.unwrap_or(50.0);
    let transform = UiTransform::new(
        m1,
        anchor,
        anchor,
        0.0,
        0.0,
        1.0,
        text_size * msg.chars().count() as f32,
        text_size,
    );

    world
        .create_entity()
        .with(transform)
        .with(UiText::new(
            font,
            m2,
            [1.0, 1.0, 1.0, 1.0],
            text_size,
            LineMode::Single,
            Anchor::Middle,
        ))
        .build()
}

pub fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

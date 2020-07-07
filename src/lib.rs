use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};
use amethyst::ui::{Anchor, TtfFormat, UiText, UiTransform};

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: u32,
    pub score_right: u32,
}

pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}

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
    fn new(side: Side) -> Paddle {
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

#[derive(Default)]
pub struct Pong {
    ball_spawn_timer: Option<f32>,
    sprite_sheet: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let sprite_sheet_handle = load_sprite_sheet(world);

        self.ball_spawn_timer.replace(1.0);
        self.sprite_sheet.replace(sprite_sheet_handle);

        initialize_paddles(world, self.sprite_sheet.clone().unwrap());
        initialize_camera(world);
        initialize_scoreboard(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            timer -= data.world.fetch::<Time>().delta_seconds();

            if timer <= 0.0 {
                initialize_ball(data.world, self.sprite_sheet.clone().unwrap());
            } else {
                self.ball_spawn_timer.replace(timer);
            }
        }

        Trans::None
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

fn initialize_paddles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // A component to actually render the paddles.
    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .build();

    world
        .create_entity()
        .with(sprite_render)
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .build();
}

fn initialize_ball(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 1,
    };

    let ball = Ball {
        radius: BALL_RADIUS,
        velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(ball)
        .with(local_transform)
        .build();
}

fn initialize_scoreboard(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        -50.0,
        -50.0,
        1.0,
        200.0,
        50.0,
    );

    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        50.0,
        -50.0,
        1.0,
        200.0,
        50.0,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    world.insert(ScoreText { p1_score, p2_score });
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    let sprite_sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();

    let texture_handle = loader.load(
        "texture/pong_spritesheet.png",
        ImageFormat::default(),
        (),
        &texture_storage,
    );

    loader.load(
        "texture/pong_spritesheet.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_storage,
    )
}

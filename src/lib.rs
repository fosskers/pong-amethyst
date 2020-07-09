pub mod audio;
pub mod core;
pub mod systems;

use crate::core::*;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::core::ArcThreadPool;
use amethyst::ecs::{Dispatcher, DispatcherBuilder};
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};
use amethyst::ui::{Anchor, TtfFormat, UiText, UiTransform};

/// Pause the game.
pub struct Pause;

impl SimpleState for Pause {
    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => {
                println!("Unpaused.");
                Trans::Pop
            }
            _ => Trans::None,
        }
    }
}

/// The main game state.
#[derive(Default)]
pub struct Pong<'a, 'b> {
    sprite_sheet: Option<Handle<SpriteSheet>>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for Pong<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        // Initial the system dispatcher unique to the "running" game state.
        let mut builder = DispatcherBuilder::new();
        builder.add(systems::MoveBallSystem, "ball_system", &[]);
        builder.add(systems::PaddleSystem, "paddle_system", &[]);
        builder.add(
            systems::BounceSystem,
            "collision_system",
            &["paddle_system", "ball_system"],
        );
        builder.add(systems::ScoreSystem, "score_system", &["ball_system"]);

        let mut dispatcher = builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);
        self.dispatcher = Some(dispatcher);

        // Set up the sprites.
        let sprite_sheet_handle = load_sprite_sheet(world);
        self.sprite_sheet.replace(sprite_sheet_handle);

        // Create all entities.
        initialize_paddles(world, self.sprite_sheet.clone().unwrap());
        initialize_camera(world);
        initialize_scoreboard(world);
        initialize_ball(world, self.sprite_sheet.clone().unwrap());
        initialize_messages(world);
        audio::initialize_audio(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        Trans::None
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => {
                println!("Paused!");
                Trans::Push(Box::new(Pause))
            }
            _ => Trans::None,
        }
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

    let active = Active {
        countdown: Some(1.0),
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(ball)
        .with(local_transform)
        .with(active)
        .build();
}

fn initialize_messages(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let transform = UiTransform::new(
        "Service".to_string(),
        Anchor::BottomMiddle,
        Anchor::BottomMiddle,
        0.0,
        0.0,
        1.0,
        200.0,
        50.0,
    );

    let text = world
        .create_entity()
        .with(transform)
        .with(UiText::new(
            font,
            "Service!".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    world.insert(ServeText(text));
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

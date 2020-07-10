pub mod audio;
pub mod core;
pub mod systems;

use crate::core::*;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::core::ArcThreadPool;
use amethyst::ecs::{Dispatcher, DispatcherBuilder, Entity};
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};
use amethyst::ui::{Anchor, FontHandle, TtfFormat, UiText, UiTransform};

pub struct GameOver {
    font: FontHandle,
}

impl SimpleState for GameOver {
    fn on_start(&mut self, data: StateData<GameData>) {
        generic_message(data.world, self.font.clone(), Anchor::Middle, "Game Over");
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::KeyPressed { .. }) => Trans::Quit,
            _ => Trans::None,
        }
    }
}

/// The "paused" `State`.
pub struct Pause {
    text: Option<Entity>,
    font: FontHandle,
}

impl Pause {
    fn new(font: FontHandle) -> Pause {
        Pause { text: None, font }
    }
}

impl SimpleState for Pause {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let entity = initialize_pause_message(world, self.font.clone());
        self.text.replace(entity);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => {
                self.text.take().iter_mut().for_each(|entity| {
                    let _ = data.world.delete_entity(*entity);
                });
                Trans::Pop
            }
            _ => Trans::None,
        }
    }
}

/// The main game `State`.
#[derive(Default)]
pub struct Pong<'a, 'b> {
    sprite_sheet: Option<Handle<SpriteSheet>>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
    font: Option<FontHandle>,
    entities: Vec<Entity>,
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

        // Read the font.
        let font: FontHandle = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );
        self.font.replace(font);

        // Create all entities.
        let (left, right) = initialize_paddles(world, self.sprite_sheet.clone().unwrap());
        initialize_camera(world);
        initialize_scoreboard(world, self.font.clone().unwrap());
        initialize_ball(world, self.sprite_sheet.clone().unwrap());
        initialize_messages(world, self.font.clone().unwrap());
        audio::initialize_audio(world);
        let entities = vec![left, right];
        self.entities = entities;
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // This state will never be used again, so we remove all of its entities.
        let _ = data.world.delete_entities(&self.entities);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        // Special scope to make the borrowed `score_board` disappear as soon as
        // it's no longer needed. The dispatch below will invoke a system that
        // wants to borrow the `ScoreBoard` too, which causes a panic.
        {
            let score_board = data.world.read_resource::<ScoreBoard>();

            if score_board.score_left >= 10 || score_board.score_right >= 10 {
                if let Some(font) = &self.font {
                    return Trans::Replace(Box::new(GameOver { font: font.clone() }));
                }
            }
        }

        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        Trans::None
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => self
                .font
                .as_ref()
                .map(|font| Trans::Push(Box::new(Pause::new(font.clone()))))
                .unwrap_or(Trans::None),
            // TODO Remove later.
            StateEvent::Input(InputEvent::KeyTyped('z')) => self
                .font
                .as_ref()
                .map(|font| Trans::Replace(Box::new(GameOver { font: font.clone() })))
                .unwrap_or(Trans::None),
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

fn initialize_paddles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> (Entity, Entity) {
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

    let left = world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .build();

    let right = world
        .create_entity()
        .with(sprite_render)
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .build();

    (left, right)
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

fn generic_message(world: &mut World, font: FontHandle, anchor: Anchor, msg: &str) -> Entity {
    let m1 = msg.to_string();
    let m2 = msg.to_string();
    let transform = UiTransform::new(m1, anchor, anchor, 0.0, 0.0, 1.0, 300.0, 50.0);

    world
        .create_entity()
        .with(transform)
        .with(UiText::new(font, m2, [1.0, 1.0, 1.0, 1.0], 50.0))
        .build()
}

fn initialize_pause_message(world: &mut World, font: FontHandle) -> Entity {
    generic_message(world, font, Anchor::Middle, "Paused")
}

fn initialize_messages(world: &mut World, font: FontHandle) {
    let text = generic_message(world, font, Anchor::BottomMiddle, "Service!");
    world.insert(ServeText(text));
}

fn initialize_scoreboard(world: &mut World, font: FontHandle) {
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

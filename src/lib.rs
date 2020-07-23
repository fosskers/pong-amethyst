pub mod audio;
pub mod core;
pub mod systems;

use crate::core::*;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::audio::AudioSink;
use amethyst::core::transform::Transform;
use amethyst::core::ArcThreadPool;
use amethyst::ecs::{Dispatcher, DispatcherBuilder, Entity};
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
};
use amethyst::ui::{
    Anchor, FontHandle, LineMode, TtfFormat, UiButtonBuilder, UiEvent, UiEventType, UiImage,
    UiText, UiTransform,
};
use amethyst::utils::fps_counter::FpsCounter;

/// The initial landing screen.
#[derive(Default)]
pub struct Welcome {
    font: Option<FontHandle>,
    button: Option<Entity>,
    entities: Vec<Entity>,
}

impl SimpleState for Welcome {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        // Read the font.
        let font: FontHandle = world.read_resource::<Loader>().load(
            "font/arcade.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );
        self.font.replace(font);

        let button_sheet = load_sprite_sheet(world, "button");
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

        // Usage instructions.
        let instructions = generic_message(
            world,
            self.font.clone().unwrap(),
            Anchor::BottomMiddle,
            "Esc to Pause, Q to Quit",
            Some(25.0),
        );
        let logo = initialize_logo(world);
        self.entities = vec![instructions, logo, button.text_entity, button.image_entity];

        initialize_camera(world);
        audio::initialize_audio(world);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let _ = data.world.delete_entities(&self.entities);
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::KeyPressed { .. }) => self
                .font
                .as_ref()
                .map(|font| Trans::Replace(Box::new(Pong::new(font.clone()))))
                .unwrap_or(Trans::None),
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

/// The final `State` before the game exits.
pub struct GameOver {
    font: FontHandle,
}

impl SimpleState for GameOver {
    fn on_start(&mut self, data: StateData<GameData>) {
        generic_message(
            data.world,
            self.font.clone(),
            Anchor::Middle,
            "Game Over",
            None,
        );
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
        world.read_resource::<AudioSink>().pause();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => Trans::Pop,
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        data.world.read_resource::<AudioSink>().play();
        self.text.take().iter_mut().for_each(|entity| {
            let _ = data.world.delete_entity(*entity);
        });
    }
}

/// The main game `State`.
pub struct Pong<'a, 'b> {
    sprite_sheet: Option<Handle<SpriteSheet>>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
    font: FontHandle,
    entities: Vec<Entity>,
    fps: Option<Entity>,
}

impl<'a, 'b> Pong<'a, 'b> {
    pub fn new(font: FontHandle) -> Pong<'a, 'b> {
        Pong {
            sprite_sheet: None,
            dispatcher: None,
            font,
            entities: vec![],
            fps: None,
        }
    }
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
        builder.add(systems::FpsSystem, "fps_system", &[]);

        let mut dispatcher = builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);
        self.dispatcher = Some(dispatcher);

        // Set up the sprites.
        let sprite_sheet_handle = load_sprite_sheet(world, "pong_spritesheet");
        self.sprite_sheet.replace(sprite_sheet_handle);

        // Create all entities.
        let (left, right) = initialize_paddles(world, self.sprite_sheet.clone().unwrap());
        initialize_scoreboard(world, self.font.clone());
        let ball = initialize_ball(world, self.sprite_sheet.clone().unwrap());
        let ready = initialize_ready_msg(world, self.font.clone());
        let fps = initialize_fps(world, self.font.clone());
        let entities = vec![left, right, ball, ready, fps];
        self.entities = entities;
        self.fps = Some(fps);
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
                return Trans::Replace(Box::new(GameOver {
                    font: self.font.clone(),
                }));
            }
        }

        // Run all `Systems` unique to this `State`.
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        Trans::None
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "pause" => {
                Trans::Push(Box::new(Pause::new(self.font.clone())))
            }
            // TODO Remove later.
            StateEvent::Input(InputEvent::KeyTyped('z')) => Trans::Replace(Box::new(GameOver {
                font: self.font.clone(),
            })),
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

fn initialize_ball(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> Entity {
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
        .build()
}

fn generic_message(
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

fn initialize_pause_message(world: &mut World, font: FontHandle) -> Entity {
    generic_message(world, font, Anchor::Middle, "Paused", None)
}

fn initialize_ready_msg(world: &mut World, font: FontHandle) -> Entity {
    let text = generic_message(world, font, Anchor::BottomMiddle, "Ready?", None);
    world.insert(ServeText(text));
    text
}

fn initialize_fps(world: &mut World, font: FontHandle) -> Entity {
    let msg = {
        let fps = world.read_resource::<FpsCounter>();
        format!("{:.0}", fps.sampled_fps())
    };
    let text = generic_message(world, font, Anchor::TopMiddle, &msg, Some(20.0));
    world.insert(FPS(text));
    text
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
            LineMode::Single,
            Anchor::Middle,
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
            LineMode::Single,
            Anchor::Middle,
        ))
        .build();

    // TODO Why is this insert necessary?
    world.insert(ScoreText { p1_score, p2_score });
}

fn initialize_logo(world: &mut World) -> Entity {
    let sprite_render = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        let sprite_sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();

        let texture_handle: Handle<Texture> = loader.load(
            "texture/logo.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        );

        let sprite_sheet: Handle<SpriteSheet> = loader.load(
            "texture/logo.ron",
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_storage,
        );

        SpriteRender {
            sprite_sheet,
            sprite_number: 0,
        }
    };

    // TODO Fix up this position.
    let mut local_transform = Transform::default();
    // local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT - 16.0, 0.0);

    world
        .create_entity()
        .with(local_transform)
        .with(sprite_render)
        .build()
}

/// Given the name of a texture png/ron pair, read its `SpriteSheet`.
fn load_sprite_sheet(world: &mut World, path: &str) -> Handle<SpriteSheet> {
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

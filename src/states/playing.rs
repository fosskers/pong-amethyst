use crate::core::*;
use crate::states::game_over::GameOver;
use crate::states::pause::Pause;
use crate::systems;
use amethyst::assets::Handle;
use amethyst::core::transform::Transform;
use amethyst::core::ArcThreadPool;
use amethyst::ecs::{Dispatcher, DispatcherBuilder, Entity};
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::SpriteSheet;
use amethyst::ui::{Anchor, FontHandle, LineMode, UiText, UiTransform};
use amethyst::utils::fps_counter::FpsCounter;

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

fn initialize_ball(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    let sprite = Sprite::new(sprite_sheet, 1);
    let ball = Ball::new(BALL_RADIUS, [BALL_VELOCITY_X, BALL_VELOCITY_Y]);
    let active = Active::new(1.0);

    world
        .create_entity()
        .with(sprite.0)
        .with(ball)
        .with(transform)
        .with(active)
        .build()
}

fn initialize_paddles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> (Entity, Entity) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // A component to actually render the paddles.
    let sprite = Sprite::new(sprite_sheet, 0);

    let left = world
        .create_entity()
        .with(sprite.0.clone())
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .build();

    let right = world
        .create_entity()
        .with(sprite.0)
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .build();

    (left, right)
}

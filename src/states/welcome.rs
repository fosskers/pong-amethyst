use crate::audio;
use crate::core::*;
use crate::states::playing::Pong;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};
use amethyst::ui::{Anchor, FontHandle, TtfFormat, UiButtonBuilder, UiEvent, UiEventType, UiImage};

/// The initial landing screen.
#[derive(Default)]
pub struct Welcome {
    font: Option<FontHandle>, // TODO Put this in a global resource instead?
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

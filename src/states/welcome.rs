use crate::audio;
use crate::core::{self, *};
use crate::states::settings::Settings;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};
use amethyst::ui::*;

/// The initial landing screen.
#[derive(Default)]
pub struct Welcome {
    font: Option<FontHandle>, // TODO Put this in a global resource instead?
    entities: Vec<Entity>,
}

impl SimpleState for Welcome {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let (conf, start) = buttons(world);

        // Read the font.
        let font: FontHandle = world.read_resource::<Loader>().load(
            "font/arcade.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );
        self.font.replace(font);

        let logo = initialize_logo(world);
        self.entities = vec![
            logo,
            conf.text_entity,
            conf.image_entity,
            start.text_entity,
            start.image_entity,
        ];

        initialize_camera(world);
        audio::initialize_audio(world);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let _ = data.world.delete_entities(&self.entities);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::KeyPressed { .. }) => self
                .font
                .as_ref()
                .map(|font| Trans::Replace(Box::new(Settings::new(font.clone()))))
                .unwrap_or(Trans::None),
            _ => Trans::None,
        }
    }
}

fn buttons(world: &mut World) -> (UiButton, UiButton) {
    let sprite_sheet = core::load_sprite_sheet(world, "button");

    let start_up = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 6,
    };

    let start_down = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 7,
    };

    let conf_up = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 8,
    };

    let conf_down = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 9,
    };

    let (_, conf) = UiButtonBuilder::<(), u32>::new("")
        .with_size(72.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::BottomMiddle)
        .with_position(-36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_image(UiImage::Sprite(conf_up))
        .with_press_image(UiImage::Sprite(conf_down))
        .build_from_world(&world);

    let (_, start) = UiButtonBuilder::<(), u32>::new("")
        .with_size(72.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::BottomMiddle)
        .with_position(36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_image(UiImage::Sprite(start_up))
        .with_press_image(UiImage::Sprite(start_down))
        .build_from_world(&world);

    (conf, start)
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
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);
    // local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT - 16.0, 0.0);

    world
        .create_entity()
        .with(local_transform)
        .with(sprite_render)
        .build()
}

use crate::audio;
use crate::core::{self, *};
use crate::states::playing::Pong;
use crate::states::settings::Settings;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::transform::Transform;
use amethyst::ecs::Entity;
use amethyst::input::InputEvent;
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture};
use amethyst::ui::*;

/// The initial landing screen.
#[derive(Default)]
pub struct Welcome {
    font: Option<FontHandle>,
    sheet: Option<Handle<SpriteSheet>>,
    conf: Option<UiButton>,
    start: Option<UiButton>,
    entities: Vec<Entity>,
}

impl SimpleState for Welcome {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let button_sheet = core::load_sprite_sheet(world, "button");
        let (conf, start) = buttons(world, button_sheet.clone());

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
        self.conf.replace(conf);
        self.start.replace(start);
        self.sheet.replace(button_sheet);

        initialize_camera(world);
        audio::initialize_audio(world);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let _ = data.world.delete_entities(&self.entities);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Input(InputEvent::ActionPressed(a)) if a == "quit" => Trans::Quit,
            StateEvent::Ui(UiEvent {
                target,
                event_type: UiEventType::ClickStop,
            }) => match (&self.conf, &self.start, &self.font, &self.sheet) {
                (Some(conf), Some(start), Some(font), Some(sheet)) => {
                    if conf.image_entity == target {
                        Trans::Replace(Box::new(Settings::new(font.clone(), sheet.clone())))
                    } else if start.image_entity == target {
                        Trans::Replace(Box::new(Pong::new(font.clone())))
                    } else {
                        Trans::None
                    }
                }
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn buttons(world: &mut World, sprite_sheet: Handle<SpriteSheet>) -> (UiButton, UiButton) {
    let start_up = Sprite::new(sprite_sheet.clone(), 6);
    let start_down = Sprite::new(sprite_sheet.clone(), 7);
    let conf_up = Sprite::new(sprite_sheet.clone(), 8);
    let conf_down = Sprite::new(sprite_sheet.clone(), 9);

    let (_, conf) = UiButtonBuilder::<(), u32>::new("")
        .with_size(72.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::BottomMiddle)
        .with_position(-36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_image(UiImage::Sprite(conf_up.0))
        .with_press_image(UiImage::Sprite(conf_down.0))
        .build_from_world(&world);

    let (_, start) = UiButtonBuilder::<(), u32>::new("")
        .with_size(72.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_anchor(Anchor::BottomMiddle)
        .with_position(36.0 * BUTTON_SCALING, 25.0 * BUTTON_SCALING)
        .with_image(UiImage::Sprite(start_up.0))
        .with_press_image(UiImage::Sprite(start_down.0))
        .build_from_world(&world);

    (conf, start)
}

fn initialize_logo(world: &mut World) -> Entity {
    let sprite = {
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

        Sprite::new(sprite_sheet, 0)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    world.create_entity().with(transform).with(sprite.0).build()
}

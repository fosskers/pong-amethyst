mod systems;

use amethyst::core::transform::TransformBundle;
use amethyst::input::{InputBundle, StringBindings};
use amethyst::prelude::*;
use amethyst::renderer::plugins::{RenderFlat2D, RenderToWindow};
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::RenderingBundle;
use systems::move_balls::MoveBallSystem;
use systems::paddle::PaddleSystem;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let app_root = amethyst::utils::application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");
    let binding_path = app_root.join("config").join("bindings.ron");

    let rendering_bundle = RenderingBundle::<DefaultBackend>::new()
        .with_plugin(
            RenderToWindow::from_config_path(display_config_path)?.with_clear([0.0, 0.0, 0.0, 1.0]),
        )
        .with_plugin(RenderFlat2D::default());

    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(rendering_bundle)?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(PaddleSystem, "paddle_system", &["input_system"])
        .with(MoveBallSystem, "move_ball_system", &["input_system"]);

    let assets_dir = app_root.join("assets");
    let mut game = Application::new(assets_dir, pong::Pong, game_data)?;
    game.run();

    Ok(())
}

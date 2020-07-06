use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::plugins::{RenderFlat2D, RenderToWindow};
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::RenderingBundle;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let app_root = amethyst::utils::application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(TransformBundle::new())?;
    let assets_dir = app_root.join("assets");
    let mut game = Application::new(assets_dir, pong::Pong, game_data)?;
    game.run();

    Ok(())
}

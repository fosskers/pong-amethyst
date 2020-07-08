use crate::core::{Active, Ball, ServeText};
use amethyst::core::timing::Time;
use amethyst::core::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage};
use amethyst::ui::UiText;

#[derive(SystemDesc)]
pub struct MoveBallSystem;

impl<'s> System<'s> for MoveBallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        WriteStorage<'s, Active>,
        WriteStorage<'s, UiText>,
        ReadExpect<'s, ServeText>,
    );

    fn run(
        &mut self,
        (balls, mut locals, time, mut actives, mut ui_text, serve_text): Self::SystemData,
    ) {
        // Time since the last frame.
        let delta = time.delta_seconds();

        for (ball, local, active) in (&balls, &mut locals, &mut actives).join() {
            match active.countdown {
                None => {
                    local.prepend_translation_x(ball.velocity[0] * delta);
                    local.prepend_translation_y(ball.velocity[1] * delta);
                }
                Some(timer) if timer - delta < 0.0 => {
                    active.countdown.take();
                    if let Some(text) = ui_text.get_mut(serve_text.0) {
                        text.color = [1.0, 1.0, 1.0, 0.0];
                    }
                }
                Some(timer) => {
                    active.countdown.replace(timer - delta);
                }
            }
        }
    }
}

use crate::core::{Active, Ball, ServeText};
use amethyst::core::timing::Time;
use amethyst::core::{Hidden, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage};

#[derive(SystemDesc)]
pub struct MoveBallSystem;

impl<'s> System<'s> for MoveBallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        WriteStorage<'s, Active>,
        ReadExpect<'s, ServeText>,
        WriteStorage<'s, Hidden>,
    );

    fn run(
        &mut self,
        (balls, mut locals, time, mut actives, serve_text, mut hiddens): Self::SystemData,
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
                    let _ = hiddens.insert(serve_text.0, Hidden);
                }
                Some(timer) => {
                    active.countdown.replace(timer - delta);
                }
            }
        }
    }
}

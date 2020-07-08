use crate::core::{Active, Ball};
use amethyst::core::timing::Time;
use amethyst::core::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage};

#[derive(SystemDesc)]
pub struct MoveBallSystem;

impl<'s> System<'s> for MoveBallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        WriteStorage<'s, Active>,
    );

    fn run(&mut self, (balls, mut locals, time, mut actives): Self::SystemData) {
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
                }
                Some(timer) => {
                    active.countdown.replace(timer - delta);
                }
            }
        }
    }
}

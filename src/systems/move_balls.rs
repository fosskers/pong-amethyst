use amethyst::core::timing::Time;
use amethyst::core::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage};
use pong::Ball;

#[derive(SystemDesc)]
pub struct MoveBallSystem;

impl<'s> System<'s> for MoveBallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut locals, time): Self::SystemData) {
        // Time since the last frame.
        let delta = time.delta_seconds();

        for (ball, local) in (&balls, &mut locals).join() {
            local.prepend_translation_x(ball.velocity[0] * delta);
            local.prepend_translation_y(ball.velocity[1] * delta);
        }
    }
}

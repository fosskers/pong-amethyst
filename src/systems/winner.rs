use amethyst::core::Transform;
use amethyst::ecs::{Join, System, WriteStorage};
use pong::{Ball, ARENA_WIDTH};

pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (WriteStorage<'s, Ball>, WriteStorage<'s, Transform>);

    fn run(&mut self, (mut balls, mut locals): Self::SystemData) {
        for (ball, transform) in (&mut balls, &mut locals).join() {
            let ball_x = transform.translation().x;

            if ball_x <= ball.radius || ball_x >= ARENA_WIDTH - ball.radius {
                ball.velocity[0] *= -1.0;
                transform.set_translation_x(ARENA_WIDTH / 2.0);
            }
        }
    }
}

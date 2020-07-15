use crate::audio;
use crate::core::*;
use amethyst::assets::AssetStorage;
use amethyst::audio::output::Output;
use amethyst::audio::Source;
use amethyst::core::Transform;
use amethyst::ecs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
use std::ops::Deref;

pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, audio::Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;
            let output = audio_output.as_ref().map(|o| o.deref());

            // Bounce off the walls.
            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
            {
                ball.velocity[1] *= -1.0;
                audio::play_bounce_sound(&sounds, &storage, output);
            }

            // Bounce off the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                // Location of the bottom corner of the paddle.
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + paddle.width + ball.radius,
                    paddle_y + paddle.height + ball.radius,
                ) {
                    match paddle.side {
                        Side::Left if ball.velocity[0] < 0.0 => {
                            ball.velocity[0] *= -1.05;
                            audio::play_bounce_sound(&sounds, &storage, output);
                        }
                        Side::Right if ball.velocity[0] > 0.0 => {
                            ball.velocity[0] *= -1.05;
                            audio::play_bounce_sound(&sounds, &storage, output);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}

//! Atomic operations over Entities and Resources.

pub use bounce::BounceSystem;
pub use fps::FpsSystem;
pub use move_balls::MoveBallSystem;
pub use paddle::PaddleSystem;
pub use score::ScoreSystem;

mod bounce;
mod fps;
mod move_balls;
mod paddle;
mod score;

use crate::core::FPS;
use amethyst::core::timing::Time;
use amethyst::ecs::{Read, ReadExpect, System, WriteStorage};
use amethyst::ui::UiText;
use amethyst::utils::fps_counter::FpsCounter;

pub struct FpsSystem;

impl<'s> System<'s> for FpsSystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        ReadExpect<'s, FPS>,
        ReadExpect<'s, FpsCounter>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut ui_text, fps, fps_counter, time): Self::SystemData) {
        if time.frame_number() % 30 == 0 {
            if let Some(text) = ui_text.get_mut(fps.0) {
                text.text = format!("{:.0}", fps_counter.sampled_fps());
            }
        }
    }
}

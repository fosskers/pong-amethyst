use crate::core::FPS;
use amethyst::ecs::{ReadExpect, System, WriteStorage};
use amethyst::ui::UiText;
use amethyst::utils::fps_counter::FpsCounter;

pub struct FpsSystem;

impl<'s> System<'s> for FpsSystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        ReadExpect<'s, FPS>,
        ReadExpect<'s, FpsCounter>,
    );

    fn run(&mut self, (mut ui_text, fps, fps_counter): Self::SystemData) {
        if let Some(text) = ui_text.get_mut(fps.0) {
            text.text = format!("{:.0}", fps_counter.sampled_fps());
        }
    }
}

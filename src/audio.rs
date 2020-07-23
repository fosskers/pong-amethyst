//! Loading audio files and toggling music.

use amethyst::assets::{AssetStorage, Loader};
use amethyst::audio::output::Output;
use amethyst::audio::{AudioSink, OggFormat, Source, SourceHandle};
use amethyst::ecs::{World, WorldExt};
use std::iter::Cycle;
use std::vec::IntoIter;

const BOUNCE_SOUND: &str = "audio/bounce.ogg";
const SCORE_SOUND: &str = "audio/score.ogg";

const MUSIC_TRACKS: &[&str] = &["audio/jetpack.ogg", "audio/albatross.ogg"];

pub struct Sounds {
    pub bounce_sfx: SourceHandle,
    pub score_sfx: SourceHandle,
}

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

pub fn initialize_audio(world: &mut World) {
    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let mut sink = world.write_resource::<AudioSink>();
        sink.set_volume(0.25);
        let music = MUSIC_TRACKS
            .iter()
            .map(|file| load_audio_track(&loader, &world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();
        let music = Music { music };

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, &world, BOUNCE_SOUND),
            score_sfx: load_audio_track(&loader, &world, SCORE_SOUND),
        };

        (sound, music)
    };

    world.insert(sound_effects);
    world.insert(music);
}

/// Turn the background music on or off, depending on its current state.
pub fn toggle_bgm(world: &mut World) {
    let sink = world.read_resource::<AudioSink>();
    if sink.is_paused() {
        sink.play();
    } else {
        sink.pause();
    }
}

pub fn play_bounce_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(o) = output {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            o.play_once(sound, 1.0);
        }
    }
}

pub fn play_score_sound(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(o) = output {
        if let Some(sound) = storage.get(&sounds.score_sfx) {
            o.play_once(sound, 1.0);
        }
    }
}

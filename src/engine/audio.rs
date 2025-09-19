use crate::common::Rand;
use crate::engine::Time;
use bevy_ecs::prelude::*;
use quad_snd::{AudioContext, PlaySoundParams, Sound};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct DelayedAudioEntry {
    key: AudioKey,
    volume: f32,
    remaining_delay: f32,
}

macro_rules! define_audio {
    ($(
        $variant:ident => $file:literal
    ),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum AudioKey {
            $(
                $variant,
            )*
        }

        impl AudioKey {
            pub fn bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        AudioKey::$variant => include_bytes!(concat!("../assets/audio/", $file)),
                    )*
                }
            }

            pub fn all() -> &'static [AudioKey] {
                &[
                    $(
                        AudioKey::$variant,
                    )*
                ]
            }
        }
    };
}

define_audio! {
    Mining1 => "mining_1.wav",
    Mining2 => "mining_2.wav",
    Mining3 => "mining_3.wav",
    RockBreak => "rock_break.wav",
    Vegetation => "vegetation.wav",
    Woodcut1 => "woodcut_1.wav",
    Woodcut2 => "woodcut_2.wav",
    Button1 => "button_1.wav",
    ButtonBack1 => "button_back_1.wav",
    RifleEmpty => "rifle_empty.wav",
    RifleShoot2 => "rifle_shoot_1.wav",
    RifleReload => "revolver_reload.wav",
    RifleReloadComplete => "rifle_reload.wav",
    RevolverEmpty => "revolver_empty.wav",
    RevolverShoot1 => "revolver_shoot_1.wav",
    RevolverReload => "revolver_reload.wav",
    RevolverReloadComplete => "revolver_cylinder_spin_1.wav",
    ShotgunEmpty => "shotgun_empty.wav",
    ShotgunShoot1 => "shotgun_shoot_1.wav",
    ShotgunReload => "shotgun_reload_bullet_1.wav",
    ShotgunReloadComplete => "shotgun_reload.wav",
    Pain1 => "pain_1.wav",
    Pain2 => "pain_2.wav",
    Growl1 => "growl_1.wav",
    Growl2 => "growl_2.wav",
    Hiss1 => "hiss_1.wav",
    Bark1 => "bark_1.wav",
    Punch1 => "punch_1.wav",
    Explosion1 => "explosion_1.wav",
    IgniteMatch => "ignite_match.wav",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCollection {
    Mining,
    RockCrumble,
    Chopping,
    Vegetation,
    Pain,
}

#[derive(Resource)]
pub struct Audio {
    pub ctx: Arc<Mutex<AudioContext>>,
    pub sounds: HashMap<AudioKey, Sound>,
    pub collections: HashMap<AudioCollection, Vec<AudioKey>>,
    delayed_queue: Vec<DelayedAudioEntry>,
}

impl Audio {
    pub fn load() -> Self {
        let ctx = Arc::new(Mutex::new(AudioContext::new()));
        let ctx_guard = ctx.lock().unwrap();

        let mut sounds = HashMap::new();
        for &audio_key in AudioKey::all() {
            sounds.insert(audio_key, Sound::load(&ctx_guard, audio_key.bytes()));
        }

        let mut collections = HashMap::new();
        collections.insert(
            AudioCollection::Mining,
            vec![AudioKey::Mining1, AudioKey::Mining2, AudioKey::Mining3],
        );
        collections.insert(AudioCollection::RockCrumble, vec![AudioKey::RockBreak]);
        collections.insert(
            AudioCollection::Chopping,
            vec![AudioKey::Woodcut1, AudioKey::Woodcut2],
        );
        collections.insert(AudioCollection::Vegetation, vec![AudioKey::Vegetation]);
        collections.insert(
            AudioCollection::Pain,
            vec![AudioKey::Pain1, AudioKey::Pain2],
        );

        Self {
            ctx: Arc::clone(&ctx),
            sounds,
            collections,
            delayed_queue: Vec::new(),
        }
    }

    pub fn get(&self, key: AudioKey) -> &Sound {
        &self.sounds[&key]
    }

    pub fn play(&self, key: AudioKey, volume: f32) {
        if let Ok(ctx) = self.ctx.lock() {
            self.get(key).play(
                &ctx,
                PlaySoundParams {
                    looped: false,
                    volume,
                },
            );
        }
    }

    pub fn play_random_from_collection(
        &self,
        collection: AudioCollection,
        rand: &mut Rand,
        volume: f32,
    ) {
        if let Some(keys) = self.collections.get(&collection)
            && !keys.is_empty()
        {
            let index = rand.pick_idx(keys);
            let key = keys[index];
            self.play(key, volume);
        }
    }

    pub fn play_delayed(&mut self, key: AudioKey, volume: f32, delay: f32) {
        self.delayed_queue.push(DelayedAudioEntry {
            key,
            volume,
            remaining_delay: delay,
        });
    }
}

pub fn process_delayed_audio(mut audio: ResMut<Audio>, time: Res<Time>) {
    let dt = time.dt;
    let mut to_play = Vec::new();

    // Process queue, decrement delays
    audio.delayed_queue.retain_mut(|entry| {
        entry.remaining_delay -= dt;
        if entry.remaining_delay <= 0.0 {
            to_play.push((entry.key, entry.volume));
            false // Remove from queue
        } else {
            true // Keep in queue
        }
    });

    // Play sounds that are ready
    for (key, volume) in to_play {
        audio.play(key, volume);
    }
}

use crate::common::Rand;
use crate::domain::PlayerPosition;
use crate::engine::Time;
use bevy_ecs::prelude::*;
use quad_snd::{AudioContext, PlaySoundParams, Sound};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Positional audio configuration constants
const MAX_AUDIO_DISTANCE: usize = 20;
const MIN_AUDIBLE_VOLUME: f32 = 0.05;

#[derive(Debug, Clone)]
pub enum AudioSource {
    Clip(AudioKey),
    Collection(AudioCollection),
}

#[derive(Debug, Clone)]
struct QueuedAudioEntry {
    source: AudioSource,
    volume: f32,
    position: Option<(usize, usize, usize)>,
    remaining_delay: f32,
}

pub struct AudioBuilder<'a> {
    audio: &'a mut Audio,
    source: AudioSource,
    volume: f32,
    position: Option<(usize, usize, usize)>,
    delay: f32,
}

impl<'a> AudioBuilder<'a> {
    fn new(audio: &'a mut Audio, source: AudioSource) -> Self {
        Self {
            audio,
            source,
            volume: 1.0,
            position: None,
            delay: 0.0,
        }
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn position(mut self, position: (usize, usize, usize)) -> Self {
        self.position = Some(position);
        self
    }

    pub fn delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    pub fn play(self) {
        self.audio.playback_queue.push(QueuedAudioEntry {
            source: self.source,
            volume: self.volume,
            position: self.position,
            remaining_delay: self.delay,
        });
    }
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
    ChestOpen1 => "chest_open_1.wav",
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
    playback_queue: Vec<QueuedAudioEntry>,
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
            playback_queue: Vec::new(),
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

    pub fn clip(&mut self, key: AudioKey) -> AudioBuilder {
        AudioBuilder::new(self, AudioSource::Clip(key))
    }

    pub fn collection(&mut self, collection: AudioCollection) -> AudioBuilder {
        AudioBuilder::new(self, AudioSource::Collection(collection))
    }

    /// Play audio at a world position with distance-based volume attenuation
    /// Sounds from different z-levels are completely muted
    pub fn play_at_position(
        &self,
        key: AudioKey,
        base_volume: f32,
        sound_pos: (usize, usize, usize),
        player_pos: &PlayerPosition,
    ) {
        // First check z-level: if different, don't play at all
        if sound_pos.2 != player_pos.z.floor() as usize {
            return;
        }

        // Calculate Manhattan distance (2D only since z-levels match)
        let distance = ((sound_pos.0 as i32 - player_pos.x as i32).abs()
            + (sound_pos.1 as i32 - player_pos.y as i32).abs()) as usize;

        // Apply distance-based volume attenuation
        let attenuated_volume = if distance >= MAX_AUDIO_DISTANCE {
            0.0 // Completely silent beyond max distance
        } else {
            base_volume * (1.0 - (distance as f32 / MAX_AUDIO_DISTANCE as f32))
        };

        // Only play if volume is above minimum threshold
        if attenuated_volume >= MIN_AUDIBLE_VOLUME {
            self.play(key, attenuated_volume);
        }
    }
}

pub fn process_audio_queue(
    mut audio: ResMut<Audio>,
    time: Res<Time>,
    player_pos: Option<Res<PlayerPosition>>,
    mut rand: ResMut<Rand>,
) {
    let dt = time.dt;
    let mut to_play = Vec::new();

    // Process unified playback queue
    audio.playback_queue.retain_mut(|entry| {
        entry.remaining_delay -= dt;
        if entry.remaining_delay <= 0.0 {
            to_play.push(entry.clone());
            false // Remove from queue
        } else {
            true // Keep in queue
        }
    });

    // Play sounds that are ready
    for entry in to_play {
        let audio_key = match entry.source {
            AudioSource::Clip(key) => key,
            AudioSource::Collection(collection) => {
                // Pick random key from collection
                if let Some(keys) = audio.collections.get(&collection) {
                    if !keys.is_empty() {
                        let index = rand.pick_idx(keys);
                        keys[index]
                    } else {
                        continue; // Skip empty collections
                    }
                } else {
                    continue; // Skip unknown collections
                }
            }
        };

        // Play with positional audio if position is specified
        if let Some(position) = entry.position {
            if let Some(ref player_pos) = player_pos {
                audio.play_at_position(audio_key, entry.volume, position, player_pos);
            }
        } else {
            // Play without positional audio
            audio.play(audio_key, entry.volume);
        }
    }
}

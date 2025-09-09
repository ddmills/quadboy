use crate::common::Rand;
use bevy_ecs::prelude::*;
use quad_snd::{AudioContext, PlaySoundParams, Sound};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioKey {
    Mining1,
    Mining2,
    Mining3,
    RockBreak,
    Vegetation,
    Woodcut1,
    Woodcut2,
    Button1,
    ButtonBack1,
    RevolverShoot1,
    RifleShoot1,
    ShotgunShoot1,
}

impl AudioKey {
    pub fn bytes(&self) -> &'static [u8] {
        match self {
            AudioKey::Mining1 => include_bytes!("../assets/audio/mining_1.wav"),
            AudioKey::Mining2 => include_bytes!("../assets/audio/mining_2.wav"),
            AudioKey::Mining3 => include_bytes!("../assets/audio/mining_3.wav"),
            AudioKey::RockBreak => include_bytes!("../assets/audio/rock_break.wav"),
            AudioKey::Vegetation => include_bytes!("../assets/audio/vegetation.wav"),
            AudioKey::Woodcut1 => include_bytes!("../assets/audio/woodcut_1.wav"),
            AudioKey::Woodcut2 => include_bytes!("../assets/audio/woodcut_2.wav"),
            AudioKey::Button1 => include_bytes!("../assets/audio/button_1.wav"),
            AudioKey::ButtonBack1 => include_bytes!("../assets/audio/button_back_1.wav"),
            AudioKey::RevolverShoot1 => include_bytes!("../assets/audio/revolver_shoot_1.wav"),
            AudioKey::RifleShoot1 => include_bytes!("../assets/audio/rifle_shoot_1.wav"),
            AudioKey::ShotgunShoot1 => include_bytes!("../assets/audio/shotgun_shoot_1.wav"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCollection {
    Mining,
    RockCrumble,
    Chopping,
    Vegetation,
}

#[derive(Resource)]
pub struct Audio {
    pub ctx: Arc<Mutex<AudioContext>>,
    pub sounds: HashMap<AudioKey, Sound>,
    pub collections: HashMap<AudioCollection, Vec<AudioKey>>,
}

impl Audio {
    pub fn load() -> Self {
        let ctx = Arc::new(Mutex::new(AudioContext::new()));
        let ctx_guard = ctx.lock().unwrap();

        let mut sounds = HashMap::new();
        sounds.insert(
            AudioKey::Mining1,
            Sound::load(&ctx_guard, AudioKey::Mining1.bytes()),
        );
        sounds.insert(
            AudioKey::Mining2,
            Sound::load(&ctx_guard, AudioKey::Mining2.bytes()),
        );
        sounds.insert(
            AudioKey::Mining3,
            Sound::load(&ctx_guard, AudioKey::Mining3.bytes()),
        );
        sounds.insert(
            AudioKey::RockBreak,
            Sound::load(&ctx_guard, AudioKey::RockBreak.bytes()),
        );
        sounds.insert(
            AudioKey::Vegetation,
            Sound::load(&ctx_guard, AudioKey::Vegetation.bytes()),
        );
        sounds.insert(
            AudioKey::Woodcut1,
            Sound::load(&ctx_guard, AudioKey::Woodcut1.bytes()),
        );
        sounds.insert(
            AudioKey::Woodcut2,
            Sound::load(&ctx_guard, AudioKey::Woodcut2.bytes()),
        );
        sounds.insert(
            AudioKey::Button1,
            Sound::load(&ctx_guard, AudioKey::Button1.bytes()),
        );
        sounds.insert(
            AudioKey::ButtonBack1,
            Sound::load(&ctx_guard, AudioKey::ButtonBack1.bytes()),
        );
        sounds.insert(
            AudioKey::RevolverShoot1,
            Sound::load(&ctx_guard, AudioKey::RevolverShoot1.bytes()),
        );
        sounds.insert(
            AudioKey::RifleShoot1,
            Sound::load(&ctx_guard, AudioKey::RifleShoot1.bytes()),
        );
        sounds.insert(
            AudioKey::ShotgunShoot1,
            Sound::load(&ctx_guard, AudioKey::ShotgunShoot1.bytes()),
        );

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

        Self {
            ctx: Arc::clone(&ctx),
            sounds,
            collections,
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
}

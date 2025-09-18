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
    RevolverReload,
    RifleReload,
    ShotgunReload,
    RevolverEmpty,
    RifleEmpty,
    ShotgunEmpty,
    Pain1,
    Pain2,
    Growl1,
    Growl2,
    Hiss1,
    Bark1,
    Punch1,
    Explosion1,
    IgniteMatch,
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
            AudioKey::RevolverReload => include_bytes!("../assets/audio/revolver_reload.wav"),
            AudioKey::RifleReload => include_bytes!("../assets/audio/rifle_reload.wav"),
            AudioKey::ShotgunReload => include_bytes!("../assets/audio/shotgun_reload.wav"),
            AudioKey::RevolverEmpty => include_bytes!("../assets/audio/revolver_empty.wav"),
            AudioKey::RifleEmpty => include_bytes!("../assets/audio/rifle_empty.wav"),
            AudioKey::ShotgunEmpty => include_bytes!("../assets/audio/shotgun_empty.wav"),
            AudioKey::Pain1 => include_bytes!("../assets/audio/pain_1.wav"),
            AudioKey::Pain2 => include_bytes!("../assets/audio/pain_2.wav"),
            AudioKey::Growl1 => include_bytes!("../assets/audio/growl_1.wav"),
            AudioKey::Growl2 => include_bytes!("../assets/audio/growl_2.wav"),
            AudioKey::Hiss1 => include_bytes!("../assets/audio/hiss_1.wav"),
            AudioKey::Bark1 => include_bytes!("../assets/audio/bark_1.wav"),
            AudioKey::Punch1 => include_bytes!("../assets/audio/punch_1.wav"),
            AudioKey::Explosion1 => include_bytes!("../assets/audio/explosion_1.wav"),
            AudioKey::IgniteMatch => include_bytes!("../assets/audio/ignite_match.wav"),
        }
    }
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
        sounds.insert(
            AudioKey::RevolverReload,
            Sound::load(&ctx_guard, AudioKey::RevolverReload.bytes()),
        );
        sounds.insert(
            AudioKey::RifleReload,
            Sound::load(&ctx_guard, AudioKey::RifleReload.bytes()),
        );
        sounds.insert(
            AudioKey::ShotgunReload,
            Sound::load(&ctx_guard, AudioKey::ShotgunReload.bytes()),
        );
        sounds.insert(
            AudioKey::RevolverEmpty,
            Sound::load(&ctx_guard, AudioKey::RevolverEmpty.bytes()),
        );
        sounds.insert(
            AudioKey::RifleEmpty,
            Sound::load(&ctx_guard, AudioKey::RifleEmpty.bytes()),
        );
        sounds.insert(
            AudioKey::ShotgunEmpty,
            Sound::load(&ctx_guard, AudioKey::ShotgunEmpty.bytes()),
        );
        sounds.insert(
            AudioKey::Pain1,
            Sound::load(&ctx_guard, AudioKey::Pain1.bytes()),
        );
        sounds.insert(
            AudioKey::Pain2,
            Sound::load(&ctx_guard, AudioKey::Pain2.bytes()),
        );
        sounds.insert(
            AudioKey::Growl1,
            Sound::load(&ctx_guard, AudioKey::Growl1.bytes()),
        );
        sounds.insert(
            AudioKey::Growl2,
            Sound::load(&ctx_guard, AudioKey::Growl2.bytes()),
        );
        sounds.insert(
            AudioKey::Hiss1,
            Sound::load(&ctx_guard, AudioKey::Hiss1.bytes()),
        );
        sounds.insert(
            AudioKey::Bark1,
            Sound::load(&ctx_guard, AudioKey::Bark1.bytes()),
        );
        sounds.insert(
            AudioKey::Punch1,
            Sound::load(&ctx_guard, AudioKey::Punch1.bytes()),
        );
        sounds.insert(
            AudioKey::Explosion1,
            Sound::load(&ctx_guard, AudioKey::Explosion1.bytes()),
        );
        sounds.insert(
            AudioKey::IgniteMatch,
            Sound::load(&ctx_guard, AudioKey::IgniteMatch.bytes()),
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
        collections.insert(
            AudioCollection::Pain,
            vec![AudioKey::Pain1, AudioKey::Pain2],
        );

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

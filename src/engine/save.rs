#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{self, File},
    io::Write,
};

#[cfg(target_arch = "wasm32")]
use web_sys;

use macroquad::prelude::{error, warn};

use crate::{
    cfg::{ENABLE_SAVES, SAVE_NAME},
    domain::ZoneSaveData,
};

pub fn save_zone(zone: &ZoneSaveData) {
    if !ENABLE_SAVES {
        return;
    }

    let Ok(save_data) = serde_json::to_string(zone) else {
        error!("could not save zone!");
        return;
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        fs::create_dir_all(format!("saves/{}", SAVE_NAME))
            .expect("Error while creating save directory");
    }

    let file_path = format!("saves/{}/zone-{}.json", SAVE_NAME, zone.idx);

    store(file_path, save_data);
}

#[cfg(not(target_arch = "wasm32"))]
fn store(file_path: String, data: String) {
    File::create(file_path)
        .and_then(|mut file| file.write(data.as_bytes()))
        .expect("Error while writing save file");
}

#[cfg(target_arch = "wasm32")]
fn store(file_path: String, data: String) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            error!("Could not access window for localStorage");
            return;
        }
    };

    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        Ok(None) => {
            error!("localStorage is not available");
            return;
        }
        Err(_) => {
            error!("Error accessing localStorage");
            return;
        }
    };

    if let Err(_) = storage.set_item(&file_path, &data) {
        error!("Failed to save to localStorage: {}", file_path);
    }
}

pub fn try_load_zone(zone_idx: usize) -> Option<ZoneSaveData> {
    if !ENABLE_SAVES {
        return None;
    }

    let file_path = format!("saves/{}/zone-{}.json", SAVE_NAME, zone_idx);
    let contents = read(&file_path)?;

    let Ok(zone) = serde_json::from_str::<ZoneSaveData>(&contents) else {
        warn!("Could not deserialize zone save! corrupt? {}", file_path);
        return None;
    };

    Some(zone)
}

#[cfg(not(target_arch = "wasm32"))]
fn read(file_path: &String) -> Option<String> {
    fs::read_to_string(file_path).ok()
}

#[cfg(target_arch = "wasm32")]
fn read(file_path: &String) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;

    match storage.get_item(&file_path) {
        Ok(result) => result,
        Err(_) => {
            warn!("Failed to read from localStorage: {}", file_path);
            None
        }
    }
}

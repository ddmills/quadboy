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

pub fn delete_save(save_name: &str) {
    if !ENABLE_SAVES {
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let save_path = format!("saves/{}", save_name);
        if let Err(e) = std::fs::remove_dir_all(&save_path) {
            warn!("Failed to delete save directory {}: {}", save_path, e);
        } else {
            warn!("Deleted save directory: {}", save_path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        delete_save_wasm(save_name);
    }
}

#[cfg(target_arch = "wasm32")]
fn delete_save_wasm(save_name: &str) {
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

    // Get all localStorage keys to find matching save files
    let length = match storage.length() {
        Ok(len) => len,
        Err(_) => {
            error!("Failed to get localStorage length");
            return;
        }
    };

    let save_prefix = format!("saves/{}/", save_name);
    let mut keys_to_delete = Vec::new();

    // Collect all keys that match the save pattern
    for i in 0..length {
        if let Ok(Some(key)) = storage.key(i) {
            if key.starts_with(&save_prefix) {
                keys_to_delete.push(key);
            }
        }
    }

    // Delete all matching keys
    let mut deleted_count = 0;
    for key in keys_to_delete {
        if storage.remove_item(&key).is_ok() {
            deleted_count += 1;
        } else {
            warn!("Failed to delete localStorage key: {}", key);
        }
    }

    if deleted_count > 0 {
        warn!(
            "Deleted {} save files for save: {}",
            deleted_count, save_name
        );
    } else {
        warn!("No save files found for save: {}", save_name);
    }
}

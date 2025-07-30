use std::{
    fs::{self, File},
    io::Write,
};

use macroquad::prelude::{debug, error, warn};

use crate::{cfg::{ENABLE_SAVES}, domain::ZoneSaveData};

pub fn save_zone(zone: &ZoneSaveData) {
    if !ENABLE_SAVES {
        return;
    }

    let Ok(save_data) = ron::to_string(zone) else {
        error!("could not save zone!");
        return;
    };

    let file_path = format!("saves/zone-{}.ron", zone.idx);
    debug!("saving {}", file_path);
    store(file_path, save_data);
}

#[cfg(not(target_arch = "wasm32"))]
fn store(file_path: String, data: String) {
    File::create(file_path)
        .and_then(|mut file| file.write(data.as_bytes()))
        .expect("Error while writing save file");
}

#[cfg(target_arch = "wasm32")]
fn store(file_path: String, data: String) -> Option<()> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;

    storage.set_item(&file_path, &data).ok()
}

pub fn try_load_zone(zone_idx: usize) -> Option<ZoneSaveData> {
    if !ENABLE_SAVES {
        return None;
    }

    let file_path = format!("saves/zone-{}.ron", zone_idx);

    debug!("loading {}", file_path);

    let contents = read(&file_path)?;

    let Ok(zone) = ron::from_str::<ZoneSaveData>(&contents) else {
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

    storage.get_item(&file_path).ok()?
}

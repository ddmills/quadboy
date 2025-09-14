use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{Equippable, Equipped, LightSource, Lightable},
    engine::AudioKey,
    ui::ListItemData,
};

pub trait ItemAction {
    fn label(&self, item_entity: Entity, world: &World) -> String;
    fn is_available(&self, item_entity: Entity, world: &World) -> bool;
    fn hotkey(&self) -> Option<KeyCode>;
    fn audio_key(&self) -> Option<AudioKey>;
    fn create_list_item_data(
        &self,
        item_entity: Entity,
        world: &World,
        callback: SystemId,
    ) -> Option<ListItemData>;
}

pub struct DropAction;

impl ItemAction for DropAction {
    fn label(&self, _item_entity: Entity, _world: &World) -> String {
        "Drop".to_string()
    }

    fn is_available(&self, _item_entity: Entity, _world: &World) -> bool {
        true // All inventory items can be dropped
    }

    fn hotkey(&self) -> Option<KeyCode> {
        Some(KeyCode::U)
    }

    fn audio_key(&self) -> Option<AudioKey> {
        None
    }

    fn create_list_item_data(
        &self,
        item_entity: Entity,
        world: &World,
        callback: SystemId,
    ) -> Option<ListItemData> {
        if !self.is_available(item_entity, world) {
            return None;
        }

        let label = format!(
            "[{{Y|{}}}] {}",
            self.hotkey()
                .map(|k| format!("{:?}", k).to_uppercase())
                .unwrap_or_default(),
            self.label(item_entity, world)
        );

        let mut item = ListItemData::new(&label, callback);
        if let Some(hotkey) = self.hotkey() {
            item = item.with_hotkey(hotkey);
        }
        if let Some(audio) = self.audio_key() {
            item = item.with_audio(audio);
        }

        Some(item)
    }
}

pub struct EquipAction;

impl ItemAction for EquipAction {
    fn label(&self, item_entity: Entity, world: &World) -> String {
        if world.get::<Equipped>(item_entity).is_some() {
            "Unequip".to_string()
        } else {
            "Equip".to_string()
        }
    }

    fn is_available(&self, item_entity: Entity, world: &World) -> bool {
        world.get::<Equippable>(item_entity).is_some()
    }

    fn hotkey(&self) -> Option<KeyCode> {
        Some(KeyCode::E)
    }

    fn audio_key(&self) -> Option<AudioKey> {
        None
    }

    fn create_list_item_data(
        &self,
        item_entity: Entity,
        world: &World,
        callback: SystemId,
    ) -> Option<ListItemData> {
        if !self.is_available(item_entity, world) {
            return None;
        }

        let label = format!(
            "[{{Y|{}}}] {}",
            self.hotkey()
                .map(|k| format!("{:?}", k).to_uppercase())
                .unwrap_or_default(),
            self.label(item_entity, world)
        );

        let mut item = ListItemData::new(&label, callback);
        if let Some(hotkey) = self.hotkey() {
            item = item.with_hotkey(hotkey);
        }
        if let Some(audio) = self.audio_key() {
            item = item.with_audio(audio);
        }

        Some(item)
    }
}

pub struct ToggleLightItemAction;

impl ItemAction for ToggleLightItemAction {
    fn label(&self, item_entity: Entity, world: &World) -> String {
        if let Some(lightable) = world.get::<Lightable>(item_entity) {
            lightable.action_label.clone()
        } else {
            "Toggle Light".to_string()
        }
    }

    fn is_available(&self, item_entity: Entity, world: &World) -> bool {
        world.get::<Lightable>(item_entity).is_some()
            && world.get::<LightSource>(item_entity).is_some()
    }

    fn hotkey(&self) -> Option<KeyCode> {
        Some(KeyCode::L)
    }

    fn audio_key(&self) -> Option<AudioKey> {
        None
    }

    fn create_list_item_data(
        &self,
        item_entity: Entity,
        world: &World,
        callback: SystemId,
    ) -> Option<ListItemData> {
        if !self.is_available(item_entity, world) {
            return None;
        }

        let label = format!(
            "[{{Y|{}}}] {}",
            self.hotkey()
                .map(|k| format!("{:?}", k).to_uppercase())
                .unwrap_or_default(),
            self.label(item_entity, world)
        );

        let mut item = ListItemData::new(&label, callback);
        if let Some(hotkey) = self.hotkey() {
            item = item.with_hotkey(hotkey);
        }
        if let Some(audio) = self.audio_key() {
            item = item.with_audio(audio);
        }

        Some(item)
    }
}

#[derive(Resource)]
pub struct ItemActionRegistry {
    pub actions: Vec<Box<dyn ItemAction + Send + Sync>>,
}

impl ItemActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: vec![
                Box::new(DropAction),
                Box::new(EquipAction),
                Box::new(ToggleLightItemAction),
            ],
        }
    }

    pub fn get_available_actions(
        &self,
        item_entity: Entity,
        world: &World,
    ) -> Vec<&Box<dyn ItemAction + Send + Sync>> {
        self.actions
            .iter()
            .filter(|action| action.is_available(item_entity, world))
            .collect()
    }

    pub fn create_action_list(
        &self,
        item_entity: Entity,
        world: &World,
        callbacks: &ItemActionCallbacks,
    ) -> Vec<ListItemData> {
        let mut actions = Vec::new();

        for action in &self.actions {
            if !action.is_available(item_entity, world) {
                continue;
            }

            let callback = match action.as_ref() {
                action if action.label(item_entity, world).contains("Drop") => callbacks.drop,
                action
                    if action.label(item_entity, world).contains("Equip")
                        || action.label(item_entity, world).contains("Unequip") =>
                {
                    callbacks.toggle_equip
                }
                action
                    if action.label(item_entity, world).contains("Light")
                        || action.label(item_entity, world).contains("Extinguish") =>
                {
                    callbacks.toggle_light
                }
                _ => continue,
            };

            if let Some(list_item) = action.create_list_item_data(item_entity, world, callback) {
                actions.push(list_item);
            }
        }

        actions
    }
}

#[derive(Resource)]
pub struct ItemActionCallbacks {
    pub drop: SystemId,
    pub toggle_equip: SystemId,
    pub toggle_light: SystemId,
}

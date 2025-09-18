use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Resource)]
pub struct LabelModifierRegistry {
    modifiers: Vec<(
        i32,
        Box<dyn Fn(&World, Entity) -> Option<String> + Send + Sync>,
    )>,
}

impl LabelModifierRegistry {
    pub fn new() -> Self {
        Self {
            modifiers: Vec::new(),
        }
    }

    pub fn register<C: Component>(
        &mut self,
        priority: i32,
        formatter: impl Fn(&C) -> Option<String> + Send + Sync + 'static,
    ) {
        self.modifiers.push((
            priority,
            Box::new(move |world, entity| world.get::<C>(entity).and_then(|comp| formatter(comp))),
        ));
    }

    pub fn build_label(&self, world: &World, entity: Entity, base_label: &str) -> String {
        let mut parts = vec![base_label.to_string()];

        let mut modifiers: Vec<(i32, String)> = self
            .modifiers
            .iter()
            .filter_map(|(priority, modifier_fn)| {
                modifier_fn(world, entity).map(|text| (*priority, text))
            })
            .collect();

        modifiers.sort_by_key(|(priority, _)| *priority);

        for (_, text) in modifiers {
            parts.push(text);
        }

        parts.join(" ")
    }
}

impl Default for LabelModifierRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DynamicLabel {
    pub cached: String,
    pub dirty: bool,
}

impl DynamicLabel {
    pub fn new(label: String) -> Self {
        Self {
            cached: label,
            dirty: false,
        }
    }

    pub fn get(&self) -> &str {
        &self.cached
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn update(&mut self, new_label: String) {
        if self.cached != new_label {
            self.cached = new_label;
        }
        self.dirty = false;
    }
}

pub fn get_dynamic_label(
    entity: Entity,
    q_labels: &Query<&crate::domain::components::Label>,
    q_dynamic_labels: &Query<&DynamicLabel>,
) -> String {
    if let Ok(dynamic_label) = q_dynamic_labels.get(entity) {
        dynamic_label.get().to_string()
    } else if let Ok(base_label) = q_labels.get(entity) {
        base_label.get().to_string()
    } else {
        "Unknown".to_string()
    }
}

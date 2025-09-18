use bevy_ecs::prelude::*;

use crate::domain::components::Label;

pub fn update_labels(
    mut q: Query<(Entity, &mut Label)>,
    // Query all the components that can modify labels
    q_equipped: Query<&crate::domain::components::Equipped>,
    q_fuse: Query<&crate::domain::components::Fuse>,
    q_light_source: Query<&crate::domain::components::LightSource>,
    q_stack_count: Query<&crate::domain::components::StackCount>,
) {
    for (entity, mut label) in q.iter_mut() {
        if label.is_dirty() {
            let mut parts = vec![label.get_base().to_string()];
            let mut modifiers: Vec<(i32, String)> = Vec::new();

            // Check equipped component
            if let Ok(equipped) = q_equipped.get(entity) {
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                modifiers.push((100, format!("{{G|[{}]}}", slot_name)));
            }

            // Check fuse component
            if let Ok(fuse) = q_fuse.get(entity) {
                if fuse.remaining_ticks > 0 {
                    modifiers.push((50, format!("{{Y|[Lit]}} {}", fuse.get_countdown_display())));
                }
            }

            // Check light source component
            if let Ok(light) = q_light_source.get(entity) {
                if light.is_enabled {
                    modifiers.push((75, "{Y|[Lit]}".to_string()));
                }
            }

            // Check stack count component
            if let Ok(stack) = q_stack_count.get(entity) {
                if stack.count > 1 {
                    modifiers.push((200, format!("x{}", stack.count)));
                }
            }

            // Sort by priority and append
            modifiers.sort_by_key(|(priority, _)| *priority);
            for (_, text) in modifiers {
                parts.push(text);
            }

            let new_cached_label = parts.join(" ");
            label.update_cache(new_cached_label);
        }
    }
}

pub fn mark_dirty_on_equipment_change(
    mut q: Query<&mut Label>,
    q_equipped: Query<Entity, (Changed<crate::domain::components::Equipped>, With<Label>)>,
) {
    for entity in q_equipped.iter() {
        if let Ok(mut label) = q.get_mut(entity) {
            label.mark_dirty();
        }
    }
}

pub fn mark_dirty_on_fuse_change(
    mut q: Query<&mut Label>,
    q_fuse: Query<Entity, (Changed<crate::domain::components::Fuse>, With<Label>)>,
) {
    for entity in q_fuse.iter() {
        if let Ok(mut label) = q.get_mut(entity) {
            label.mark_dirty();
        }
    }
}

pub fn mark_dirty_on_light_change(
    mut q: Query<&mut Label>,
    q_light: Query<Entity, (Changed<crate::domain::components::LightSource>, With<Label>)>,
) {
    for entity in q_light.iter() {
        if let Ok(mut label) = q.get_mut(entity) {
            label.mark_dirty();
        }
    }
}

pub fn mark_dirty_on_stack_change(
    mut q: Query<&mut Label>,
    q_stack: Query<Entity, (Changed<crate::domain::components::StackCount>, With<Label>)>,
) {
    for entity in q_stack.iter() {
        if let Ok(mut label) = q.get_mut(entity) {
            label.mark_dirty();
        }
    }
}

pub fn ensure_labels_initialized(
    mut q_labels: Query<(Entity, &mut Label)>,
    // Query all the components that can modify labels to build them immediately
    q_equipped: Query<&crate::domain::components::Equipped>,
    q_fuse: Query<&crate::domain::components::Fuse>,
    q_light_source: Query<&crate::domain::components::LightSource>,
    q_stack_count: Query<&crate::domain::components::StackCount>,
) {
    for (entity, mut label) in q_labels.iter_mut() {
        // Build the cached label with all modifiers for newly created labels
        if label.get() == label.get_base() && !label.is_dirty() {
            let mut parts = vec![label.get_base().to_string()];
            let mut modifiers: Vec<(i32, String)> = Vec::new();

            // Check equipped component
            if let Ok(equipped) = q_equipped.get(entity) {
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                modifiers.push((100, format!("[{}]", slot_name)));
            }

            // Check fuse component
            if let Ok(fuse) = q_fuse.get(entity) {
                if fuse.remaining_ticks > 0 {
                    modifiers.push((50, format!("[Lit] {}", fuse.get_countdown_display())));
                }
            }

            // Check light source component
            if let Ok(light) = q_light_source.get(entity) {
                if light.is_enabled {
                    modifiers.push((75, "[Lit]".to_string()));
                }
            }

            // Check stack count component
            if let Ok(stack) = q_stack_count.get(entity) {
                if stack.count > 1 {
                    modifiers.push((200, format!("x{}", stack.count)));
                }
            }

            // Sort by priority and append
            modifiers.sort_by_key(|(priority, _)| *priority);
            for (_, text) in modifiers {
                parts.push(text);
            }

            let full_label = parts.join(" ");
            label.update_cache(full_label);
        }
    }
}

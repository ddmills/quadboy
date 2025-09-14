use crate::{
    common::Palette,
    rendering::{Layer, Position, Text, Visibility},
    ui::{List, ListItemData},
};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ActionDropdown {
    pub target_item_id: Option<u64>,
    pub is_visible: bool,
    pub ui_spawned: bool, // Track if UI elements have been spawned
    pub last_updated_item_id: Option<u64>, // Track last item we updated content for
}

impl ActionDropdown {
    pub fn new() -> Self {
        Self {
            target_item_id: None,
            is_visible: false,
            ui_spawned: false,
            last_updated_item_id: None,
        }
    }

    pub fn show_for_item(&mut self, item_id: u64) {
        self.target_item_id = Some(item_id);
        self.is_visible = true;
        // Don't reset last_updated_item_id here - let the update system handle it
    }

    pub fn hide(&mut self) {
        self.target_item_id = None;
        self.is_visible = false;
        self.last_updated_item_id = None; // Reset when hiding
    }

    pub fn needs_content_update(&self) -> bool {
        self.is_visible && self.target_item_id != self.last_updated_item_id
    }

    pub fn mark_content_updated(&mut self) {
        self.last_updated_item_id = self.target_item_id;
    }

    pub fn is_showing_for(&self, item_id: u64) -> bool {
        self.target_item_id == Some(item_id) && self.is_visible
    }
}

#[derive(Component)]
pub struct ActionDropdownTitle {
    pub parent_dropdown: Entity,
}

#[derive(Component)]
pub struct ActionDropdownList {
    pub parent_dropdown: Entity,
}

pub fn setup_action_dropdowns(
    mut cmds: Commands,
    mut q_dropdowns: Query<(Entity, &mut ActionDropdown, &Position)>,
    q_existing_titles: Query<(Entity, &ActionDropdownTitle)>,
    q_existing_lists: Query<(Entity, &ActionDropdownList)>,
    mut q_title_visibility: Query<
        &mut Visibility,
        (With<ActionDropdownTitle>, Without<ActionDropdownList>),
    >,
    mut q_list_visibility: Query<
        &mut Visibility,
        (With<ActionDropdownList>, Without<ActionDropdownTitle>),
    >,
) {
    for (dropdown_entity, mut dropdown, dropdown_pos) in q_dropdowns.iter_mut() {
        if dropdown.is_visible && !dropdown.ui_spawned {
            // Only create UI elements if they don't exist yet

            // Create title
            cmds.spawn((
                Text::new("ACTIONS").fg1(Palette::Yellow).layer(Layer::Ui),
                Position::new_f32(dropdown_pos.x, dropdown_pos.y, dropdown_pos.z),
                ActionDropdownTitle {
                    parent_dropdown: dropdown_entity,
                },
                Visibility::Visible,
            ));

            // Create list (will be populated by update system)
            cmds.spawn((
                List::new(Vec::new()).with_focus_order(3000),
                Position::new_f32(dropdown_pos.x, dropdown_pos.y + 1.0, dropdown_pos.z),
                ActionDropdownList {
                    parent_dropdown: dropdown_entity,
                },
                Visibility::Visible,
            ));

            dropdown.ui_spawned = true;
        } else if dropdown.is_visible && dropdown.ui_spawned {
            // Show existing elements
            for (title_entity, title) in q_existing_titles.iter() {
                if title.parent_dropdown == dropdown_entity {
                    if let Ok(mut visibility) = q_title_visibility.get_mut(title_entity) {
                        *visibility = Visibility::Visible;
                    }
                }
            }
            for (list_entity, list) in q_existing_lists.iter() {
                if list.parent_dropdown == dropdown_entity {
                    if let Ok(mut visibility) = q_list_visibility.get_mut(list_entity) {
                        *visibility = Visibility::Visible;
                    }
                }
            }
        } else if !dropdown.is_visible && dropdown.ui_spawned {
            // Hide elements and clear list contents to prevent entity issues
            for (title_entity, title) in q_existing_titles.iter() {
                if title.parent_dropdown == dropdown_entity {
                    if let Ok(mut visibility) = q_title_visibility.get_mut(title_entity) {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
            for (list_entity, list) in q_existing_lists.iter() {
                if list.parent_dropdown == dropdown_entity {
                    if let Ok(mut visibility) = q_list_visibility.get_mut(list_entity) {
                        *visibility = Visibility::Hidden;
                    }
                    // Clear the list items to allow clean repopulation
                    if let Ok(mut list_component) = cmds.get_entity(list_entity) {
                        // We can't directly modify List here, so we'll do it in the update system
                    }
                }
            }
        }
    }
}

pub fn clear_hidden_dropdown_lists(
    mut q_dropdown_lists: Query<(&mut List, &Visibility), With<ActionDropdownList>>,
) {
    for (mut dropdown_list, visibility) in q_dropdown_lists.iter_mut() {
        if *visibility == Visibility::Hidden && !dropdown_list.items.is_empty() {
            // Clear list items when hidden to prevent entity churn
            dropdown_list.items.clear();
        }
    }
}

pub fn cleanup_action_dropdowns(
    mut cmds: Commands,
    q_titles: Query<Entity, With<ActionDropdownTitle>>,
    q_lists: Query<Entity, With<ActionDropdownList>>,
    mut q_dropdowns: Query<&mut ActionDropdown>,
) {
    // Reset dropdown spawn state
    for mut dropdown in q_dropdowns.iter_mut() {
        dropdown.ui_spawned = false;
        dropdown.is_visible = false;
        dropdown.last_updated_item_id = None;
    }

    // Despawn all dropdown elements
    for entity in q_titles.iter() {
        cmds.entity(entity).despawn();
    }
    for entity in q_lists.iter() {
        cmds.entity(entity).despawn();
    }
}

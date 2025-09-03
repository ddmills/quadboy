use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    engine::{KeyInput, Mouse},
    rendering::{Glyph, Layer, Position, Text, Visibility, text_content_length},
    ui::{Callback, Hotkey, Interactable, Interaction},
};

#[derive(Resource, Default)]
pub struct ListFocus {
    pub active_list: Option<Entity>,
}

#[derive(Resource)]
pub struct ListContext {
    pub activated_item_index: usize,
    pub activated_list: Entity,
    pub context_data: Option<u64>,
}

impl Default for ListContext {
    fn default() -> Self {
        Self {
            activated_item_index: 0,
            activated_list: Entity::PLACEHOLDER,
            context_data: None,
        }
    }
}

pub struct ListItemData {
    pub label: String,
    pub callback: SystemId,
    pub hotkey: Option<KeyCode>,
    pub icon: Option<Glyph>,
    pub context_data: Option<u64>,
}

#[derive(Component)]
pub struct List {
    pub items: Vec<ListItemData>,
}

#[derive(Component)]
pub struct ListState {
    pub selected_index: usize,
    pub has_focus: bool,
}

impl List {
    pub fn new(items: Vec<ListItemData>) -> Self {
        Self { items }
    }
}

impl ListState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            has_focus: false,
        }
    }

    pub fn with_focus(mut self, focus: bool) -> Self {
        self.has_focus = focus;
        self
    }
}

#[derive(Component)]
pub struct ListItem {
    pub index: usize,
    pub parent_list: Entity,
}

#[derive(Component)]
pub struct ListItemBg {
    pub index: usize,
    pub parent_list: Entity,
}

#[derive(Component)]
pub struct ListCursor {
    pub parent_list: Entity,
}

#[derive(Component)]
pub struct ListItemIcon {
    pub index: usize,
    pub parent_list: Entity,
}

pub fn setup_lists(
    mut cmds: Commands,
    mut q_lists: Query<
        (Entity, &List, &mut ListState, &Position, Option<&Children>),
        Changed<List>,
    >,
) {
    for (list_entity, list, mut list_state, list_pos, existing_children) in q_lists.iter_mut() {
        // Fix cursor position if it's out of bounds
        if list_state.selected_index >= list.items.len() && !list.items.is_empty() {
            list_state.selected_index = list.items.len() - 1;
        } else if list.items.is_empty() {
            list_state.selected_index = 0;
        }
        if let Some(children) = existing_children {
            for child in children.iter() {
                cmds.entity(child).despawn();
            }
        }

        cmds.spawn((
            Text::new("→").fg1(Palette::Yellow).layer(Layer::Ui),
            Position::new_f32(0., 0., 0.),
            ListCursor {
                parent_list: list_entity,
            },
            Visibility::Visible,
            ChildOf(list_entity),
        ));

        let has_icons = list.items.iter().any(|x| x.icon.is_some());
        let item_x = if has_icons { 3. } else { 1.0 };

        for (i, item_data) in list.items.iter().enumerate() {
            let item_spacing = if has_icons { 1.0 } else { 0.5 };
            let item_y = i as f32 * item_spacing;

            cmds.spawn((
                Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                Glyph::idx(6)
                    .scale((10., item_spacing))
                    .layer(Layer::UiPanels),
                ListItemBg {
                    index: i,
                    parent_list: list_entity,
                },
                Interaction::None,
                Interactable::new(14., item_spacing),
                ChildOf(list_entity),
            ));

            let item_offset_y = if has_icons { 0.25 } else { 0. };

            cmds.spawn((
                Text::new(&item_data.label).layer(Layer::Ui),
                Position::new_f32(
                    list_pos.x + item_x,
                    list_pos.y + item_y + item_offset_y,
                    list_pos.z,
                ),
                ListItem {
                    index: i,
                    parent_list: list_entity,
                },
                ChildOf(list_entity),
            ));

            // Note: List items don't use hotkeys directly - hotkeys are handled by buttons instead
            // Add icon if provided
            if let Some(icon) = &item_data.icon {
                cmds.spawn((
                    icon.clone().layer(Layer::Ui),
                    Position::new_f32(list_pos.x + 1.5, list_pos.y + item_y, list_pos.z),
                    ListItemIcon {
                        index: i,
                        parent_list: list_entity,
                    },
                    ChildOf(list_entity),
                ));
            }
        }
    }
}

pub fn list_navigation(
    list_focus: Res<ListFocus>,
    mut q_lists: Query<(Entity, &List, &mut ListState)>,
    keys: Res<KeyInput>,
) {
    let Some(active_list) = list_focus.active_list else {
        return;
    };

    for (entity, list, mut list_state) in q_lists.iter_mut() {
        if entity != active_list || !list_state.has_focus {
            continue;
        }

        if keys.is_pressed(KeyCode::W) && list_state.selected_index > 0 {
            list_state.selected_index -= 1;
        }

        if keys.is_pressed(KeyCode::S)
            && list_state.selected_index < list.items.len().saturating_sub(1)
        {
            list_state.selected_index += 1;
        }
    }
}

pub fn list_focus_switching(
    mut list_focus: ResMut<ListFocus>,
    mut q_lists: Query<(Entity, &List, &mut ListState)>,
    keys: Res<KeyInput>,
) {
    if keys.is_pressed(KeyCode::Tab) {
        let lists: Vec<Entity> = q_lists.iter().map(|(e, _, _)| e).collect();

        if lists.len() > 1 {
            let current_index = lists
                .iter()
                .position(|&e| Some(e) == list_focus.active_list)
                .unwrap_or(0);

            let next_index = (current_index + 1) % lists.len();
            let next_list = lists[next_index];

            for (entity, _, mut list_state) in q_lists.iter_mut() {
                list_state.has_focus = entity == next_list;
            }

            list_focus.active_list = Some(next_list);
        }
    }
}

pub fn list_mouse_hover(
    mut list_focus: ResMut<ListFocus>,
    mut q_lists: Query<(Entity, &List, &mut ListState), With<List>>,
    q_items: Query<(&ListItemBg, &Interactable, &Position)>,
    mouse: Res<Mouse>,
) {
    for (item, interactable, pos) in q_items.iter() {
        let is_hovered = mouse.ui.0 >= pos.x
            && mouse.ui.0 <= pos.x + interactable.width
            && mouse.ui.1 > pos.y
            && mouse.ui.1 < pos.y + interactable.height;

        if is_hovered {
            list_focus.active_list = Some(item.parent_list);

            for (entity, _, mut list_state) in q_lists.iter_mut() {
                if entity == item.parent_list {
                    list_state.has_focus = true;
                    list_state.selected_index = item.index;
                } else {
                    list_state.has_focus = false;
                }
            }
        }
    }
}

pub fn list_item_activation(
    mut cmds: Commands,
    mut list_context: ResMut<ListContext>,
    list_focus: Res<ListFocus>,
    q_lists: Query<(Entity, &List, &ListState)>,
    keys: Res<KeyInput>,
) {
    // Activate selected item with Enter key
    if keys.is_pressed(KeyCode::Enter) {
        let Some(active_list) = list_focus.active_list else {
            return;
        };

        for (entity, list, list_state) in q_lists.iter() {
            if entity != active_list || !list_state.has_focus {
                continue;
            }

            if list_state.selected_index < list.items.len() {
                let selected_item = &list.items[list_state.selected_index];

                // Store context before running callback
                list_context.activated_item_index = list_state.selected_index;
                list_context.activated_list = entity;
                list_context.context_data = selected_item.context_data;

                // Run the callback
                cmds.run_system(selected_item.callback);
            }
        }
    }
}

pub fn update_list_context(
    mut list_context: ResMut<ListContext>,
    list_focus: Res<ListFocus>,
    q_lists: Query<(&List, &ListState)>,
) {
    let Some(active_list) = list_focus.active_list else {
        return;
    };

    if let Ok((list, list_state)) = q_lists.get(active_list)
        && list_state.has_focus
        && list_state.selected_index < list.items.len()
    {
        let selected_item = &list.items[list_state.selected_index];
        list_context.activated_item_index = list_state.selected_index;
        list_context.activated_list = active_list;
        list_context.context_data = selected_item.context_data;
    }
}

pub fn list_styles(
    q_lists: Query<(Entity, &List, &ListState, &Position, &Children)>,
    mut q_cursors: Query<(&ListCursor, &mut Position, &mut Visibility), Without<List>>,
    mut q_items: Query<(&ListItemBg, &mut Glyph)>,
) {
    for (list_entity, list, list_state, list_pos, children) in q_lists.iter() {
        let has_icons = list.items.iter().any(|x| x.icon.is_some());
        for child in children.iter() {
            if let Ok((cursor, mut cursor_pos, mut cursor_vis)) = q_cursors.get_mut(child)
                && cursor.parent_list == list_entity
            {
                *cursor_vis = if list_state.has_focus {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };

                let cursor_spacing = if has_icons { 1.0 } else { 0.5 };
                let cursor_gap = if has_icons { 1.0 } else { 0.0 };
                cursor_pos.x = list_pos.x;
                cursor_pos.y =
                    list_pos.y + (list_state.selected_index as f32 * cursor_spacing) + cursor_gap;
                cursor_pos.z = list_pos.z;
            }
        }

        for (item, mut glyph) in q_items.iter_mut() {
            if item.parent_list != list_entity {
                continue;
            }

            if item.index == list_state.selected_index {
                if list_state.has_focus {
                    glyph.bg = Some(Palette::Gray.into());
                } else {
                    glyph.bg = Some(Palette::DarkGray.into());
                }
            } else {
                glyph.bg = Some(Palette::Clear.into());
            }
        }
    }
}

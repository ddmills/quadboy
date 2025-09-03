use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::GameSettings,
    engine::{KeyInput, Mouse, Time},
    rendering::{Glyph, Layer, Position, Text, Visibility},
    ui::{Interactable, Interaction},
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
    pub context_data: Option<u64>,
}

#[derive(Component)]
pub struct List {
    pub items: Vec<ListItemData>,
    pub width: f32,
}

#[derive(Component)]
pub struct ListState {
    pub selected_index: usize,
    pub has_focus: bool,
}

impl List {
    pub fn new(items: Vec<ListItemData>) -> Self {
        Self { items, width: 16. }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
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

pub fn setup_lists(
    mut cmds: Commands,
    mut q_lists: Query<
        (Entity, &List, &mut ListState, &Position, Option<&Children>),
        Changed<List>,
    >,
) {
    for (list_entity, list, mut list_state, list_pos, existing_children) in q_lists.iter_mut() {
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

        for (i, item_data) in list.items.iter().enumerate() {
            let item_spacing = 0.5;
            let item_y = i as f32 * item_spacing;

            cmds.spawn((
                Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                Glyph::idx(6)
                    .scale((list.width, item_spacing))
                    .layer(Layer::UiPanels),
                ListItemBg {
                    index: i,
                    parent_list: list_entity,
                },
                Interaction::None,
                Interactable::new(14., item_spacing),
                ChildOf(list_entity),
            ));

            cmds.spawn((
                Text::new(&item_data.label).layer(Layer::Ui),
                Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                ListItem {
                    index: i,
                    parent_list: list_entity,
                },
                ChildOf(list_entity),
            ));
        }
    }
}

pub fn list_navigation(
    list_focus: Res<ListFocus>,
    mut q_lists: Query<(Entity, &List, &mut ListState)>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut navigation_timer: Local<(f64, bool)>,
) {
    let Some(active_list) = list_focus.active_list else {
        return;
    };

    let now = time.fixed_t;
    let mut rate = settings.input_delay;
    let delay = settings.input_initial_delay;

    let navigation_keys_down = keys.is_down(KeyCode::W) || keys.is_down(KeyCode::S);
    let navigation_keys_pressed = keys.is_pressed(KeyCode::W) || keys.is_pressed(KeyCode::S);

    if !navigation_keys_down {
        navigation_timer.1 = false;
    }

    if keys.is_down(KeyCode::LeftShift) {
        rate /= 2.0;
    }

    let can_navigate = if navigation_keys_pressed {
        true
    } else if navigation_keys_down {
        if navigation_timer.1 {
            now - navigation_timer.0 >= rate
        } else if now - navigation_timer.0 >= delay {
            navigation_timer.1 = true;
            true
        } else {
            false
        }
    } else {
        false
    };

    if navigation_keys_down && can_navigate {
        for (entity, list, mut list_state) in q_lists.iter_mut() {
            if entity != active_list || !list_state.has_focus {
                continue;
            }

            if keys.is_down(KeyCode::W) && list_state.selected_index > 0 {
                list_state.selected_index -= 1;
                navigation_timer.0 = now;
            }

            if keys.is_down(KeyCode::S)
                && list_state.selected_index < list.items.len().saturating_sub(1)
            {
                list_state.selected_index += 1;
                navigation_timer.0 = now;
            }
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
        for child in children.iter() {
            if let Ok((cursor, mut cursor_pos, mut cursor_vis)) = q_cursors.get_mut(child)
                && cursor.parent_list == list_entity
            {
                *cursor_vis = if list_state.has_focus {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };

                cursor_pos.x = list_pos.x;
                cursor_pos.y = list_pos.y + (list_state.selected_index as f32 * 0.5);
                cursor_pos.z = list_pos.z;
            }
        }

        for (item, mut glyph) in q_items.iter_mut() {
            if item.parent_list != list_entity {
                continue;
            }

            if item.index == list_state.selected_index {
                if list_state.has_focus {
                    glyph.bg = Some(Palette::DarkGray.into());
                } else {
                    glyph.bg = Some(Palette::DarkGray.into());
                }
            } else {
                glyph.bg = Some(Palette::Clear.into());
            }
        }
    }
}

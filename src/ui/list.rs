use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;
use std::collections::HashSet;

use crate::{
    common::Palette,
    engine::{AudioKey, AudioRegistry, KeyInput, Mouse},
    rendering::{Glyph, Layer, Position, Text, Visibility},
    ui::{Activatable, ActivatableBuilder, FocusType, Interactable, Interaction, UiFocus},
};

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
    pub audio_key: Option<AudioKey>,
}

impl ListItemData {
    pub fn new(label: &str, callback: SystemId) -> Self {
        Self {
            label: label.to_owned(),
            callback,
            hotkey: None,
            context_data: None,
            audio_key: None,
        }
    }

    pub fn with_hotkey(mut self, hotkey: KeyCode) -> Self {
        self.hotkey = Some(hotkey);
        self
    }

    pub fn with_context(mut self, context_data: u64) -> Self {
        self.context_data = Some(context_data);
        self
    }

    pub fn with_audio(mut self, audio_key: AudioKey) -> Self {
        self.audio_key = Some(audio_key);
        self
    }

    // Helper to convert to Activatable::ListItem
    pub fn to_activatable(
        &self,
        index: usize,
        parent_list: Entity,
        parent_focus_order: Option<i32>,
    ) -> Activatable {
        let mut builder = ActivatableBuilder::new(&self.label, self.callback);

        if let Some(hotkey) = self.hotkey {
            builder = builder.with_hotkey(hotkey);
        }

        if let Some(audio_key) = self.audio_key {
            builder = builder.with_audio(audio_key);
        }

        // Add focus order if parent has one, with index offset for sub-ordering
        if let Some(focus_order) = parent_focus_order {
            builder = builder.with_focus_order(focus_order + index as i32);
        }

        if let Some(context_data) = self.context_data {
            builder.as_list_item_with_context(index, parent_list, context_data)
        } else {
            builder.as_list_item(index, parent_list)
        }
    }
}

#[derive(Component)]
pub struct List {
    pub items: Vec<ListItemData>,
    pub width: f32,
    pub focus_order: Option<i32>,
}

#[derive(Component)]
pub struct ListState {
    pub selected_index: usize,
}

impl List {
    pub fn new(items: Vec<ListItemData>) -> Self {
        Self {
            items,
            width: 16.,
            focus_order: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_focus_order(mut self, focus_order: i32) -> Self {
        self.focus_order = Some(focus_order);
        self
    }
}

impl ListState {
    pub fn new() -> Self {
        Self { selected_index: 0 }
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

#[derive(Component, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Single,   // Only one item can be selected at a time
    Multiple, // Multiple items can be selected
}

#[derive(Component)]
pub struct SelectableList {
    pub selection_mode: SelectionMode,
    pub on_selection_change: Option<SystemId>,
}

#[derive(Component)]
pub struct SelectableListState {
    pub selected_indices: HashSet<usize>,
}

#[derive(Component)]
pub struct ListItemSelected;

pub fn setup_lists(
    mut cmds: Commands,
    mut q_lists: Query<
        (
            Entity,
            &List,
            &mut ListState,
            Option<&SelectableList>,
            Option<&SelectableListState>,
            Option<&Children>,
        ),
        Changed<List>,
    >,
    mut q_cursors: Query<&mut ListCursor>,
    mut q_items: ParamSet<(
        Query<
            (
                &mut ListItemBg,
                &mut Position,
                &mut Glyph,
                &mut Interactable,
            ),
            Without<Text>,
        >,
        Query<(&mut ListItem, &mut Text, &mut Position), With<Text>>,
        Query<&Position>,
    )>,
) {
    for (
        list_entity,
        list,
        mut list_state,
        selectable_opt,
        selectable_state_opt,
        existing_children,
    ) in q_lists.iter_mut()
    {
        // Get and copy the position data first
        let list_pos = {
            let q_pos = q_items.p2();
            let Ok(pos) = q_pos.get(list_entity) else {
                continue;
            };
            pos.clone() // Clone the position so we don't hold a borrow
        };

        // Fix cursor bounds
        if list_state.selected_index >= list.items.len() && !list.items.is_empty() {
            list_state.selected_index = list.items.len() - 1;
        } else if list.items.is_empty() {
            list_state.selected_index = 0;
        }

        let mut existing_cursors = Vec::new();
        let mut existing_bg_items = Vec::new();
        let mut existing_text_items = Vec::new();

        // Collect existing children by type
        if let Some(children) = existing_children {
            for child in children.iter() {
                if q_cursors.get_mut(child).is_ok() {
                    existing_cursors.push(child);
                } else if q_items.p0().get_mut(child).is_ok() {
                    existing_bg_items.push(child);
                } else if q_items.p1().get_mut(child).is_ok() {
                    existing_text_items.push(child);
                }
            }
        }

        // Ensure we have exactly one cursor
        if existing_cursors.is_empty() {
            cmds.spawn((
                Text::new("→").fg1(Palette::Yellow).layer(Layer::Ui),
                Position::new_f32(0., 0., 0.),
                ListCursor {
                    parent_list: list_entity,
                },
                Visibility::Visible,
                ChildOf(list_entity),
            ));
        }

        for (i, item_data) in list.items.iter().enumerate() {
            let item_spacing = 0.5;
            let item_y = i as f32 * item_spacing;

            if let Some(&bg_entity) = existing_bg_items.get(i) {
                if let Ok((mut bg_item, mut pos, mut glyph, mut interactable)) =
                    q_items.p0().get_mut(bg_entity)
                {
                    bg_item.index = i;
                    pos.x = list_pos.x + 1.0;
                    pos.y = list_pos.y + item_y;
                    pos.z = list_pos.z;

                    // Only update glyph properties if they changed to avoid triggering change detection
                    if glyph.idx != 6 {
                        glyph.idx = 6;
                    }
                    if glyph.scale != (list.width, item_spacing) {
                        glyph.scale = (list.width, item_spacing);
                    }
                    if glyph.layer_id != Layer::UiPanels {
                        glyph.layer_id = Layer::UiPanels;
                    }

                    *interactable = Interactable::new(14., item_spacing);
                }
            } else {
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
            }

            // Update or create text item
            if let Some(&text_entity) = existing_text_items.get(i) {
                if let Ok((mut text_item, mut text, mut pos)) = q_items.p1().get_mut(text_entity) {
                    // Update existing text item
                    text_item.index = i;
                    text.value = item_data.label.clone();
                    pos.x = list_pos.x + 1.0;
                    pos.y = list_pos.y + item_y;
                    pos.z = list_pos.z;
                }
                // Add/update Activatable and Interaction components for existing items
                let activatable = item_data.to_activatable(i, list_entity, list.focus_order);
                cmds.entity(text_entity).insert((
                    activatable,
                    Interaction::None,
                    Interactable::new(list.width, 0.5),
                ));
            } else {
                // Create new text item with Activatable component
                let activatable = item_data.to_activatable(i, list_entity, list.focus_order);
                cmds.spawn((
                    Text::new(&item_data.label).layer(Layer::Ui),
                    Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                    ListItem {
                        index: i,
                        parent_list: list_entity,
                    },
                    activatable,
                    Interaction::None,
                    Interactable::new(list.width, 0.5),
                    ChildOf(list_entity),
                ));
            }
        }

        // Despawn excess items if list got shorter
        for bg_entity in existing_bg_items.iter().skip(list.items.len()) {
            cmds.entity(*bg_entity).despawn();
        }
        for text_entity in existing_text_items.iter().skip(list.items.len()) {
            cmds.entity(*text_entity).despawn();
        }

        // Handle selection state for selectable lists
        if let (Some(_selectable), Some(selectable_state)) = (selectable_opt, selectable_state_opt)
        {
            // Clean up invalid selections (indices that are out of bounds)
            let valid_indices: HashSet<usize> = selectable_state
                .selected_indices
                .iter()
                .filter(|&&index| index < list.items.len())
                .copied()
                .collect();

            // Update selection state if it changed
            if valid_indices != selectable_state.selected_indices
                && let Ok(mut entity_cmds) = cmds.get_entity(list_entity) {
                    entity_cmds.try_insert(SelectableListState {
                        selected_indices: valid_indices.clone(),
                    });
                }

            // Apply ListItemSelected markers to text items based on selection state
            for (i, &text_entity) in existing_text_items
                .iter()
                .take(list.items.len())
                .enumerate()
            {
                if valid_indices.contains(&i) {
                    if let Ok(mut entity_cmds) = cmds.get_entity(text_entity) {
                        entity_cmds.try_insert(ListItemSelected);
                    }
                } else if let Ok(mut entity_cmds) = cmds.get_entity(text_entity) {
                    entity_cmds.remove::<ListItemSelected>();
                }
            }
        }
    }
}

pub fn update_list_context(
    mut list_context: ResMut<ListContext>,
    ui_focus: Res<UiFocus>,
    q_lists: Query<(&List, &ListState)>,
    q_list_items: Query<&ListItem>,
) {
    let Some(focused_element) = ui_focus.focused_element else {
        return;
    };

    // Check if the focused element is a list item
    if let Ok(list_item) = q_list_items.get(focused_element)
        && let Ok((list, list_state)) = q_lists.get(list_item.parent_list)
        && list_item.index < list.items.len()
    {
        let selected_item = &list.items[list_item.index];
        list_context.activated_item_index = list_item.index;
        list_context.activated_list = list_item.parent_list;
        list_context.context_data = selected_item.context_data;
    }
}

pub fn list_cursor_visibility(
    q_lists: Query<(Entity, &List, &ListState, &Position, &Children)>,
    mut q_cursors: Query<(&ListCursor, &mut Position, &mut Visibility), Without<List>>,
    q_list_items: Query<&ListItem>,
    ui_focus: Res<UiFocus>,
) {
    for (list_entity, _list, list_state, list_pos, children) in q_lists.iter() {
        for child in children.iter() {
            if let Ok((cursor, mut cursor_pos, mut cursor_vis)) = q_cursors.get_mut(child)
                && cursor.parent_list == list_entity
            {
                // Check if any item in this list has focus and get its index
                if let Some(focused_entity) = ui_focus.focused_element {
                    if let Ok(focused_list_item) = q_list_items.get(focused_entity) {
                        if focused_list_item.parent_list == list_entity {
                            // Show cursor and position it at the focused item
                            *cursor_vis = Visibility::Visible;
                            cursor_pos.x = list_pos.x;
                            cursor_pos.y = list_pos.y + (focused_list_item.index as f32 * 0.5);
                            cursor_pos.z = list_pos.z;
                        } else {
                            // Hide cursor if no item in this list is focused
                            *cursor_vis = Visibility::Hidden;
                        }
                    } else {
                        // Hide cursor if focused entity is not a list item
                        *cursor_vis = Visibility::Hidden;
                    }
                } else {
                    // Hide cursor if nothing is focused
                    *cursor_vis = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn selectable_list_interaction(
    mut cmds: Commands,
    mut q_selectable_lists: Query<(Entity, &SelectableList, &mut SelectableListState)>,
    q_list_items: Query<
        (Entity, &ListItem, &Interaction, Option<&ListItemSelected>),
        Changed<Interaction>,
    >,
    q_all_list_items: Query<(Entity, &ListItem)>,
    q_focused_items: Query<&ListItem>,
    keys: Res<KeyInput>,
    ui_focus: Res<UiFocus>,
    mut mouse: ResMut<Mouse>,
    audio: Res<AudioRegistry>,
) {
    // Handle Enter key for focused item
    if keys.is_pressed(KeyCode::Enter)
        && let Some(focused_entity) = ui_focus.focused_element
            && ui_focus.focus_type == FocusType::Keyboard
                && let Ok(list_item) = q_focused_items.get(focused_entity)
            {
                // Find parent selectable list
                for (list_entity, selectable_list, mut state) in q_selectable_lists.iter_mut() {
                    if list_item.parent_list == list_entity {
                        toggle_selection(
                            &mut cmds,
                            focused_entity,
                            list_item.index,
                            list_entity,
                            selectable_list,
                            &mut state,
                            &q_all_list_items,
                            &audio,
                        );
                        break;
                    }
                }
            }

    // Handle mouse clicks (Interaction::Released)
    for (item_entity, list_item, interaction, _selected) in q_list_items.iter() {
        if *interaction == Interaction::Released {
            // Find parent selectable list
            for (list_entity, selectable_list, mut state) in q_selectable_lists.iter_mut() {
                if list_item.parent_list == list_entity {
                    toggle_selection(
                        &mut cmds,
                        item_entity,
                        list_item.index,
                        list_entity,
                        selectable_list,
                        &mut state,
                        &q_all_list_items,
                        &audio,
                    );
                    // Mark the mouse click as captured to prevent other systems from handling it
                    mouse.is_captured = true;
                    break;
                }
            }
        }
    }
}

fn toggle_selection(
    cmds: &mut Commands,
    item_entity: Entity,
    item_index: usize,
    list_entity: Entity,
    selectable_list: &SelectableList,
    state: &mut SelectableListState,
    q_all_list_items: &Query<(Entity, &ListItem)>,
    audio: &AudioRegistry,
) {
    let was_selected = state.selected_indices.contains(&item_index);

    // Play selection sound
    audio.play(AudioKey::Button1, 0.7);

    match selectable_list.selection_mode {
        SelectionMode::Single => {
            if was_selected {
                // Deselect the item
                state.selected_indices.clear();
                if let Ok(mut entity_cmds) = cmds.get_entity(item_entity) {
                    entity_cmds.remove::<ListItemSelected>();
                }
            } else {
                // Clear all previous selections for this list visually
                for (entity, list_item) in q_all_list_items.iter() {
                    if list_item.parent_list == list_entity
                        && let Ok(mut entity_cmds) = cmds.get_entity(entity) {
                            entity_cmds.remove::<ListItemSelected>();
                        }
                }

                // Set new selection
                state.selected_indices.clear();
                state.selected_indices.insert(item_index);
                if let Ok(mut entity_cmds) = cmds.get_entity(item_entity) {
                    entity_cmds.try_insert(ListItemSelected);
                }
            }
        }
        SelectionMode::Multiple => {
            if was_selected {
                // Remove from selection
                state.selected_indices.remove(&item_index);
                if let Ok(mut entity_cmds) = cmds.get_entity(item_entity) {
                    entity_cmds.remove::<ListItemSelected>();
                }
            } else {
                // Add to selection
                state.selected_indices.insert(item_index);
                if let Ok(mut entity_cmds) = cmds.get_entity(item_entity) {
                    entity_cmds.try_insert(ListItemSelected);
                }
            }
        }
    }

    // Trigger callback if configured
    if let Some(callback) = selectable_list.on_selection_change {
        cmds.run_system(callback);
    }
}

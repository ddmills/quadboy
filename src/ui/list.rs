use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;
use std::collections::HashSet;

use crate::{
    common::Palette,
    engine::{Audio, AudioKey, KeyInput, Mouse},
    rendering::{Glyph, Layer, Position, Text, Visibility},
    ui::{Activatable, ActivatableBuilder, FocusType, Interactable, Interaction, UiFocus},
};
use bevy_ecs::prelude::ChildOf;

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
    pub selected_index: usize,
    pub height: Option<usize>,
    pub scroll_offset: usize,
}

impl List {
    pub fn new(items: Vec<ListItemData>) -> Self {
        Self {
            items,
            width: 16.,
            focus_order: None,
            selected_index: 0,
            height: None,
            scroll_offset: 0,
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

    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    pub fn ensure_item_visible(&mut self, item_index: usize) {
        if let Some(height) = self.height {
            if item_index < self.scroll_offset {
                self.scroll_offset = item_index;
            } else if item_index >= self.scroll_offset + height {
                self.scroll_offset = item_index.saturating_sub(height - 1);
            }
        }
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

#[derive(Component)]
pub struct ListScrollUpIndicator {
    pub parent_list: Entity,
}

#[derive(Component)]
pub struct ListScrollDownIndicator {
    pub parent_list: Entity,
}

pub fn setup_lists(
    mut cmds: Commands,
    mut q_lists: Query<
        (
            Entity,
            &mut List,
            Option<&SelectableList>,
            Option<&SelectableListState>,
            Option<&Children>,
        ),
        Changed<List>,
    >,
    mut q_cursors: Query<&mut ListCursor>,
    mut q_scroll_up: Query<&mut ListScrollUpIndicator>,
    mut q_scroll_down: Query<&mut ListScrollDownIndicator>,
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
    for (list_entity, mut list, selectable_opt, selectable_state_opt, existing_children) in
        q_lists.iter_mut()
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
        if list.selected_index >= list.items.len() && !list.items.is_empty() {
            list.selected_index = list.items.len() - 1;
        } else if list.items.is_empty() {
            list.selected_index = 0;
        }

        // Clamp scroll offset to valid range
        if let Some(height) = list.height {
            let max_offset = list.items.len().saturating_sub(height);
            list.scroll_offset = list.scroll_offset.min(max_offset);
        } else {
            list.scroll_offset = 0;
        }

        // Calculate visible range
        let visible_start = list.scroll_offset;
        let visible_end = if let Some(height) = list.height {
            (list.scroll_offset + height).min(list.items.len())
        } else {
            list.items.len()
        };

        let mut existing_cursors = Vec::new();
        let mut existing_bg_items = Vec::new();
        let mut existing_text_items = Vec::new();
        let mut existing_scroll_up_indicators = Vec::new();
        let mut existing_scroll_down_indicators = Vec::new();

        // Collect existing children by type
        if let Some(children) = existing_children {
            for child in children.iter() {
                if q_cursors.get_mut(child).is_ok() {
                    existing_cursors.push(child);
                } else if q_scroll_up.get_mut(child).is_ok() {
                    existing_scroll_up_indicators.push(child);
                } else if q_scroll_down.get_mut(child).is_ok() {
                    existing_scroll_down_indicators.push(child);
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

        for (actual_index, item_data) in list.items.iter().enumerate() {
            let item_spacing = 0.5;
            let is_visible = if list.height.is_some() {
                actual_index >= visible_start && actual_index < visible_end
            } else {
                true // All items visible if no height constraint
            };
            let visual_index = if is_visible {
                actual_index - list.scroll_offset
            } else {
                0
            };
            let item_y = if is_visible {
                visual_index as f32 * item_spacing
            } else {
                -1000.0
            };

            if let Some(&bg_entity) = existing_bg_items.get(actual_index) {
                if let Ok((mut bg_item, mut pos, mut glyph, mut interactable)) =
                    q_items.p0().get_mut(bg_entity)
                {
                    bg_item.index = actual_index;
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

                // Set visibility for background item
                if !is_visible {
                    cmds.entity(bg_entity).insert(Visibility::Hidden);
                } else {
                    cmds.entity(bg_entity).insert(Visibility::Visible);
                }
            } else {
                cmds.spawn((
                    Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                    Glyph::idx(6)
                        .scale((list.width, item_spacing))
                        .layer(Layer::UiPanels),
                    ListItemBg {
                        index: actual_index,
                        parent_list: list_entity,
                    },
                    Interaction::None,
                    Interactable::new(14., item_spacing),
                    if is_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ChildOf(list_entity),
                ));
            }

            // Update or create text item
            if let Some(&text_entity) = existing_text_items.get(actual_index) {
                if let Ok((mut text_item, mut text, mut pos)) = q_items.p1().get_mut(text_entity) {
                    // Update existing text item
                    text_item.index = actual_index;
                    text.value = item_data.label.clone();
                    pos.x = list_pos.x + 1.0;
                    pos.y = list_pos.y + item_y;
                    pos.z = list_pos.z;
                }
                // Update Activatable for existing items (contains dynamic data that may change)
                let activatable =
                    item_data.to_activatable(actual_index, list_entity, list.focus_order);

                // For existing entities, only update the Activatable component and visibility
                // Avoid reinserting Interaction and Interactable to prevent breaking interaction detection
                cmds.entity(text_entity).insert((
                    activatable,
                    if is_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                ));
            } else {
                // Create new text item with Activatable component
                let activatable =
                    item_data.to_activatable(actual_index, list_entity, list.focus_order);
                cmds.spawn((
                    Text::new(&item_data.label).layer(Layer::Ui),
                    Position::new_f32(list_pos.x + 1.0, list_pos.y + item_y, list_pos.z),
                    ListItem {
                        index: actual_index,
                        parent_list: list_entity,
                    },
                    activatable,
                    Interaction::None,
                    Interactable::new(list.width, 0.5),
                    if is_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ChildOf(list_entity),
                ));
            }
        }

        // Clean up excess items when list shrinks
        let current_item_count = list.items.len();

        // Despawn excess text items
        for i in current_item_count..existing_text_items.len() {
            if let Some(&entity) = existing_text_items.get(i) {
                cmds.entity(entity).despawn();
            }
        }

        // Despawn excess background items
        for i in current_item_count..existing_bg_items.len() {
            if let Some(&entity) = existing_bg_items.get(i) {
                cmds.entity(entity).despawn();
            }
        }

        // Handle scroll indicators
        if let Some(height) = list.height {
            let can_scroll_up = list.scroll_offset > 0;
            let can_scroll_down = list.scroll_offset + height < list.items.len();

            // Up indicator
            if can_scroll_up {
                if existing_scroll_up_indicators.is_empty() {
                    cmds.spawn((
                        Text::new("▲").fg1(Palette::Yellow).layer(Layer::Ui),
                        Position::new_f32(list_pos.x, list_pos.y - 0.5, list_pos.z),
                        ListScrollUpIndicator {
                            parent_list: list_entity,
                        },
                        ChildOf(list_entity),
                    ));
                } else {
                    // Update existing indicator position (if needed)
                    for &up_indicator in existing_scroll_up_indicators.iter() {
                        cmds.entity(up_indicator).insert(Position::new_f32(
                            list_pos.x,
                            list_pos.y - 0.5,
                            list_pos.z,
                        ));
                    }
                }
            } else {
                // Remove up indicator if exists
                for up_indicator in existing_scroll_up_indicators.iter() {
                    cmds.entity(*up_indicator).despawn();
                }
            }

            // Down indicator
            if can_scroll_down {
                let visible_count = (visible_end - visible_start).min(height);
                let down_y = list_pos.y + (visible_count as f32 * 0.5);

                if existing_scroll_down_indicators.is_empty() {
                    cmds.spawn((
                        Text::new("▼").fg1(Palette::Yellow).layer(Layer::Ui),
                        Position::new_f32(list_pos.x, down_y, list_pos.z),
                        ListScrollDownIndicator {
                            parent_list: list_entity,
                        },
                        ChildOf(list_entity),
                    ));
                } else {
                    // Update existing indicator position (if needed)
                    for &down_indicator in existing_scroll_down_indicators.iter() {
                        cmds.entity(down_indicator)
                            .insert(Position::new_f32(list_pos.x, down_y, list_pos.z));
                    }
                }
            } else {
                // Remove down indicator if exists
                for down_indicator in existing_scroll_down_indicators.iter() {
                    cmds.entity(*down_indicator).despawn();
                }
            }
        } else {
            // Remove all scroll indicators for non-scrollable lists
            for up_indicator in existing_scroll_up_indicators.iter() {
                cmds.entity(*up_indicator).despawn();
            }
            for down_indicator in existing_scroll_down_indicators.iter() {
                cmds.entity(*down_indicator).despawn();
            }
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
                && let Ok(mut entity_cmds) = cmds.get_entity(list_entity)
            {
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
    q_lists: Query<&List>,
    q_list_items: Query<&ListItem>,
) {
    let Some(focused_element) = ui_focus.focused_element else {
        return;
    };

    // Check if the focused element is a list item
    if let Ok(list_item) = q_list_items.get(focused_element)
        && let Ok(list) = q_lists.get(list_item.parent_list)
        && list_item.index < list.items.len()
    {
        let selected_item = &list.items[list_item.index];
        list_context.activated_item_index = list_item.index;
        list_context.activated_list = list_item.parent_list;
        list_context.context_data = selected_item.context_data;
    }
}

pub fn list_cursor_visibility(
    q_lists: Query<(Entity, &List, &Position, &Children)>,
    mut q_cursors: Query<(&ListCursor, &mut Position, &mut Visibility), Without<List>>,
    q_list_items: Query<&ListItem>,
    ui_focus: Res<UiFocus>,
) {
    for (list_entity, list, list_pos, children) in q_lists.iter() {
        for child in children.iter() {
            if let Ok((cursor, mut cursor_pos, mut cursor_vis)) = q_cursors.get_mut(child)
                && cursor.parent_list == list_entity
            {
                // Check if any item in this list has focus and get its index
                if let Some(focused_entity) = ui_focus.focused_element {
                    if let Ok(focused_list_item) = q_list_items.get(focused_entity) {
                        if focused_list_item.parent_list == list_entity {
                            // Check if focused item is visible when list has height constraint
                            if let Some(height) = list.height {
                                let item_visible = focused_list_item.index >= list.scroll_offset
                                    && focused_list_item.index < list.scroll_offset + height;

                                if !item_visible {
                                    *cursor_vis = Visibility::Hidden;
                                    continue;
                                }
                            }

                            // Calculate visual position based on scroll offset
                            let visual_index =
                                focused_list_item.index.saturating_sub(list.scroll_offset);
                            *cursor_vis = Visibility::Visible;
                            cursor_pos.x = list_pos.x;
                            cursor_pos.y = list_pos.y + (visual_index as f32 * 0.5);
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
    audio: Res<Audio>,
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
    audio: &Audio,
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
                        && let Ok(mut entity_cmds) = cmds.get_entity(entity)
                    {
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

pub fn list_mouse_wheel_scroll(mut q_lists: Query<(&mut List, &Position)>, mouse: Res<Mouse>) {
    if mouse.wheel_delta.1.abs() < 0.01 {
        return; // No wheel movement
    }

    for (mut list, pos) in q_lists.iter_mut() {
        let Some(height) = list.height else {
            continue; // List isn't scrollable
        };

        // Check if mouse is over this list
        let visible_count = height.min(list.items.len());
        let list_height = visible_count as f32 * 0.5;
        let mouse_over = mouse.ui.0 >= pos.x
            && mouse.ui.0 <= pos.x + list.width
            && mouse.ui.1 >= pos.y
            && mouse.ui.1 <= pos.y + list_height;

        if !mouse_over {
            continue;
        }

        // Scroll by 3 items per wheel tick
        let scroll_speed = 3;
        let max_offset = list.items.len().saturating_sub(height);

        if mouse.wheel_delta.1 > 0.0 {
            // Scroll up (decrease offset)
            list.scroll_offset = list.scroll_offset.saturating_sub(scroll_speed);
        } else {
            // Scroll down (increase offset)
            list.scroll_offset = (list.scroll_offset + scroll_speed).min(max_offset);
        }
    }
}

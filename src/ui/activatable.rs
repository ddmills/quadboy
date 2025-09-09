use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    engine::{Audio, AudioKey, KeyInput, Mouse, Time},
    rendering::{Glyph, Layer, Text},
    ui::{
        DialogContent, DialogState, FocusType, Interaction, ListItem, ListItemSelected,
        SelectableList, UiFocus,
    },
};

#[derive(Component)]
pub struct HotkeyPressed {
    pub remaining_time: f32,
}

impl HotkeyPressed {
    pub fn new() -> Self {
        Self {
            remaining_time: 0.1, // Show pressed state for 100ms
        }
    }
}

/// Unified enum for all activatable UI elements
#[derive(Component, Clone)]
pub enum Activatable {
    Button {
        label: String,
        callback: SystemId,
        hotkeys: Vec<KeyCode>,
        audio_key: Option<AudioKey>,
        layer: Layer,
        hover_color: Option<u32>,
        pressed_color: Option<u32>,
        normal_color: Option<u32>,
        focus_order: Option<i32>,
    },
    ListItem {
        label: String,
        callback: SystemId,
        hotkeys: Vec<KeyCode>,
        audio_key: Option<AudioKey>,
        context_data: Option<u64>,
        index: usize,
        parent_list: Entity,
        hover_color: Option<u32>,
        pressed_color: Option<u32>,
        normal_color: Option<u32>,
        focus_order: Option<i32>,
    },
}

impl Activatable {
    /// Get the label text for this element
    pub fn label(&self) -> &str {
        match self {
            Self::Button { label, .. } | Self::ListItem { label, .. } => label,
        }
    }

    /// Get the callback SystemId for this element
    pub fn callback(&self) -> SystemId {
        match self {
            Self::Button { callback, .. } | Self::ListItem { callback, .. } => *callback,
        }
    }

    /// Get the hotkeys for this element
    pub fn hotkeys(&self) -> &[KeyCode] {
        match self {
            Self::Button { hotkeys, .. } | Self::ListItem { hotkeys, .. } => hotkeys,
        }
    }

    /// Get the audio key for this element
    pub fn audio_key(&self) -> Option<AudioKey> {
        match self {
            Self::Button { audio_key, .. } | Self::ListItem { audio_key, .. } => *audio_key,
        }
    }

    /// Get the hover color for this element
    pub fn hover_color(&self) -> Option<u32> {
        match self {
            Self::Button { hover_color, .. } | Self::ListItem { hover_color, .. } => *hover_color,
        }
    }

    /// Get the pressed color for this element
    pub fn pressed_color(&self) -> Option<u32> {
        match self {
            Self::Button { pressed_color, .. } | Self::ListItem { pressed_color, .. } => {
                *pressed_color
            }
        }
    }

    /// Get the normal color for this element
    pub fn normal_color(&self) -> Option<u32> {
        match self {
            Self::Button { normal_color, .. } | Self::ListItem { normal_color, .. } => {
                *normal_color
            }
        }
    }

    /// Check if any of this element's hotkeys are currently pressed
    pub fn is_hotkey_pressed(&self, keys: &KeyInput) -> bool {
        self.hotkeys().iter().any(|key| keys.is_pressed(*key))
    }

    /// Perform activation: play audio and run callback
    pub fn activate(&self, cmds: &mut Commands, audio: &Audio) {
        let audio_key = self.audio_key().unwrap_or(AudioKey::Button1);
        audio.play(audio_key, 0.7);
        cmds.run_system(self.callback());
    }

    /// Get the focus order for this element
    pub fn focus_order(&self) -> Option<i32> {
        match self {
            Self::Button { focus_order, .. } | Self::ListItem { focus_order, .. } => *focus_order,
        }
    }

    /// Get the context data for list items
    pub fn context_data(&self) -> Option<u64> {
        match self {
            Self::ListItem { context_data, .. } => *context_data,
            _ => None,
        }
    }

    /// Get button-specific data if this is a button
    pub fn as_button(&self) -> Option<(&str, Layer)> {
        match self {
            Self::Button { label, layer, .. } => Some((label, *layer)),
            _ => None,
        }
    }

    /// Get list item-specific data if this is a list item
    pub fn as_list_item(&self) -> Option<(&str, usize, Entity, Option<u64>)> {
        match self {
            Self::ListItem {
                label,
                index,
                parent_list,
                context_data,
                ..
            } => Some((label, *index, *parent_list, *context_data)),
            _ => None,
        }
    }

    /// Generate a display string for the hotkeys (for UI display)
    pub fn hotkey_display(&self) -> String {
        match self.hotkeys() {
            [] => String::new(),
            [single] => format!("{:?}", single),
            multiple => {
                let keys: Vec<String> = multiple.iter().map(|k| format!("{:?}", k)).collect();
                keys.join(" or ")
            }
        }
    }

    /// Set the label for this element (mutable operation)
    pub fn set_label(&mut self, new_label: String) {
        match self {
            Self::Button { label, .. } | Self::ListItem { label, .. } => {
                *label = new_label;
            }
        }
    }
}

/// Builder for creating Activatable elements with a fluent interface
pub struct ActivatableBuilder {
    label: String,
    callback: SystemId,
    hotkeys: Vec<KeyCode>,
    audio_key: Option<AudioKey>,
    hover_color: Option<u32>,
    pressed_color: Option<u32>,
    normal_color: Option<u32>,
    focus_order: Option<i32>,
}

impl ActivatableBuilder {
    /// Create a new builder with required fields
    pub fn new(label: &str, callback: SystemId) -> Self {
        Self {
            label: label.to_string(),
            callback,
            hotkeys: Vec::new(),
            audio_key: None,
            hover_color: None,
            pressed_color: None,
            normal_color: None,
            focus_order: None,
        }
    }

    /// Add a single hotkey
    pub fn with_hotkey(mut self, key: KeyCode) -> Self {
        self.hotkeys.push(key);
        self
    }

    /// Add multiple hotkeys
    pub fn with_hotkeys(mut self, keys: &[KeyCode]) -> Self {
        self.hotkeys.extend_from_slice(keys);
        self
    }

    /// Set custom audio key
    pub fn with_audio(mut self, key: AudioKey) -> Self {
        self.audio_key = Some(key);
        self
    }

    /// Set custom hover color
    pub fn with_hover_color(mut self, color: u32) -> Self {
        self.hover_color = Some(color);
        self
    }

    /// Set custom pressed color
    pub fn with_pressed_color(mut self, color: u32) -> Self {
        self.pressed_color = Some(color);
        self
    }

    /// Set custom normal color
    pub fn with_normal_color(mut self, color: u32) -> Self {
        self.normal_color = Some(color);
        self
    }

    /// Set focus order for tab navigation
    /// Use spaced values like 1000, 2000, 3000 to allow for insertion
    pub fn with_focus_order(mut self, order: i32) -> Self {
        self.focus_order = Some(order);
        self
    }

    /// Build as a Button variant
    pub fn as_button(self, layer: Layer) -> Activatable {
        Activatable::Button {
            label: self.label,
            callback: self.callback,
            hotkeys: self.hotkeys,
            audio_key: self.audio_key,
            layer,
            hover_color: self.hover_color,
            pressed_color: self.pressed_color,
            normal_color: self.normal_color,
            focus_order: self.focus_order,
        }
    }

    /// Build as a ListItem variant
    pub fn as_list_item(self, index: usize, parent_list: Entity) -> Activatable {
        Activatable::ListItem {
            label: self.label,
            callback: self.callback,
            hotkeys: self.hotkeys,
            audio_key: self.audio_key,
            context_data: None,
            index,
            parent_list,
            hover_color: self.hover_color,
            pressed_color: self.pressed_color,
            normal_color: self.normal_color,
            focus_order: self.focus_order,
        }
    }

    /// Build as a ListItem variant with context data
    pub fn as_list_item_with_context(
        self,
        index: usize,
        parent_list: Entity,
        context: u64,
    ) -> Activatable {
        Activatable::ListItem {
            label: self.label,
            callback: self.callback,
            hotkeys: self.hotkeys,
            audio_key: self.audio_key,
            context_data: Some(context),
            index,
            parent_list,
            hover_color: self.hover_color,
            pressed_color: self.pressed_color,
            normal_color: self.normal_color,
            focus_order: self.focus_order,
        }
    }
}

/// Unified system for handling all keyboard activation (hotkeys and Enter key)
pub fn unified_keyboard_activation_system(
    mut cmds: Commands,
    q_activatable: Query<(Entity, &Activatable)>,
    q_dialog_content: Query<&DialogContent>,
    q_list_items: Query<&ListItem>,
    q_selectable_lists: Query<&SelectableList>,
    dialog_state: Res<DialogState>,
    ui_focus: Res<UiFocus>,
    keys: Res<KeyInput>,
    audio: Res<Audio>,
) {
    // Handle hotkey activation for all elements
    for (entity, activatable) in q_activatable.iter() {
        if dialog_state.is_open && q_dialog_content.get(entity).is_err() {
            continue;
        }

        if activatable.is_hotkey_pressed(&keys) {
            activatable.activate(&mut cmds, &audio);
            cmds.entity(entity).try_insert(HotkeyPressed::new());
            return;
        }
    }

    // Handle Enter key activation for focused elements
    if keys.is_pressed(KeyCode::Enter) {
        // Get the currently focused element
        let Some(focused_entity) = ui_focus.focused_element else {
            return;
        };

        // Only activate if it has keyboard focus (not mouse focus)
        if ui_focus.focus_type != FocusType::Keyboard {
            return;
        }

        if let Ok((entity, activatable)) = q_activatable.get(focused_entity) {
            if dialog_state.is_open && q_dialog_content.get(entity).is_err() {
                return;
            }

            // Check if this is a list item in a selectable list
            if let Ok(list_item) = q_list_items.get(entity)
                && q_selectable_lists.get(list_item.parent_list).is_ok()
            {
                // This is a selectable list item, don't activate - let selectable system handle it
                return;
            }

            activatable.activate(&mut cmds, &audio);

            // Add visual feedback for Enter key activation (same as hotkey)
            cmds.entity(entity).try_insert(HotkeyPressed::new());
        }
    }
}

pub fn unified_click_system(
    mut cmds: Commands,
    q_activatable: Query<(Entity, &Activatable, &Interaction), Changed<Interaction>>,
    q_list_items: Query<&ListItem>,
    q_selectable_lists: Query<&SelectableList>,
    audio: Res<Audio>,
    mut mouse: ResMut<Mouse>,
) {
    for (entity, activatable, interaction) in q_activatable.iter() {
        if matches!(interaction, Interaction::Released) {
            // Check if this is a list item in a selectable list
            if let Ok(list_item) = q_list_items.get(entity)
                && q_selectable_lists.get(list_item.parent_list).is_ok()
            {
                // This is a selectable list item, don't activate - let selectable system handle it
                continue;
            }

            mouse.is_captured = true;
            activatable.activate(&mut cmds, &audio);
        }
    }
}

/// System to manage HotkeyPressed timer and remove expired ones
pub fn hotkey_pressed_timer_system(
    mut cmds: Commands,
    time: Res<Time>,
    mut q_hotkey_pressed: Query<(Entity, &mut HotkeyPressed)>,
) {
    for (entity, mut hotkey_pressed) in q_hotkey_pressed.iter_mut() {
        hotkey_pressed.remaining_time -= time.dt;
        if hotkey_pressed.remaining_time <= 0.0 {
            cmds.entity(entity).remove::<HotkeyPressed>();
        }
    }
}

/// Unified styling system that applies consistent colors to all Activatable types
pub fn unified_style_system(
    mut q_text: Query<(
        &mut Text,
        &Activatable,
        Option<&Interaction>,
        Option<&HotkeyPressed>,
        Option<&ListItemSelected>,
    )>,
    mut q_glyph: Query<
        (
            &mut Glyph,
            &Activatable,
            Option<&Interaction>,
            Option<&HotkeyPressed>,
            Option<&ListItemSelected>,
        ),
        Without<Text>,
    >,
) {
    // Apply styling to Text components (buttons, dialog buttons)
    for (mut text, activatable, interaction_opt, hotkey_pressed_opt, selected_opt) in
        q_text.iter_mut()
    {
        let bg_color = determine_background_color(
            activatable,
            interaction_opt,
            hotkey_pressed_opt,
            selected_opt,
        );
        text.bg = Some(bg_color);
    }

    // Apply styling to Glyph components (list item backgrounds)
    for (mut glyph, activatable, interaction_opt, hotkey_pressed_opt, selected_opt) in
        q_glyph.iter_mut()
    {
        let bg_color = determine_background_color(
            activatable,
            interaction_opt,
            hotkey_pressed_opt,
            selected_opt,
        );
        glyph.bg = Some(bg_color);
    }
}

/// Determine the appropriate background color based on state and configuration
fn determine_background_color(
    activatable: &Activatable,
    interaction_opt: Option<&Interaction>,
    hotkey_pressed_opt: Option<&HotkeyPressed>,
    selected_opt: Option<&ListItemSelected>,
) -> u32 {
    // If selected, always show pressed state (highest priority)
    if selected_opt.is_some() {
        return activatable
            .pressed_color()
            .unwrap_or(Palette::DarkBlue.into());
    }

    // If hotkey was pressed, show pressed state
    if hotkey_pressed_opt.is_some() {
        return activatable
            .pressed_color()
            .unwrap_or(Palette::DarkBlue.into());
    }

    // Otherwise use interaction state
    match interaction_opt {
        Some(Interaction::Released) | Some(Interaction::Pressed) => activatable
            .pressed_color()
            .unwrap_or(Palette::DarkBlue.into()),
        Some(Interaction::Hovered) => activatable.hover_color().unwrap_or(Palette::Gray.into()),
        Some(Interaction::None) | None => {
            activatable.normal_color().unwrap_or(Palette::Black.into())
        }
    }
}

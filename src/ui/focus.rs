use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    domain::GameSettings,
    engine::{InputRate, KeyInput, Mouse, Time},
    ui::{DialogContent, DialogState, Interaction},
};

/// Unified resource for tracking UI focus state
#[derive(Resource, Default)]
pub struct UiFocus {
    pub focused_element: Option<Entity>,
    pub focus_type: FocusType,
}

impl UiFocus {
    pub fn set_focus(&mut self, element: Entity, focus_type: FocusType) {
        self.focused_element = Some(element);
        self.focus_type = focus_type;
    }

    pub fn clear_focus(&mut self) {
        self.focused_element = None;
        self.focus_type = FocusType::None;
    }

    pub fn has_focus(&self, element: Entity) -> bool {
        self.focused_element == Some(element)
    }

    pub fn has_keyboard_focus(&self, element: Entity) -> bool {
        self.has_focus(element) && self.focus_type == FocusType::Keyboard
    }
}

/// How an element received focus
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FocusType {
    #[default]
    None,
    Keyboard,
    Mouse,
}

/// Component marking elements that can receive focus
#[derive(Component, Default)]
pub struct Focusable {
    pub order: i32,
    pub can_tab_focus: bool,
}

impl Focusable {
    pub fn new() -> Self {
        Self {
            order: 0,
            can_tab_focus: true,
        }
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_tab_focus(mut self, can_tab: bool) -> Self {
        self.can_tab_focus = can_tab;
        self
    }

    pub fn no_tab(mut self) -> Self {
        self.can_tab_focus = false;
        self
    }
}

/// System that updates Interaction components based on focus state
pub fn sync_focus_to_interaction(
    mut q_interactions: Query<&mut Interaction>,
    ui_focus: Res<UiFocus>,
) {
    let Some(focused_entity) = ui_focus.focused_element else {
        return;
    };

    // Handle direct element focus (buttons, list items, etc.)
    if let Ok(mut interaction) = q_interactions.get_mut(focused_entity)
        && ui_focus.focus_type == FocusType::Keyboard
    {
        *interaction = Interaction::Hovered;
    }
}

/// System that handles Tab navigation between focusable elements
pub fn tab_navigation(
    mut ui_focus: ResMut<UiFocus>,
    q_focusable: Query<(Entity, &Focusable)>,
    q_dialog_content: Query<&DialogContent>,
    dialog_state: Res<DialogState>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
    settings: Res<GameSettings>,
) {
    let now = time.fixed_t;
    let rate = settings.input_delay;
    let delay = settings.input_initial_delay;

    // Clean up released keys first
    for key in keys.released.iter() {
        input_rate.keys.remove(key);
    }

    if !keys.is_down(KeyCode::Tab) {
        return;
    }

    if !input_rate.try_key(KeyCode::Tab, now, rate, delay) {
        return;
    }

    let reverse = keys.is_down(KeyCode::LeftShift) || keys.is_down(KeyCode::RightShift);

    // Get all focusable elements, filtering for dialog state
    let mut focusable: Vec<(Entity, i32)> = q_focusable
        .iter()
        .filter(|(entity, focusable)| {
            if !focusable.can_tab_focus {
                return false;
            }

            // If a dialog is open, only allow focusing dialog elements
            if dialog_state.is_open {
                q_dialog_content.get(*entity).is_ok()
            } else {
                q_dialog_content.get(*entity).is_err()
            }
        })
        .map(|(entity, focusable)| (entity, focusable.order))
        .collect();

    if focusable.is_empty() {
        return;
    }

    // Sort by tab order
    focusable.sort_by_key(|(_, order)| *order);

    let current_index = if let Some(focused) = ui_focus.focused_element {
        focusable.iter().position(|(entity, _)| *entity == focused)
    } else {
        None
    };

    let next_index = match current_index {
        Some(current) => {
            if reverse {
                if current == 0 {
                    focusable.len() - 1
                } else {
                    current - 1
                }
            } else {
                (current + 1) % focusable.len()
            }
        }
        None => 0,
    };

    let (next_entity, _) = focusable[next_index];
    ui_focus.set_focus(next_entity, FocusType::Keyboard);
}

/// System that updates focus when elements are hovered with mouse
pub fn update_focus_from_mouse(
    mut ui_focus: ResMut<UiFocus>,
    q_hovered: Query<Entity, (With<Focusable>, With<Interaction>)>,
    q_interactions: Query<&Interaction>,
    mouse: Res<Mouse>,
) {
    // Only update focus if mouse has moved
    if !mouse.has_moved {
        return;
    }

    for entity in q_hovered.iter() {
        if let Ok(interaction) = q_interactions.get(entity)
            && *interaction == Interaction::Hovered
        {
            ui_focus.set_focus(entity, FocusType::Mouse);
            break; // Only one element can be hovered at a time
        }
    }
}

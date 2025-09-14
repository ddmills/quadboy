use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{Equippable, Item, Label, MeleeWeapon, StackCount},
    engine::{AudioKey, StableIdRegistry},
    rendering::{Glyph, Layer, Position, text_content_length},
    ui::{
        ActivatableBuilder, Dialog, DialogContent, DialogIcon, DialogProperty, DialogText,
        DialogTextStyle,
    },
};

pub struct ItemDialogButton {
    pub label: String,
    pub callback: SystemId,
    pub hotkey: Option<KeyCode>,
    pub audio_key: Option<AudioKey>,
    pub focus_order: i32,
}

impl ItemDialogButton {
    pub fn new(label: &str, callback: SystemId) -> Self {
        Self {
            label: label.to_string(),
            callback,
            hotkey: None,
            audio_key: None,
            focus_order: 1000,
        }
    }

    pub fn with_hotkey(mut self, hotkey: KeyCode) -> Self {
        self.hotkey = Some(hotkey);
        self
    }

    pub fn with_focus_order(mut self, focus_order: i32) -> Self {
        self.focus_order = focus_order;
        self
    }
}

pub struct ItemDialogBuilder {
    item_entity: Entity,
    position: Position,
    width: f32,
    height: f32,
    cleanup_component: Option<()>,
    buttons: Vec<ItemDialogButton>,
    close_callback: Option<SystemId>,
}

impl ItemDialogBuilder {
    pub fn new(item_entity: Entity) -> Self {
        Self {
            item_entity,
            position: Position::new_f32(5.0, 3.0, 0.0),
            width: 20.0,
            height: 8.0,
            cleanup_component: None,
            buttons: Vec::new(),
            close_callback: None,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Position::new_f32(x, y, 0.0);
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn add_button(mut self, button: ItemDialogButton) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn with_close_callback(mut self, callback: SystemId) -> Self {
        self.close_callback = Some(callback);
        self
    }

    pub fn spawn(
        self,
        cmds: &mut Commands,
        item_entity: Entity,
        _id_registry: &StableIdRegistry,
        q_labels: &Query<&Label>,
        q_glyphs: &Query<&Glyph>,
        q_items: &Query<&Item>,
        q_equippable: &Query<&Equippable>,
        q_melee_weapons: &Query<&MeleeWeapon>,
        q_stack_counts: &Query<&StackCount>,
        cleanup_component: impl Bundle + Clone,
    ) -> Entity {
        let item_name = if let Ok(label) = q_labels.get(item_entity) {
            label.get().to_string()
        } else {
            "Unknown Item".to_string()
        };

        let dialog_entity = cmds
            .spawn((
                Dialog::new("", self.width, self.height),
                self.position.clone(),
                cleanup_component.clone(),
            ))
            .id();

        // Add centered icon
        if let Ok(glyph) = q_glyphs.get(item_entity) {
            cmds.spawn((
                DialogIcon {
                    glyph_idx: glyph.idx,
                    scale: 2.0,
                    fg1: glyph.fg1,
                    fg2: glyph.fg2,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 10,
                },
                Position::new_f32(
                    self.position.x + (self.width / 2.0) - 1.0,
                    self.position.y + 0.5,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        // Add centered item name
        cmds.spawn((
            DialogText {
                value: item_name.clone(),
                style: DialogTextStyle::Title,
            },
            DialogContent {
                parent_dialog: dialog_entity,
                order: 11,
            },
            Position::new_f32(
                self.position.x + (self.width / 2.0)
                    - (text_content_length(&item_name) as f32 * 0.25),
                self.position.y + 2.5,
                self.position.z,
            ),
            cleanup_component.clone(),
            ChildOf(dialog_entity),
        ));

        let mut content_y = 3.5;
        let mut order = 12;

        // Add item properties
        if let Ok(item) = q_items.get(item_entity) {
            cmds.spawn((
                DialogProperty {
                    label: "Weight".to_string(),
                    value: format!("{:.1} kg", item.weight),
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    self.position.x + 1.0,
                    self.position.y + content_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
            content_y += 0.5;
            order += 1;
        }

        if let Ok(stack_count) = q_stack_counts.get(item_entity) {
            cmds.spawn((
                DialogProperty {
                    label: "Quantity".to_string(),
                    value: format!("x{}", stack_count.count),
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    self.position.x + 1.0,
                    self.position.y + content_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
            content_y += 0.5;
            order += 1;
        }

        if let Ok(equippable) = q_equippable.get(item_entity) {
            let equipment_type = format!("{:?}", equippable.equipment_type);
            cmds.spawn((
                DialogProperty {
                    label: "Type".to_string(),
                    value: equipment_type,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    self.position.x + 1.0,
                    self.position.y + content_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
            content_y += 0.5;
            order += 1;

            let slots = equippable
                .slot_requirements
                .iter()
                .map(|slot| slot.display_name())
                .collect::<Vec<_>>()
                .join(", ");

            cmds.spawn((
                DialogProperty {
                    label: "Slots".to_string(),
                    value: slots,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    self.position.x + 1.0,
                    self.position.y + content_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
            content_y += 0.5;
            order += 1;
        }

        if let Ok(melee_weapon) = q_melee_weapons.get(item_entity) {
            cmds.spawn((
                DialogProperty {
                    label: "Damage".to_string(),
                    value: melee_weapon.damage_dice.clone(),
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    self.position.x + 1.0,
                    self.position.y + content_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        // Add buttons
        let button_y = self.position.y + self.height - 1.5;
        let mut button_x = self.position.x + 1.0;

        for button in self.buttons {
            let mut activatable_builder = ActivatableBuilder::new(&button.label, button.callback)
                .with_focus_order(button.focus_order);

            if let Some(hotkey) = button.hotkey {
                activatable_builder = activatable_builder.with_hotkey(hotkey);
            }

            if let Some(audio_key) = button.audio_key {
                activatable_builder = activatable_builder.with_audio(audio_key);
            }

            cmds.spawn((
                activatable_builder.as_button(Layer::DialogContent),
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 20,
                },
                Position::new_f32(button_x, button_y, self.position.z),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));

            button_x += button.label.len() as f32 + 2.0;
        }

        // Add close button
        if let Some(close_callback) = self.close_callback {
            cmds.spawn((
                ActivatableBuilder::new("[{Y|ESC}] Close", close_callback)
                    .with_audio(AudioKey::ButtonBack1)
                    .with_hotkey(KeyCode::Escape)
                    .with_focus_order(3000)
                    .as_button(Layer::DialogContent),
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 22,
                },
                Position::new_f32(
                    self.position.x + self.width - 6.5,
                    button_y,
                    self.position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        dialog_entity
    }
}

pub fn spawn_item_dialog(
    cmds: &mut Commands,
    item_id: u64,
    builder: ItemDialogBuilder,
    id_registry: &StableIdRegistry,
    q_labels: &Query<&Label>,
    q_glyphs: &Query<&Glyph>,
    q_items: &Query<&Item>,
    q_equippable: &Query<&Equippable>,
    q_melee_weapons: &Query<&MeleeWeapon>,
    q_stack_counts: &Query<&StackCount>,
    cleanup_component: impl Bundle + Clone,
) -> Option<Entity> {
    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return None;
    };

    Some(builder.spawn(
        cmds,
        item_entity,
        id_registry,
        q_labels,
        q_glyphs,
        q_items,
        q_equippable,
        q_melee_weapons,
        q_stack_counts,
        cleanup_component,
    ))
}

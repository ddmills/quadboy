use bevy_ecs::{
    prelude::*,
    system::{RunSystemOnce, SystemId},
};

use crate::{
    domain::{Description, FactionMember, Label, get_effective_relationship},
    engine::StableId,
    rendering::{Glyph, ScreenSize},
    states::CleanupStateExplore,
    ui::examine_dialog::ExamineDialogBuilder,
};

pub struct SpawnExamineDialogCommand {
    pub entity: Entity,
    pub player_entity: Entity,
    pub close_callback: SystemId,
}

impl SpawnExamineDialogCommand {
    pub fn new(entity: Entity, player_entity: Entity, close_callback: SystemId) -> Self {
        Self {
            entity,
            player_entity,
            close_callback,
        }
    }
}

impl Command for SpawnExamineDialogCommand {
    fn apply(self, world: &mut World) {
        let relationship_text = if let (Some(entity_faction), Some(player_faction)) = (
            world.get::<FactionMember>(self.entity),
            world.get::<FactionMember>(self.player_entity),
        ) {
            if entity_faction.faction_id != player_faction.faction_id {
                let relationship =
                    get_effective_relationship(self.player_entity, self.entity, world);
                Some(
                    crate::domain::systems::faction_system::format_relationship_display(
                        relationship,
                    ),
                )
            } else {
                None
            }
        } else {
            None
        };

        let entity = self.entity;
        let close_callback = self.close_callback;

        world
            .run_system_once(
                move |mut cmds: Commands,
                      q_labels: Query<&Label>,
                      q_descriptions: Query<&Description>,
                      q_glyphs: Query<&Glyph>,
                      q_stable_ids: Query<&StableId>,
                      screen: Res<ScreenSize>| {
                    ExamineDialogBuilder::new(entity, close_callback)
                        .with_relationship_text(relationship_text.clone())
                        .spawn(
                            &mut cmds,
                            &q_labels,
                            &q_descriptions,
                            &q_glyphs,
                            &q_stable_ids,
                            CleanupStateExplore,
                            &screen,
                        )
                },
            )
            .unwrap();
    }
}

use bevy_ecs::{
    prelude::*,
    system::{RunSystemOnce, SystemId},
};

use crate::{
    domain::{AiController, Energy, Label},
    engine::Clock,
    rendering::{Position, ScreenSize},
    states::CleanupStateExplore,
    ui::ai_debug_dialog::AiDebugDialogBuilder,
};

pub struct SpawnAiDebugDialogCommand {
    pub entity: Entity,
    pub close_callback: SystemId,
}

impl SpawnAiDebugDialogCommand {
    pub fn new(entity: Entity, close_callback: SystemId) -> Self {
        Self {
            entity,
            close_callback,
        }
    }
}

impl Command for SpawnAiDebugDialogCommand {
    fn apply(self, world: &mut World) {
        let entity = self.entity;
        let close_callback = self.close_callback;

        world
            .run_system_once(
                move |mut cmds: Commands,
                      q_labels: Query<&Label>,
                      q_ai_controllers: Query<&AiController>,
                      q_energy: Query<&Energy>,
                      q_positions: Query<&Position>,
                      clock: Res<Clock>,
                      screen: Res<ScreenSize>| {
                    AiDebugDialogBuilder::new(entity, close_callback).spawn(
                        &mut cmds,
                        &q_labels,
                        &q_ai_controllers,
                        &q_energy,
                        &q_positions,
                        &clock,
                        CleanupStateExplore,
                        &screen,
                    )
                },
            )
            .unwrap();
    }
}

pub fn spawn_ai_debug_dialog(world: &mut World, entity: Entity, close_callback: SystemId) {
    let cmd = SpawnAiDebugDialogCommand::new(entity, close_callback);
    cmd.apply(world);
}

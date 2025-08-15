use bevy_ecs::{
    event::{Event, EventReader},
    resource::Resource,
    system::ResMut,
};

use crate::engine::Plugin;

use super::ScheduleType;

pub struct ExitAppPlugin;

impl Plugin for ExitAppPlugin {
    fn build(&self, app: &mut super::App) {
        app.init_resource::<ExitApp>()
            .register_event::<ExitAppEvent>()
            .add_systems(ScheduleType::FrameFinal, on_exit_app);
    }
}

#[derive(Event)]
pub struct ExitAppEvent;

#[derive(Resource, Default)]
pub struct ExitApp(pub bool);

fn on_exit_app(e_exit_app: EventReader<ExitAppEvent>, mut exit_app: ResMut<ExitApp>) {
    if !e_exit_app.is_empty() {
        exit_app.0 = true;
    }
}

use bevy_ecs::{event::EventRegistry, prelude::*, system::ScheduleSystem};

use crate::{engine::ExitApp, tracy_span};

pub enum ScheduleType {
    PreUpdate,
    Update,
    PostUpdate,
    FrameFinal,
    StateTransition,
}

pub struct App {
    world: World,
    schedule_pre_update: Schedule,
    schedule_update: Schedule,
    schedule_post_update: Schedule,
    schedule_frame_final: Schedule,
    schedule_state_transition: Schedule,
}

pub trait Plugin {
    fn build(&self, app: &mut App);
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule_pre_update: Schedule::default(),
            schedule_update: Schedule::default(),
            schedule_post_update: Schedule::default(),
            schedule_frame_final: Schedule::default(),
            schedule_state_transition: Schedule::default(),
        }
    }

    pub fn run(&mut self) -> bool {
        {
            tracy_span!("schedule_pre_update");
            ("schedule_pre_update");
            self.schedule_pre_update.run(&mut self.world);
        }

        {
            tracy_span!("schedule_update");
            ("schedule_update");
            self.schedule_update.run(&mut self.world);
        }

        {
            tracy_span!("schedule_post_update");
            ("schedule_post_update");
            self.schedule_post_update.run(&mut self.world);
        }

        {
            tracy_span!("schedule_frame_final");
            ("schedule_frame_final");
            self.schedule_frame_final.run(&mut self.world);
        }

        {
            tracy_span!("schedule_state_transition");
            ("schedule_state_transition");
            self.schedule_state_transition.run(&mut self.world);
        }

        let exit = self
            .world
            .get_resource::<ExitApp>()
            .map(|x| x.0)
            .unwrap_or(false);

        !exit
    }

    pub fn register_event<T: Event>(&mut self) -> &mut Self {
        EventRegistry::register_event::<T>(&mut self.world);
        self
    }

    pub fn add_systems<M>(
        &mut self,
        schedule_type: ScheduleType,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let schedule = self.get_schedule_mut(schedule_type);
        schedule.add_systems(systems);
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, value: R) -> &mut Self {
        self.world.insert_resource(value);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn get_world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn get_schedule_mut(&mut self, schedule_type: ScheduleType) -> &mut Schedule {
        match schedule_type {
            ScheduleType::PreUpdate => &mut self.schedule_pre_update,
            ScheduleType::Update => &mut self.schedule_update,
            ScheduleType::PostUpdate => &mut self.schedule_post_update,
            ScheduleType::FrameFinal => &mut self.schedule_frame_final,
            ScheduleType::StateTransition => &mut self.schedule_state_transition,
        }
    }
}

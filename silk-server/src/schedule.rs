use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkServerSchedule;

pub fn run_silk_schedule(world: &mut World) {
    world.run_schedule(SilkServerSchedule);
}

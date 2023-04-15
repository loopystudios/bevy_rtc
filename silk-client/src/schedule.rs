use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkClientSchedule;

pub fn run_silk_schedule(world: &mut World) {
    world.run_schedule(SilkClientSchedule);
}

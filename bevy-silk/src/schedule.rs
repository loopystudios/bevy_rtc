use bevy::ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SilkSchedule;

use bevy::prelude::*;
use bevy_matchbox::{matchbox_socket::MultipleChannels, MatchboxSocket};

use crate::{
    events::SocketRecvEvent, schedule::SilkSchedule, sets::SilkSet,
    socket::common_socket_reader,
};

pub struct SilkCommonPlugin;

impl Plugin for SilkCommonPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the schedule for silk
        app.init_schedule(SilkSchedule);
        app.edit_schedule(SilkSchedule, |schedule| {
            schedule.configure_sets(SilkSet::sets());
        });

        app.add_event::<SocketRecvEvent>()
            .add_systems(SilkSchedule, trace_flush.before(SilkSet::Flush))
            .add_systems(
                SilkSchedule,
                trace_network_read
                    .after(SilkSet::Flush)
                    .before(SilkSet::NetworkRead),
            )
            .add_systems(
                SilkSchedule,
                // Read silk events always before servers, who hook into this
                // stage
                common_socket_reader
                    .after(trace_network_read)
                    .before(SilkSet::NetworkRead)
                    .run_if(
                        resource_exists::<MatchboxSocket<MultipleChannels>>(),
                    ),
            )
            .add_systems(
                SilkSchedule,
                trace_process
                    .after(SilkSet::NetworkRead)
                    .before(SilkSet::Process),
            )
            .add_systems(
                SilkSchedule,
                trace_silk_events
                    .after(SilkSet::Process)
                    .before(SilkSet::SilkEvents),
            )
            .add_systems(
                SilkSchedule,
                trace_pre_update
                    .after(SilkSet::SilkEvents)
                    .before(SilkSet::PreUpdate),
            )
            .add_systems(
                SilkSchedule,
                trace_update
                    .after(SilkSet::PreUpdate)
                    .before(SilkSet::Update),
            )
            .add_systems(
                SilkSchedule,
                trace_post_update
                    .after(SilkSet::Update)
                    .before(SilkSet::PostUpdate),
            )
            .add_systems(
                SilkSchedule,
                trace_network_write
                    .after(SilkSet::PostUpdate)
                    .before(SilkSet::NetworkWrite),
            );

        // add scheduler
        app.add_systems(FixedUpdate, |world: &mut World| {
            world.run_schedule(SilkSchedule);
        });
    }
}

fn trace_flush() {
    trace!("System start: {}", SilkSet::Flush);
}

fn trace_network_read() {
    trace!("System start: {}", SilkSet::NetworkRead);
}

fn trace_process() {
    trace!("System start: {}", SilkSet::Process);
}

fn trace_silk_events() {
    trace!("System start: {}", SilkSet::SilkEvents);
}

fn trace_pre_update() {
    trace!("System start: {}", SilkSet::PreUpdate);
}

fn trace_update() {
    trace!("System start: {}", SilkSet::Update);
}

fn trace_post_update() {
    trace!("System start: {}", SilkSet::PostUpdate);
}

fn trace_network_write() {
    trace!("System start: {}", SilkSet::NetworkWrite);
}

#![expect(unused)]

use bevy_ecs::{prelude::*, system::SystemParam};

use crate::path::flow::{DoorRegions, FlowField, RegionDoors};

#[derive(SystemParam)]
pub struct PathParam<'w, 's> {
    flow_fields: Query<'w, 's, &'static FlowField>,
    doors: Query<'w, 's, &'static DoorRegions>,
    regions: Query<'w, 's, &'static RegionDoors>,
}

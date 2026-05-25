#[cfg(test)]
mod tests;

use std::time::Duration;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;

use wdn_physics::collision::{ColliderDisabled, TileCollider};

use crate::WorldSystems;

pub struct DoorPlugin;

#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TileCollider, DoorDirection)]
pub struct Door {
    state: DoorState,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub enum DoorDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DoorState {
    #[default]
    Closed,
    Opening {
        position: f32,
    },
    Open {
        timer: Duration,
    },
    Closing {
        position: f32,
    },
}

pub fn update_doors(
    mut commands: Commands,
    mut doors: Query<(Entity, &mut Door)>,
    time: Res<Time>,
) {
    doors.iter_mut().for_each(|(id, mut door)| {
        if !matches!(door.state, DoorState::Closed) {
            door.tick(id, &time, &mut commands);
        }
    });
}

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_doors.in_set(WorldSystems::UpdateDoors));
    }
}

impl Door {
    const OPEN_SPEED: f32 = 1.0;
    const OPEN_DURATION: Duration = Duration::from_secs(3);

    pub fn is_open(&self) -> bool {
        match self.state {
            DoorState::Closed => false,
            DoorState::Opening { position } | DoorState::Closing { position } => position >= 0.5,
            DoorState::Open { .. } => true,
        }
    }

    pub fn position(&self) -> f32 {
        match self.state {
            DoorState::Closed => 0.0,
            DoorState::Opening { position } | DoorState::Closing { position } => position,
            DoorState::Open { .. } => 1.0,
        }
    }

    pub fn open(&mut self) {
        match self.state {
            DoorState::Closed => {
                self.state = DoorState::Opening { position: 0.0 };
            }
            DoorState::Closing { position } => {
                self.state = DoorState::Opening { position };
            }
            _ => {}
        }
    }

    pub fn close(&mut self) {
        match self.state {
            DoorState::Open { .. } => {
                self.state = DoorState::Closing { position: 1.0 };
            }
            DoorState::Opening { position } => {
                self.state = DoorState::Closing { position };
            }
            _ => {}
        }
    }

    pub fn toggle(&mut self) {
        match self.state {
            DoorState::Closed | DoorState::Closing { .. } => self.open(),
            DoorState::Opening { .. } | DoorState::Open { .. } => self.close(),
        }
    }

    pub fn tick(&mut self, id: Entity, time: &Time, commands: &mut Commands) {
        let was_open = self.is_open();

        match self.state {
            DoorState::Closed => {}
            DoorState::Opening { ref mut position } => {
                *position += Self::OPEN_SPEED * time.delta_secs();

                if *position >= 1.0 {
                    self.state = DoorState::Open {
                        timer: Duration::ZERO,
                    };
                }
            }
            DoorState::Open { ref mut timer } => {
                *timer += time.delta();

                if *timer >= Self::OPEN_DURATION {
                    self.state = DoorState::Closing { position: 1.0 };
                }
            }
            DoorState::Closing { ref mut position } => {
                *position -= Self::OPEN_SPEED * time.delta_secs();

                if *position <= 0.0 {
                    self.state = DoorState::Closed;
                }
            }
        }

        match (was_open, self.is_open()) {
            (false, true) => {
                commands.entity(id).insert(ColliderDisabled);
            }
            (true, false) => {
                commands.entity(id).remove::<ColliderDisabled>();
            }
            _ => {}
        }
    }
}

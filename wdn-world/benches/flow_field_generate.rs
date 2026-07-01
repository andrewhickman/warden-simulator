use bevy_app::{App, FixedUpdate, TaskPoolPlugin};
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wdn_physics::{
    layer::Layer,
    tile::{TilePlugin, material::TileMaterial, position::TilePosition, storage::TileStorageMut},
};
use wdn_world::{
    door::Door,
    path::{
        PathPlugin,
        flow::{CostField, FlowField, FlowPolicy},
        region::RegionTiles,
    },
};

const WIDTH: i32 = 256;
const HEIGHT: i32 = 256;

fn make_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TilePlugin, PathPlugin));
    let layer = app.world_mut().spawn(Layer::default()).id();
    (app, layer)
}

fn bench_flow_field_generate(c: &mut Criterion) {
    let (mut app, layer) = make_app();

    app.world_mut()
        .run_system_once(move |mut commands: Commands, mut storage: TileStorageMut| {
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    if y > HEIGHT / 4 && y < HEIGHT / 4 * 3 && x % 16 == 0 {
                        storage.set_material(TilePosition::new(layer, x, y), TileMaterial::WALL);
                    } else {
                        storage.set_material(TilePosition::new(layer, x, y), TileMaterial::EMPTY);
                    }
                }
            }

            let door = TilePosition::new(layer, WIDTH / 2, HEIGHT / 2);
            storage.set_material(door, TileMaterial::DOOR);
            commands.spawn((ChildOf(layer), Door::default(), door));
        })
        .expect("failed to seed benchmark tiles");

    app.world_mut().run_schedule(FixedUpdate);

    let mut regions_query = app.world_mut().query::<&RegionTiles>();
    let query = regions_query.query(app.world());
    let region_tiles = query.iter().find(|region| region.door_count() > 0).unwrap();

    let door = region_tiles.doors()[0];

    c.bench_function("CostField::generate", |b| {
        b.iter(|| {
            let mut cost_field = CostField::new(region_tiles.size());
            cost_field.generate::<FlowPolicy>(
                &FlowPolicy,
                region_tiles,
                door.index(),
                door.position(),
                door.adjacency(),
            );

            black_box(cost_field)
        });
    });

    let mut cost_field = CostField::new(region_tiles.size());
    cost_field.generate::<FlowPolicy>(
        &FlowPolicy,
        region_tiles,
        door.index(),
        door.position(),
        door.adjacency(),
    );

    c.bench_function("FlowField::populate", |b| {
        b.iter(|| {
            let mut flow_field = FlowField::from_cost_field(
                TilePosition::from((layer, door.position())),
                door.index(),
                door.adjacency(),
                cost_field.clone(),
            );

            flow_field.populate_flow(region_tiles);

            black_box(flow_field)
        });
    });
}

criterion_group!(benches, bench_flow_field_generate);
criterion_main!(benches);

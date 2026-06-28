use bevy_app::{App, TaskPoolPlugin, Update};
use bevy_ecs::{prelude::*, system::RunSystemOnce};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wdn_physics::{
    layer::Layer,
    tile::{
        TilePlugin,
        material::TileMaterial,
        position::TilePosition,
        storage::{TileMap, TileStorageMut},
    },
};
use wdn_world::path::section::{TileChunkSections, update_chunk_sections};

fn make_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), TilePlugin));
    app.register_required_components::<wdn_physics::tile::storage::TileChunk, TileChunkSections>();
    app.init_resource::<wdn_world::path::section::TileChunkSectionChanges>();
    app.add_systems(Update, update_chunk_sections);

    let layer = app.world_mut().spawn(Layer::default()).id();
    (app, layer)
}

fn run_update_chunk_sections(app: &mut App) {
    app.world_mut().run_schedule(Update);
}

fn set_material(app: &mut App, position: TilePosition, material: TileMaterial) {
    app.world_mut()
        .run_system_once(move |mut storage: TileStorageMut| {
            storage.set_material(position, material);
        })
        .expect("failed to set benchmark tile material");
}

fn bench_update_chunk_sections_after_tile_modify(c: &mut Criterion) {
    let (mut app, layer) = make_app();
    let position = TilePosition::new(layer, 10, 10);

    set_material(&mut app, position, TileMaterial::Wall);
    run_update_chunk_sections(&mut app);

    set_material(&mut app, position, TileMaterial::Empty);
    run_update_chunk_sections(&mut app);

    let mut place_wall = true;
    c.bench_function("update_chunk_sections_after_tile_modify", |b| {
        b.iter(|| {
            let material = if place_wall {
                TileMaterial::Wall
            } else {
                TileMaterial::Empty
            };
            place_wall = !place_wall;

            set_material(&mut app, position, material);
            run_update_chunk_sections(&mut app);
        });
    });
}

fn bench_update_chunk_sections_after_tile_unchanged(c: &mut Criterion) {
    let (mut app, layer) = make_app();
    let position = TilePosition::new(layer, 10, 10);

    set_material(&mut app, position, TileMaterial::Empty);
    run_update_chunk_sections(&mut app);

    c.bench_function("bench_update_chunk_sections_after_tile_unchanged", |b| {
        b.iter(|| {
            set_material(&mut app, position, TileMaterial::Empty);
            run_update_chunk_sections(&mut app);
        });
    });
}

fn bench_tile_chunk_sections_region_lookup(c: &mut Criterion) {
    let (mut app, layer) = make_app();
    let position = TilePosition::new(layer, 5, 5);

    set_material(&mut app, position, TileMaterial::Empty);
    run_update_chunk_sections(&mut app);

    let chunk_entity = app
        .world()
        .resource::<TileMap>()
        .get(position.chunk_position())
        .expect("missing benchmark chunk");
    let chunk_offset = position.chunk_offset();

    let region = app.world_mut().spawn_empty().id();
    {
        let mut chunk_sections = app
            .world_mut()
            .get_mut::<TileChunkSections>(chunk_entity)
            .expect("missing TileChunkSections component");
        let section_ids: Vec<_> = chunk_sections.sections().collect();
        for section_id in section_ids {
            chunk_sections.section_mut(section_id).set_region(region);
        }
    }

    c.bench_function("tile_chunk_sections_region_lookup", |b| {
        b.iter(|| {
            let chunk_sections = app
                .world()
                .get::<TileChunkSections>(chunk_entity)
                .expect("missing TileChunkSections component");

            black_box(chunk_sections.region(chunk_offset))
                .expect("region lookup should succeed for benchmark tile");
        });
    });
}

criterion_group!(
    benches,
    bench_update_chunk_sections_after_tile_modify,
    bench_update_chunk_sections_after_tile_unchanged,
    bench_tile_chunk_sections_region_lookup
);
criterion_main!(benches);

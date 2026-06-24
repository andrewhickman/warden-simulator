use bevy::{
    ecs::system::Commands,
    prelude::{Entity, Resource, Timer, TimerMode},
};
use rand::RngExt;
use std::{collections::VecDeque, time::Duration};
use wdn_physics::tile::{material::TileMaterial, position::TilePosition, storage::TileStorageMut};
use wdn_world::door::Door;

pub const MAP_SIZE: usize = 126;
pub const GRID_MUTATION_PERIOD_SECS: f32 = 16.0;
pub const GRID_MUTATION_TILES: usize = 4;

const WORLD_SIZE: usize = 512;
const TILE_SIZE: usize = 4;
const LOGICAL_SIZE: usize = WORLD_SIZE / TILE_SIZE;

const ALL_TILE_KINDS: [TileKind; 6] = [
    TileKind::Outside,
    TileKind::Inside,
    TileKind::CellsNorth,
    TileKind::CellsSouth,
    TileKind::CellsEast,
    TileKind::CellsWest,
];

const ALL_TILE_MASK: u8 = (1u8 << ALL_TILE_KINDS.len()) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Outside,
    Inside,
    CellsNorth,
    CellsSouth,
    CellsEast,
    CellsWest,
}

#[derive(Resource)]
pub struct GeneratedTileGrid {
    kinds: Box<[[TileKind; MAP_SIZE]; MAP_SIZE]>,
    timer: Timer,
}

impl GeneratedTileGrid {
    pub fn new() -> Self {
        Self {
            kinds: generate_random_tile_grid(),
            timer: Timer::from_seconds(GRID_MUTATION_PERIOD_SECS, TimerMode::Repeating),
        }
    }

    pub fn kinds(&self) -> &[[TileKind; MAP_SIZE]; MAP_SIZE] {
        &self.kinds
    }

    pub fn tick_and_maybe_regenerate(&mut self, delta: Duration) -> bool {
        if !self.timer.tick(delta).just_finished() {
            return false;
        }

        mutate_and_regenerate_tile_grid(&mut self.kinds, GRID_MUTATION_TILES);
        true
    }
}

impl TileKind {
    fn is_cells_variant(&self) -> bool {
        matches!(
            self,
            TileKind::CellsNorth | TileKind::CellsSouth | TileKind::CellsEast | TileKind::CellsWest
        )
    }

    pub fn allowed_north(&self, other: TileKind) -> bool {
        match (*self, other) {
            (TileKind::CellsNorth, TileKind::Inside) | (TileKind::Inside, TileKind::CellsSouth) => {
                true
            }
            (TileKind::CellsNorth, _) | (_, TileKind::CellsSouth) => false,
            _ => true,
        }
    }

    pub fn allowed_south(&self, other: TileKind) -> bool {
        match (*self, other) {
            (TileKind::CellsSouth, TileKind::Inside) | (TileKind::Inside, TileKind::CellsNorth) => {
                true
            }
            (TileKind::CellsSouth, _) | (_, TileKind::CellsNorth) => false,
            _ => true,
        }
    }

    pub fn allowed_east(&self, other: TileKind) -> bool {
        match (*self, other) {
            (TileKind::CellsEast, TileKind::Inside) | (TileKind::Inside, TileKind::CellsWest) => {
                true
            }
            (TileKind::CellsEast, _) | (_, TileKind::CellsWest) => false,
            _ => true,
        }
    }

    pub fn allowed_west(&self, other: TileKind) -> bool {
        match (*self, other) {
            (TileKind::CellsWest, TileKind::Inside) | (TileKind::Inside, TileKind::CellsEast) => {
                true
            }
            (TileKind::CellsWest, _) | (_, TileKind::CellsEast) => false,
            _ => true,
        }
    }
}

pub fn generate_random_tile_grid() -> Box<[[TileKind; MAP_SIZE]; MAP_SIZE]> {
    let mut random = rand::rng();

    for _ in 0..64 {
        if let Some(grid) = generate_random_tile_grid_attempt(&mut random) {
            return grid;
        }
    }

    panic!("wave function collapse failed to generate a valid grid after several attempts");
}

pub fn mutate_and_regenerate_tile_grid(
    kinds: &mut Box<[[TileKind; MAP_SIZE]; MAP_SIZE]>,
    cleared_tiles: usize,
) {
    let mut random = rand::rng();

    for _ in 0..64 {
        if let Some(next) =
            mutate_and_regenerate_tile_grid_attempt(kinds, cleared_tiles, &mut random)
        {
            *kinds = next;
            return;
        }
    }

    *kinds = generate_random_tile_grid();
}

pub fn apply_grid_to_map(
    commands: &mut Commands,
    storage: &mut TileStorageMut,
    layer: Entity,
    kinds: &[[TileKind; MAP_SIZE]; MAP_SIZE],
) -> usize {
    debug_assert_eq!(MAP_SIZE + 2, LOGICAL_SIZE);

    let mut desired = vec![TileMaterial::Empty; WORLD_SIZE * WORLD_SIZE];
    let mut desired_doors = Vec::<(usize, usize)>::new();

    let mut mark_wall = |x: usize, y: usize| {
        desired[world_xy_to_index(x, y)] = TileMaterial::Wall;
    };
    let mut mark_door = |x: usize, y: usize| {
        desired_doors.push((x, y));
    };

    for logical_y in 1..(LOGICAL_SIZE - 1) {
        for logical_x in 1..(LOGICAL_SIZE - 2) {
            let left = kinds[logical_y - 1][logical_x - 1];
            let right = kinds[logical_y - 1][logical_x];
            if should_fill_wall(left, right) {
                let world_x = (logical_x + 1) * TILE_SIZE;
                let world_y_start = logical_y * TILE_SIZE;
                for world_y in world_y_start..=(world_y_start + TILE_SIZE) {
                    mark_wall(world_x, world_y);
                }

                if is_inside_outside_border(left, right)
                    && deterministic_border_door(world_x, world_y_start + 2)
                {
                    mark_door(world_x, world_y_start + 2);
                }
            }

            if should_spawn_two_doors_horizontal(left, right) {
                let world_x = (logical_x + 1) * TILE_SIZE;
                let world_y_start = logical_y * TILE_SIZE;

                mark_door(world_x, world_y_start + 1);
                mark_door(world_x, world_y_start + 3);
            }
        }
    }

    for logical_y in 1..(LOGICAL_SIZE - 2) {
        for logical_x in 1..(LOGICAL_SIZE - 1) {
            let top = kinds[logical_y - 1][logical_x - 1];
            let bottom = kinds[logical_y][logical_x - 1];
            if should_fill_wall(top, bottom) {
                let world_y = (logical_y + 1) * TILE_SIZE;
                let world_x_start = logical_x * TILE_SIZE;
                for world_x in world_x_start..=(world_x_start + TILE_SIZE) {
                    mark_wall(world_x, world_y);
                }

                if is_inside_outside_border(top, bottom)
                    && deterministic_border_door(world_x_start + 2, world_y)
                {
                    mark_door(world_x_start + 2, world_y);
                }
            }

            if should_spawn_two_doors_vertical(top, bottom) {
                let world_y = (logical_y + 1) * TILE_SIZE;
                let world_x_start = logical_x * TILE_SIZE;

                mark_door(world_x_start + 1, world_y);
                mark_door(world_x_start + 3, world_y);
            }
        }
    }

    // Add walls between the generated interior ring and the empty map border.
    for logical_y in 1..(LOGICAL_SIZE - 1) {
        let left_edge_tile = kinds[logical_y - 1][0];
        if should_fill_wall(left_edge_tile, TileKind::Outside) {
            let world_x = TILE_SIZE;
            let world_y_start = logical_y * TILE_SIZE;
            for world_y in world_y_start..=(world_y_start + TILE_SIZE) {
                mark_wall(world_x, world_y);
            }

            if is_inside_outside_border(left_edge_tile, TileKind::Outside)
                && deterministic_border_door(world_x, world_y_start + 2)
            {
                mark_door(world_x, world_y_start + 2);
            }
        }

        let right_edge_tile = kinds[logical_y - 1][MAP_SIZE - 1];
        if should_fill_wall(right_edge_tile, TileKind::Outside) {
            let world_x = (LOGICAL_SIZE - 1) * TILE_SIZE;
            let world_y_start = logical_y * TILE_SIZE;
            for world_y in world_y_start..=(world_y_start + TILE_SIZE) {
                mark_wall(world_x, world_y);
            }

            if is_inside_outside_border(right_edge_tile, TileKind::Outside)
                && deterministic_border_door(world_x, world_y_start + 2)
            {
                mark_door(world_x, world_y_start + 2);
            }
        }
    }

    for logical_x in 1..(LOGICAL_SIZE - 1) {
        let top_edge_tile = kinds[0][logical_x - 1];
        if should_fill_wall(top_edge_tile, TileKind::Outside) {
            let world_y = TILE_SIZE;
            let world_x_start = logical_x * TILE_SIZE;
            for world_x in world_x_start..=(world_x_start + TILE_SIZE) {
                mark_wall(world_x, world_y);
            }

            if is_inside_outside_border(top_edge_tile, TileKind::Outside)
                && deterministic_border_door(world_x_start + 2, world_y)
            {
                mark_door(world_x_start + 2, world_y);
            }
        }

        let bottom_edge_tile = kinds[MAP_SIZE - 1][logical_x - 1];
        if should_fill_wall(bottom_edge_tile, TileKind::Outside) {
            let world_y = (LOGICAL_SIZE - 1) * TILE_SIZE;
            let world_x_start = logical_x * TILE_SIZE;
            for world_x in world_x_start..=(world_x_start + TILE_SIZE) {
                mark_wall(world_x, world_y);
            }

            if is_inside_outside_border(bottom_edge_tile, TileKind::Outside)
                && deterministic_border_door(world_x_start + 2, world_y)
            {
                mark_door(world_x_start + 2, world_y);
            }
        }
    }

    for kind_y in 0..MAP_SIZE {
        for kind_x in 0..MAP_SIZE {
            let tile_kind = kinds[kind_y][kind_x];
            let logical_x = kind_x + 1;
            let logical_y = kind_y + 1;
            let world_x_start = logical_x * TILE_SIZE;
            let world_y_start = logical_y * TILE_SIZE;

            match tile_kind {
                TileKind::CellsNorth | TileKind::CellsSouth => {
                    let center_x = world_x_start + 2;
                    for world_y in world_y_start..=(world_y_start + TILE_SIZE) {
                        mark_wall(center_x, world_y);
                    }
                }
                TileKind::CellsEast | TileKind::CellsWest => {
                    let center_y = world_y_start + 2;
                    for world_x in world_x_start..=(world_x_start + TILE_SIZE) {
                        mark_wall(world_x, center_y);
                    }
                }
                TileKind::Outside | TileKind::Inside => {}
            }
        }
    }

    for (x, y) in desired_doors {
        desired[world_xy_to_index(x, y)] = TileMaterial::Door;
    }

    let mut changed_tiles = 0usize;
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            let target = desired[world_xy_to_index(x, y)];
            let position = TilePosition::new(layer, x as i32, y as i32);
            changed_tiles += if target == TileMaterial::Door {
                spawn_door_if_needed(commands, storage, position) as usize
            } else {
                set_material_if_needed(commands, storage, position, target) as usize
            };
        }
    }

    changed_tiles
}

fn world_xy_to_index(x: usize, y: usize) -> usize {
    y * WORLD_SIZE + x
}

fn deterministic_border_door(x: usize, y: usize) -> bool {
    let mut hash = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
    hash ^= (y as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xFF51AFD7ED558CCD);
    hash ^= hash >> 33;
    (hash % 100) < 5
}

fn set_material_if_needed(
    commands: &mut Commands,
    storage: &mut TileStorageMut,
    position: TilePosition,
    material: TileMaterial,
) -> bool {
    let current = storage.get_material(position);
    if current == material {
        return false;
    }

    if current == TileMaterial::Door {
        if let Some(entity) = storage.index.get_tile(position) {
            commands.entity(entity).try_despawn();
        }
    }

    storage.set_material(position, material);
    true
}

fn spawn_door_if_needed(
    commands: &mut Commands,
    storage: &mut TileStorageMut,
    position: TilePosition,
) -> bool {
    if storage.get_material(position) == TileMaterial::Door {
        return false;
    }

    if let Some(entity) = storage.index.get_tile(position) {
        commands.entity(entity).try_despawn();
    }

    commands.spawn((Door::default(), position));
    true
}

fn should_fill_wall(a: TileKind, b: TileKind) -> bool {
    if a.is_cells_variant() || b.is_cells_variant() {
        return true;
    }

    matches!(
        (a, b),
        (TileKind::Outside, TileKind::Inside) | (TileKind::Inside, TileKind::Outside)
    )
}

fn is_inside_outside_border(a: TileKind, b: TileKind) -> bool {
    matches!(
        (a, b),
        (TileKind::Inside, TileKind::Outside) | (TileKind::Outside, TileKind::Inside)
    )
}

fn should_spawn_two_doors_vertical(a: TileKind, b: TileKind) -> bool {
    match (a, b) {
        (TileKind::CellsNorth, _) | (_, TileKind::CellsSouth) => true,
        _ => false,
    }
}

fn should_spawn_two_doors_horizontal(a: TileKind, b: TileKind) -> bool {
    match (a, b) {
        (TileKind::CellsEast, _) | (_, TileKind::CellsWest) => true,
        _ => false,
    }
}

fn mutate_and_regenerate_tile_grid_attempt<R: RngExt>(
    kinds: &[[TileKind; MAP_SIZE]; MAP_SIZE],
    cleared_tiles: usize,
    random: &mut R,
) -> Option<Box<[[TileKind; MAP_SIZE]; MAP_SIZE]>> {
    let mut possibilities = vec![0u8; MAP_SIZE * MAP_SIZE];

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            possibilities[xy_to_index(x, y)] = mask_for_tile(kinds[y][x]);
        }
    }

    for _ in 0..cleared_tiles {
        let index = random.random_range(0..possibilities.len());
        possibilities[index] = ALL_TILE_MASK;
    }

    if !propagate_all_constraints(&mut possibilities) {
        return None;
    }

    while let Some(index) = pick_lowest_entropy_index(&possibilities, random) {
        let tile_mask = possibilities[index];
        let selected_tile = random_tile_from_mask(tile_mask, random)?;
        possibilities[index] = mask_for_tile(selected_tile);

        if !propagate_constraints(&mut possibilities, index) {
            return None;
        }
    }

    possibilities_to_grid(possibilities)
}

fn generate_random_tile_grid_attempt<R: RngExt>(
    random: &mut R,
) -> Option<Box<[[TileKind; MAP_SIZE]; MAP_SIZE]>> {
    let mut possibilities = vec![ALL_TILE_MASK; MAP_SIZE * MAP_SIZE];

    while let Some(index) = pick_lowest_entropy_index(&possibilities, random) {
        let tile_mask = possibilities[index];
        let selected_tile = random_tile_from_mask(tile_mask, random)?;
        possibilities[index] = mask_for_tile(selected_tile);

        if !propagate_constraints(&mut possibilities, index) {
            return None;
        }
    }

    possibilities_to_grid(possibilities)
}

fn possibilities_to_grid(possibilities: Vec<u8>) -> Option<Box<[[TileKind; MAP_SIZE]; MAP_SIZE]>> {
    let mut rows = Vec::with_capacity(MAP_SIZE);
    for y in 0..MAP_SIZE {
        let mut row = [TileKind::Outside; MAP_SIZE];
        for x in 0..MAP_SIZE {
            let index = xy_to_index(x, y);
            row[x] = tile_from_single_mask(possibilities[index])?;
        }
        rows.push(row);
    }

    let boxed_rows = rows.into_boxed_slice();
    let boxed_grid = boxed_rows.try_into().ok()?;
    Some(boxed_grid)
}

fn pick_lowest_entropy_index<R: RngExt>(possibilities: &[u8], random: &mut R) -> Option<usize> {
    let mut best_entropy = u32::MAX;
    let mut candidates = Vec::new();

    for (index, &mask) in possibilities.iter().enumerate() {
        let entropy = mask.count_ones();
        if entropy <= 1 {
            continue;
        }

        if entropy < best_entropy {
            best_entropy = entropy;
            candidates.clear();
            candidates.push(index);
        } else if entropy == best_entropy {
            candidates.push(index);
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates[random.random_range(0..candidates.len())])
    }
}

fn propagate_constraints(possibilities: &mut [u8], start_index: usize) -> bool {
    let mut queue = VecDeque::new();
    queue.push_back(start_index);

    propagate_with_queue(possibilities, &mut queue)
}

fn propagate_all_constraints(possibilities: &mut [u8]) -> bool {
    let mut queue = (0..possibilities.len()).collect::<VecDeque<_>>();

    propagate_with_queue(possibilities, &mut queue)
}

fn propagate_with_queue(possibilities: &mut [u8], queue: &mut VecDeque<usize>) -> bool {
    while let Some(index) = queue.pop_front() {
        let x = index % MAP_SIZE;
        let y = index / MAP_SIZE;
        let source_mask = possibilities[index];

        if source_mask == 0 {
            return false;
        }

        if y > 0 {
            let neighbor_index = xy_to_index(x, y - 1);
            if !constrain_neighbor(
                possibilities,
                index,
                source_mask,
                neighbor_index,
                Direction::North,
                queue,
            ) {
                return false;
            }
        }

        if y + 1 < MAP_SIZE {
            let neighbor_index = xy_to_index(x, y + 1);
            if !constrain_neighbor(
                possibilities,
                index,
                source_mask,
                neighbor_index,
                Direction::South,
                queue,
            ) {
                return false;
            }
        }

        if x + 1 < MAP_SIZE {
            let neighbor_index = xy_to_index(x + 1, y);
            if !constrain_neighbor(
                possibilities,
                index,
                source_mask,
                neighbor_index,
                Direction::East,
                queue,
            ) {
                return false;
            }
        }

        if x > 0 {
            let neighbor_index = xy_to_index(x - 1, y);
            if !constrain_neighbor(
                possibilities,
                index,
                source_mask,
                neighbor_index,
                Direction::West,
                queue,
            ) {
                return false;
            }
        }
    }

    true
}

fn constrain_neighbor(
    possibilities: &mut [u8],
    source_index: usize,
    source_mask: u8,
    neighbor_index: usize,
    direction: Direction,
    queue: &mut VecDeque<usize>,
) -> bool {
    let allowed_neighbor_mask = compatible_neighbor_mask(source_mask, direction);
    let previous_neighbor_mask = possibilities[neighbor_index];
    let updated_neighbor_mask = previous_neighbor_mask & allowed_neighbor_mask;

    if updated_neighbor_mask == 0 {
        return false;
    }

    if updated_neighbor_mask != previous_neighbor_mask {
        possibilities[neighbor_index] = updated_neighbor_mask;
        queue.push_back(neighbor_index);
    }

    // Re-checking the source helps constraints bounce both ways through the queue.
    if updated_neighbor_mask != previous_neighbor_mask {
        queue.push_back(source_index);
    }

    true
}

fn compatible_neighbor_mask(source_mask: u8, direction: Direction) -> u8 {
    let mut neighbor_mask = 0u8;

    for source_tile in tiles_from_mask(source_mask) {
        for neighbor_tile in ALL_TILE_KINDS {
            let allowed = match direction {
                Direction::North => source_tile.allowed_north(neighbor_tile),
                Direction::South => source_tile.allowed_south(neighbor_tile),
                Direction::East => source_tile.allowed_east(neighbor_tile),
                Direction::West => source_tile.allowed_west(neighbor_tile),
            };

            if allowed {
                neighbor_mask |= mask_for_tile(neighbor_tile);
            }
        }
    }

    neighbor_mask
}

fn random_tile_from_mask<R: RngExt>(mask: u8, random: &mut R) -> Option<TileKind> {
    let mut candidates = [TileKind::Outside; ALL_TILE_KINDS.len()];
    let mut count = 0usize;

    for tile in tiles_from_mask(mask) {
        candidates[count] = tile;
        count += 1;
    }

    if count == 0 {
        None
    } else {
        Some(candidates[random.random_range(0..count)])
    }
}

fn tiles_from_mask(mask: u8) -> impl Iterator<Item = TileKind> {
    ALL_TILE_KINDS
        .into_iter()
        .enumerate()
        .filter(move |(idx, _)| mask & (1u8 << idx) != 0)
        .map(|(_, tile)| tile)
}

fn mask_for_tile(tile: TileKind) -> u8 {
    match tile {
        TileKind::Outside => 1 << 0,
        TileKind::Inside => 1 << 1,
        TileKind::CellsNorth => 1 << 2,
        TileKind::CellsSouth => 1 << 3,
        TileKind::CellsEast => 1 << 4,
        TileKind::CellsWest => 1 << 5,
    }
}

fn tile_from_single_mask(mask: u8) -> Option<TileKind> {
    if mask.count_ones() != 1 {
        return None;
    }

    Some(match mask {
        m if m == mask_for_tile(TileKind::Outside) => TileKind::Outside,
        m if m == mask_for_tile(TileKind::Inside) => TileKind::Inside,
        m if m == mask_for_tile(TileKind::CellsNorth) => TileKind::CellsNorth,
        m if m == mask_for_tile(TileKind::CellsSouth) => TileKind::CellsSouth,
        m if m == mask_for_tile(TileKind::CellsEast) => TileKind::CellsEast,
        m if m == mask_for_tile(TileKind::CellsWest) => TileKind::CellsWest,
        _ => return None,
    })
}

fn xy_to_index(x: usize, y: usize) -> usize {
    y * MAP_SIZE + x
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_grid_respects_adjacency_constraints() {
        let grid = generate_random_tile_grid();

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let tile = grid[y][x];

                if y > 0 {
                    assert!(tile.allowed_north(grid[y - 1][x]));
                }
                if y + 1 < MAP_SIZE {
                    assert!(tile.allowed_south(grid[y + 1][x]));
                }
                if x + 1 < MAP_SIZE {
                    assert!(tile.allowed_east(grid[y][x + 1]));
                }
                if x > 0 {
                    assert!(tile.allowed_west(grid[y][x - 1]));
                }
            }
        }
    }
}

//! Benchmarks for ECS performance comparison

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ecs::prelude::*;

#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug, Clone)]
struct Health {
    current: i32,
    max: i32,
}

/// Benchmark legacy HashMap-based ECS
fn benchmark_legacy_ecs(c: &mut Criterion) {
    let mut world = World::new();
    world.initialize().unwrap();
    world.start().unwrap();

    // Create entities with components
    for i in 0..1000 {
        let entity = world.create_entity();
        world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
        if i % 3 == 0 {
            world.add_component(&entity, Health { current: 100, max: 100 }).unwrap();
        }
    }

    c.bench_function("legacy_ecs_component_access", |b| {
        b.iter(|| {
            // Simulate component access pattern
            for entity_id in 0..1000 {
                let entity = EntityId::new(entity_id);
                if let Ok(pos) = world.get_component::<Position>(&entity) {
                    let pos = pos as &Position;
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    c.bench_function("legacy_ecs_entity_iteration", |b| {
        b.iter(|| {
            // Simulate entity iteration
            for entity_id in 0..1000 {
                let entity = EntityId::new(entity_id);
                if world.has_component::<Position>(&entity).unwrap() && 
                   world.has_component::<Velocity>(&entity).unwrap() {
                    black_box(entity_id);
                }
            }
        });
    });
}

/// Benchmark archetype-based ECS
fn benchmark_archetype_ecs(c: &mut Criterion) {
    let mut world = World::new_optimized();
    world.initialize().unwrap();
    world.start().unwrap();

    // Create entities with components
    for i in 0..1000 {
        let entity = world.create_entity();
        world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
        if i % 3 == 0 {
            world.add_component(&entity, Health { current: 100, max: 100 }).unwrap();
        }
    }

    c.bench_function("archetype_ecs_component_access", |b| {
        b.iter(|| {
            // Simulate component access pattern
            for entity_id in 0..1000 {
                let entity = EntityId::new(entity_id);
                if let Ok(pos) = world.get_component::<Position>(&entity) {
                    let pos = pos as &Position;
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    c.bench_function("archetype_ecs_entity_iteration", |b| {
        b.iter(|| {
            // Simulate entity iteration
            for entity_id in 0..1000 {
                let entity = EntityId::new(entity_id);
                if world.has_component::<Position>(&entity).unwrap() && 
                   world.has_component::<Velocity>(&entity).unwrap() {
                    black_box(entity_id);
                }
            }
        });
    });
}

/// Benchmark entity creation
fn benchmark_entity_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_creation");

    group.bench_function("legacy", |b| {
        b.iter(|| {
            let mut world = World::new();
            world.initialize().unwrap();
            
            for i in 0..100 {
                let entity = world.create_entity();
                world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
                world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
            }
            black_box(world.entity_count());
        });
    });

    group.bench_function("archetype", |b| {
        b.iter(|| {
            let mut world = World::new_optimized();
            world.initialize().unwrap();
            
            for i in 0..100 {
                let entity = world.create_entity();
                world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
                world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
            }
            black_box(world.entity_count());
        });
    });

    group.finish();
}

/// Benchmark component operations
fn benchmark_component_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_operations");

    // Setup legacy world
    let mut legacy_world = World::new();
    legacy_world.initialize().unwrap();
    let legacy_entities: Vec<_> = (0..100).map(|i| {
        let entity = legacy_world.create_entity();
        legacy_world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        entity
    }).collect();

    // Setup archetype world
    let mut archetype_world = World::new_optimized();
    archetype_world.initialize().unwrap();
    let archetype_entities: Vec<_> = (0..100).map(|i| {
        let entity = archetype_world.create_entity();
        archetype_world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        entity
    }).collect();

    group.bench_function("legacy_add_component", |b| {
        b.iter(|| {
            for entity in &legacy_entities {
                legacy_world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
            }
        });
    });

    group.bench_function("archetype_add_component", |b| {
        b.iter(|| {
            for entity in &archetype_entities {
                archetype_world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
            }
        });
    });

    group.bench_function("legacy_get_component", |b| {
        b.iter(|| {
            for entity in &legacy_entities {
                if let Ok(pos) = legacy_world.get_component::<Position>(&entity) {
                    let pos = pos as &Position;
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    group.bench_function("archetype_get_component", |b| {
        b.iter(|| {
            for entity in &archetype_entities {
                if let Ok(pos) = archetype_world.get_component::<Position>(&entity) {
                    let pos = pos as &Position;
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_legacy_ecs,
    benchmark_archetype_ecs,
    benchmark_entity_creation,
    benchmark_component_operations
);
criterion_main!(benches);
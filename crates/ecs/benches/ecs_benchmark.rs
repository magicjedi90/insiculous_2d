//! Benchmarks for ECS performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ecs::prelude::*;

#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Health {
    current: i32,
    max: i32,
}

/// Benchmark ECS component access and iteration
fn benchmark_ecs(c: &mut Criterion) {
    let mut world = World::new();
    world.initialize().unwrap();
    world.start().unwrap();

    // Create entities with components and store their IDs
    let entities: Vec<EntityId> = (0..1000).map(|i| {
        let entity = world.create_entity();
        world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        world.add_component(&entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
        if i % 3 == 0 {
            world.add_component(&entity, Health { current: 100, max: 100 }).unwrap();
        }
        entity
    }).collect();

    c.bench_function("ecs_component_access", |b| {
        b.iter(|| {
            // Access components using typed getter
            for entity in &entities {
                if let Some(pos) = world.get::<Position>(*entity) {
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    c.bench_function("ecs_entity_iteration", |b| {
        b.iter(|| {
            // Iterate using entities() method
            for entity_id in world.entities() {
                if world.get::<Position>(entity_id).is_some() &&
                   world.get::<Velocity>(entity_id).is_some() {
                    black_box(entity_id);
                }
            }
        });
    });
}

/// Benchmark entity creation
fn benchmark_entity_creation(c: &mut Criterion) {
    c.bench_function("entity_creation", |b| {
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
}

/// Benchmark component operations
fn benchmark_component_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_operations");

    let mut world = World::new();
    world.initialize().unwrap();
    let entities: Vec<_> = (0..100).map(|i| {
        let entity = world.create_entity();
        world.add_component(&entity, Position { x: i as f32, y: i as f32 }).unwrap();
        entity
    }).collect();

    group.bench_function("add_component", |b| {
        b.iter(|| {
            for entity in &entities {
                let _ = world.add_component(entity, Velocity { dx: 1.0, dy: 1.0 });
            }
        });
    });

    group.bench_function("get_component", |b| {
        b.iter(|| {
            for entity in &entities {
                if let Some(pos) = world.get::<Position>(*entity) {
                    black_box(pos.x + pos.y);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ecs,
    benchmark_entity_creation,
    benchmark_component_operations
);
criterion_main!(benches);

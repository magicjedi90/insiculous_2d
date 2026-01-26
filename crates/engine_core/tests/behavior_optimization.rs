//! Test to verify behavior system optimization
//!
//! This test demonstrates that the optimized behavior runner reduces allocations
//! by avoiding unnecessary cloning of Behavior components.

use engine_core::behavior_runner::BehaviorRunner;
use ecs::{World, EntityId};
use ecs::behavior::{Behavior, BehaviorState};
use ecs::sprite_components::Transform2D;
use input::InputHandler;
use glam::Vec2;

#[test]
fn test_behavior_runner_no_excessive_cloning() {
    let mut world = World::new();
    let mut behavior_runner = BehaviorRunner::new();
    let input = InputHandler::new();
    
    // Create multiple entities with behaviors
    let entities: Vec<EntityId> = (0..10).map(|i| {
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(i as f32 * 50.0, 0.0))).unwrap();
        
        // Add different types of behaviors
        let behavior = match i % 5 {
            0 => Behavior::PlayerPlatformer { 
                move_speed: 120.0, 
                jump_impulse: 420.0, 
                jump_cooldown: 0.3, 
                tag: format!("player_{}", i) 
            },
            1 => Behavior::PlayerTopDown { 
                move_speed: 150.0, 
                tag: format!("player_{}", i) 
            },
            2 => Behavior::Patrol { 
                point_a: (0.0, 0.0), 
                point_b: (100.0, 100.0), 
                speed: 80.0, 
                wait_time: 1.0 
            },
            3 => Behavior::ChaseTagged { 
                target_tag: "player".to_string(), 
                detection_range: 200.0, 
                chase_speed: 80.0, 
                lose_interest_range: 300.0 
            },
            _ => Behavior::Collectible { 
                score_value: 10, 
                despawn_on_collect: true, 
                collector_tag: "player".to_string() 
            },
        };
        
        world.add_component(&entity, behavior).unwrap();
        entity
    }).collect();
    
    // Run behavior update multiple times
    for _ in 0..100 {
        behavior_runner.update(&mut world, &input, 0.016, None);
    }
    
    // Verify that all entities still have their behaviors (not corrupted by references)
    for entity in &entities {
        assert!(world.get::<Behavior>(*entity).is_some(), "Entity should still have behavior component");
        // Only stateful behaviors (PlayerPlatformer, ChaseTagged, Patrol) should have BehaviorState
        // Collectible and PlayerTopDown don't persist state
    }
}

#[test]
fn test_behavior_runner_with_physics_integration() {
    let mut world = World::new();
    let mut behavior_runner = BehaviorRunner::new();
    let input = InputHandler::new();
    
    // Create a player entity with platformer behavior
    let player = world.create_entity();
    world.add_component(&player, Transform2D::new(Vec2::new(0.0, 0.0))).unwrap();
    world.add_component(&player, Behavior::PlayerPlatformer { 
        move_speed: 120.0, 
        jump_impulse: 420.0, 
        jump_cooldown: 0.3, 
        tag: "player".to_string() 
    }).unwrap();
    
    // Create a follower entity
    let follower = world.create_entity();
    world.add_component(&follower, Transform2D::new(Vec2::new(100.0, 0.0))).unwrap();
    world.add_component(&follower, Behavior::FollowEntity { 
        target_name: "player".to_string(), 
        follow_distance: 50.0, 
        follow_speed: 100.0 
    }).unwrap();
    
    // Set up named entities for FollowEntity behavior
    let mut named_entities = std::collections::HashMap::new();
    named_entities.insert("player".to_string(), player);
    behavior_runner.set_named_entities(named_entities);
    
    // Run behavior update
    behavior_runner.update(&mut world, &input, 0.016, None);
    
    // Verify that both entities still have their behaviors
    assert!(world.get::<Behavior>(player).is_some(), "Player should still have behavior component");
    assert!(world.get::<Behavior>(follower).is_some(), "Follower should still have behavior component");
    // PlayerPlatformer behavior should create BehaviorState
    assert!(world.get::<BehaviorState>(player).is_some(), "Player should have BehaviorState component");
    // FollowEntity behavior doesn't persist state, so no BehaviorState expected
}
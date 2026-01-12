//! Tests for system lifecycle management.

use ecs::prelude::*;
use ecs::system::SystemRegistry;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct TestSystem {
    name: String,
    initialized: Arc<Mutex<bool>>,
    started: Arc<Mutex<bool>>,
    stopped: Arc<Mutex<bool>>,
    shutdown: Arc<Mutex<bool>>,
    update_count: Arc<Mutex<u32>>,
}

impl TestSystem {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            initialized: Arc::new(Mutex::new(false)),
            started: Arc::new(Mutex::new(false)),
            stopped: Arc::new(Mutex::new(false)),
            shutdown: Arc::new(Mutex::new(false)),
            update_count: Arc::new(Mutex::new(0)),
        }
    }

    fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    fn is_started(&self) -> bool {
        *self.started.lock().unwrap()
    }

    fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap()
    }

    fn is_shutdown(&self) -> bool {
        *self.shutdown.lock().unwrap()
    }

    fn update_count(&self) -> u32 {
        *self.update_count.lock().unwrap()
    }
}

impl System for TestSystem {
    fn initialize(&mut self, _world: &mut World) -> Result<(), String> {
        *self.initialized.lock().unwrap() = true;
        log::debug!("System '{}' initialized", self.name);
        Ok(())
    }

    fn start(&mut self, _world: &mut World) -> Result<(), String> {
        if !self.is_initialized() {
            return Err("System not initialized".to_string());
        }
        if self.is_started() {
            return Err("System already started".to_string());
        }
        *self.started.lock().unwrap() = true;
        log::debug!("System '{}' started", self.name);
        Ok(())
    }

    fn update(&mut self, _world: &mut World, _delta_time: f32) {
        if self.is_started() && !self.is_stopped() {
            *self.update_count.lock().unwrap() += 1;
        }
    }

    fn stop(&mut self, _world: &mut World) -> Result<(), String> {
        if !self.is_started() {
            return Err("System not started".to_string());
        }
        if self.is_stopped() {
            return Err("System already stopped".to_string());
        }
        *self.stopped.lock().unwrap() = true;
        log::debug!("System '{}' stopped", self.name);
        Ok(())
    }

    fn shutdown(&mut self, _world: &mut World) -> Result<(), String> {
        if !self.is_initialized() {
            return Err("System not initialized".to_string());
        }
        *self.shutdown.lock().unwrap() = true;
        log::debug!("System '{}' shut down", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[test]
fn test_system_lifecycle() {
    let mut world = World::new();
    let mut system = TestSystem::new("TestSystem");
    
    // Test lifecycle
    assert!(!system.is_initialized());
    assert!(!system.is_started());
    assert!(!system.is_stopped());
    assert!(!system.is_shutdown());
    assert_eq!(system.update_count(), 0);
    
    // Initialize
    system.initialize(&mut world).unwrap();
    assert!(system.is_initialized());
    
    // Start
    system.start(&mut world).unwrap();
    assert!(system.is_started());
    
    // Update (should increment count)
    system.update(&mut world, 0.016);
    assert_eq!(system.update_count(), 1);
    
    // Stop
    system.stop(&mut world).unwrap();
    assert!(system.is_stopped());
    
    // Update after stop (should not increment)
    system.update(&mut world, 0.016);
    assert_eq!(system.update_count(), 1);
    
    // Shutdown
    system.shutdown(&mut world).unwrap();
    assert!(system.is_shutdown());
}

#[test]
fn test_system_registry_lifecycle() {
    let mut registry = SystemRegistry::new();
    let mut world = World::new();
    
    let system1 = TestSystem::new("System1");
    let system2 = TestSystem::new("System2");
    
    registry.add(system1);
    registry.add(system2);
    
    assert_eq!(registry.len(), 2);
    
    // Initialize
    registry.initialize().unwrap();
    
    // Start
    registry.start().unwrap();
    
    // Update systems through world
    world.initialize().unwrap();
    world.start().unwrap();
    world.update(0.016).unwrap();
    
    // Stop
    registry.stop().unwrap();
    
    // Shutdown
    registry.shutdown().unwrap();
}

#[test]
fn test_system_lifecycle_errors() {
    let mut world = World::new();
    let mut system = TestSystem::new("TestSystem");
    
    // Try to start without initialization
    let result = system.start(&mut world);
    assert!(result.is_err());
    
    // Initialize
    system.initialize(&mut world).unwrap();
    
    // Try to start twice
    system.start(&mut world).unwrap();
    let result = system.start(&mut world);
    assert!(result.is_err());
    
    // Try to stop without being started
    system.stop(&mut world).unwrap(); // Stop it first
    let result = system.stop(&mut world);
    assert!(result.is_err());
    
    // Try to shutdown without initialization
    let mut new_system = TestSystem::new("NewSystem");
    let result = new_system.shutdown(&mut world);
    assert!(result.is_err());
}

#[test]
fn test_system_registry_error_handling() {
    let mut registry = SystemRegistry::new();
    
    // Try to start without initialization
    let result = registry.start();
    assert!(result.is_err());
    
    // Initialize
    registry.initialize().unwrap();
    
    // Try to initialize twice
    let result = registry.initialize();
    assert!(result.is_err());
    
    // Start
    registry.start().unwrap();
    
    // Try to start twice
    let result = registry.start();
    assert!(result.is_err());
    
    // Stop
    registry.stop().unwrap();
    
    // Try to stop twice
    let result = registry.stop();
    assert!(result.is_err());
}

#[test]
fn test_system_update_safety() {
    let mut registry = SystemRegistry::new();
    let mut world = World::new();
    
    let system = TestSystem::new("UpdateTest");
    registry.add(system);
    
    // Try to update without starting (should log warning but not panic)
    registry.update_all(&mut world, 0.016);
    
    // Initialize and start
    registry.initialize().unwrap();
    registry.start().unwrap();
    
    // Update should work now
    world.initialize().unwrap();
    world.start().unwrap();
    world.update(0.016).unwrap();
    
    // Stop and try to update (should log warning)
    registry.stop().unwrap();
    registry.update_all(&mut world, 0.016);
}

#[test]
fn test_panic_recovery_in_systems() {
    use std::panic;
    
    struct PanicSystem;
    
    impl System for PanicSystem {
        fn update(&mut self, _world: &mut World, _delta_time: f32) {
            panic!("Test panic in system");
        }
        
        fn name(&self) -> &str {
            "PanicSystem"
        }
    }
    
    let mut registry = SystemRegistry::new();
    let mut world = World::new();
    
    registry.add(PanicSystem);
    registry.initialize().unwrap();
    registry.start().unwrap();
    
    world.initialize().unwrap();
    world.start().unwrap();
    
    // This should not panic the whole engine
    registry.update_all(&mut world, 0.016);
    
    // Other systems should still work
    let normal_system = TestSystem::new("NormalSystem");
    registry.add(normal_system);
    
    // Should still be able to update
    registry.update_all(&mut world, 0.016);
}

#[test]
fn test_world_lifecycle_integration() {
    let mut world = World::new();
    let system = TestSystem::new("WorldIntegration");
    world.add_system(system);
    
    // Test full world lifecycle
    assert!(!world.is_initialized());
    assert!(!world.is_running());
    
    world.initialize().unwrap();
    assert!(world.is_initialized());
    
    world.start().unwrap();
    assert!(world.is_running());
    
    // Should be able to update
    world.update(0.016).unwrap();
    
    world.stop().unwrap();
    assert!(!world.is_running());
    
    world.shutdown().unwrap();
    assert!(!world.is_initialized());
}
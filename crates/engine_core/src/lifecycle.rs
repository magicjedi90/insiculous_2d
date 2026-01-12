//! Lifecycle management for engine systems.

use std::sync::{Mutex, RwLock};

/// Represents the current state of a system or component lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// System is created but not initialized
    Created,
    /// System is being initialized
    Initializing,
    /// System is initialized and ready
    Initialized,
    /// System is running/active
    Running,
    /// System is being shut down
    ShuttingDown,
    /// System is shut down
    ShutDown,
    /// System encountered an error
    Error,
}

impl LifecycleState {
    /// Check if the system is in a valid operational state
    pub fn is_operational(&self) -> bool {
        matches!(self, LifecycleState::Initialized | LifecycleState::Running)
    }

    /// Check if the system can be initialized
    pub fn can_initialize(&self) -> bool {
        matches!(self, LifecycleState::Created | LifecycleState::ShutDown | LifecycleState::Error)
    }

    /// Check if the system can be shut down
    pub fn can_shutdown(&self) -> bool {
        matches!(self, LifecycleState::Initialized | LifecycleState::Running | LifecycleState::Error)
    }

    /// Check if the system can start running
    pub fn can_start(&self) -> bool {
        matches!(self, LifecycleState::Initialized)
    }

    /// Check if the system can stop running
    pub fn can_stop(&self) -> bool {
        matches!(self, LifecycleState::Running)
    }
}

impl Clone for LifecycleManager {
    fn clone(&self) -> Self {
        let current_state = self.current_state();
        Self::new_with_state(current_state)
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager with a specific state
    pub fn new_with_state(state: LifecycleState) -> Self {
        Self {
            state: RwLock::new(state),
            init_lock: Mutex::new(false),
            shutdown_lock: Mutex::new(false),
        }
    }
}

/// Thread-safe lifecycle state manager
#[derive(Debug)]
pub struct LifecycleManager {
    /// Current lifecycle state
    state: RwLock<LifecycleState>,
    /// Initialization lock to prevent concurrent initialization
    init_lock: Mutex<bool>,
    /// Shutdown lock to prevent concurrent shutdown
    shutdown_lock: Mutex<bool>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager in Created state
    pub fn new() -> Self {
        Self {
            state: RwLock::new(LifecycleState::Created),
            init_lock: Mutex::new(false),
            shutdown_lock: Mutex::new(false),
        }
    }

    /// Get the current lifecycle state
    pub fn current_state(&self) -> LifecycleState {
        *self.state.read().unwrap()
    }

    /// Check if the system is in an operational state
    pub fn is_operational(&self) -> bool {
        self.current_state().is_operational()
    }

    /// Check if the system can be initialized
    pub fn can_initialize(&self) -> bool {
        self.current_state().can_initialize()
    }

    /// Check if the system can be shut down
    pub fn can_shutdown(&self) -> bool {
        self.current_state().can_shutdown()
    }

    /// Check if the system can start running
    pub fn can_start(&self) -> bool {
        self.current_state().can_start()
    }

    /// Check if the system can stop running
    pub fn can_stop(&self) -> bool {
        self.current_state().can_stop()
    }

    /// Transition to initializing state
    pub fn begin_initialization(&self) -> Result<(), String> {
        let mut init_lock = self.init_lock.lock().unwrap();
        if *init_lock {
            return Err("Initialization already in progress".to_string());
        }

        let current = self.current_state();
        if !current.can_initialize() {
            return Err(format!("Cannot initialize from state {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::Initializing;
        *init_lock = true;
        Ok(())
    }

    /// Complete initialization
    pub fn complete_initialization(&self) -> Result<(), String> {
        let mut init_lock = self.init_lock.lock().unwrap();
        if !*init_lock {
            return Err("No initialization in progress".to_string());
        }

        let current = self.current_state();
        if current != LifecycleState::Initializing {
            return Err(format!("Expected Initializing state, got {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::Initialized;
        *init_lock = false;
        Ok(())

    }

    /// Transition to running state
    pub fn start(&self) -> Result<(), String> {
        let current = self.current_state();
        if !current.can_start() {
            return Err(format!("Cannot start from state {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::Running;
        Ok(())
    }

    /// Transition to stopped state
    pub fn stop(&self) -> Result<(), String> {
        let current = self.current_state();
        if !current.can_stop() {
            return Err(format!("Cannot stop from state {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::Initialized;
        Ok(())
    }

    /// Begin shutdown
    pub fn begin_shutdown(&self) -> Result<(), String> {
        let mut shutdown_lock = self.shutdown_lock.lock().unwrap();
        if *shutdown_lock {
            return Err("Shutdown already in progress".to_string());
        }

        let current = self.current_state();
        if !current.can_shutdown() {
            return Err(format!("Cannot shutdown from state {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::ShuttingDown;
        *shutdown_lock = true;
        Ok(())
    }

    /// Complete shutdown
    pub fn complete_shutdown(&self) -> Result<(), String> {
        let mut shutdown_lock = self.shutdown_lock.lock().unwrap();
        if !*shutdown_lock {
            return Err("No shutdown in progress".to_string());
        }

        let current = self.current_state();
        if current != LifecycleState::ShuttingDown {
            return Err(format!("Expected ShuttingDown state, got {:?}", current));
        }

        *self.state.write().unwrap() = LifecycleState::ShutDown;
        *shutdown_lock = false;
        Ok(())
    }

    /// Transition to error state
    pub fn set_error(&self, error: Option<String>) -> Result<(), String> {
        *self.state.write().unwrap() = LifecycleState::Error;
        
        // Release any locks that might be held
        if let Ok(mut init_lock) = self.init_lock.lock() {
            *init_lock = false;
        }
        if let Ok(mut shutdown_lock) = self.shutdown_lock.lock() {
            *shutdown_lock = false;
        }

        if let Some(err) = error {
            return Err(err);
        }
        Ok(())
    }

    /// Wait until the system reaches a specific state (with timeout)
    pub fn wait_for_state(&self, target: LifecycleState, timeout: std::time::Duration) -> Result<(), String> {
        let start = std::time::Instant::now();
        
        while self.current_state() != target {
            if start.elapsed() > timeout {
                return Err(format!("Timeout waiting for state {:?}", target));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        Ok(())
    }
}

/// A trait for objects that have a managed lifecycle
pub trait Lifecycle: Send + Sync {
    /// Initialize the system
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Start the system (transition to running state)
    fn start(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Stop the system (transition from running to initialized)
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Shutdown the system
    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Get the lifecycle manager for this system
    fn lifecycle(&self) -> &LifecycleManager;
}
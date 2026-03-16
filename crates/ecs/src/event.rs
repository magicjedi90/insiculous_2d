//! Typed event bus for loose-coupled communication between systems.
//!
//! Events are emitted during a frame and readable by any system during
//! the same frame. Call `flush_events()` at the end of each frame to
//! clear all queues.
//!
//! # Example
//! ```ignore
//! use ecs::World;
//!
//! #[derive(Debug, Clone)]
//! struct CoinCollected { entity_id: u64, value: u32 }
//!
//! let mut world = World::new();
//! world.emit_event(CoinCollected { entity_id: 1, value: 10 });
//! world.emit_event(CoinCollected { entity_id: 2, value: 5 });
//!
//! for event in world.read_events::<CoinCollected>() {
//!     println!("Collected {} coins", event.value);
//! }
//!
//! world.flush_events(); // Call at end of frame
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Trait object interface for type-erased event queue operations.
trait EventQueueOps: Send + Sync {
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A typed queue holding events of a single type.
struct TypedEventQueue<E: Send + Sync + 'static> {
    events: Vec<E>,
}

impl<E: Send + Sync + 'static> TypedEventQueue<E> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl<E: Send + Sync + 'static> EventQueueOps for TypedEventQueue<E> {
    fn clear(&mut self) {
        self.events.clear();
    }

    fn len(&self) -> usize {
        self.events.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// An event bus that stores per-type event queues.
///
/// Events are emitted with `emit()`, read with `read()`, and
/// cleared each frame with `flush()`.
pub struct EventBus {
    queues: HashMap<TypeId, Box<dyn EventQueueOps>>,
}

impl EventBus {
    /// Create an empty event bus.
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    /// Emit an event. It will be readable until the next `flush()`.
    pub fn emit<E: Send + Sync + 'static>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        let queue = self.queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E>::new()));

        let typed = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .expect("event queue type mismatch");
        typed.events.push(event);
    }

    /// Read all events of type `E` emitted since the last flush.
    /// Returns an empty slice if no events of this type exist.
    pub fn read<E: Send + Sync + 'static>(&self) -> &[E] {
        let type_id = TypeId::of::<E>();
        self.queues
            .get(&type_id)
            .and_then(|queue| {
                queue
                    .as_any()
                    .downcast_ref::<TypedEventQueue<E>>()
                    .map(|typed| typed.events.as_slice())
            })
            .unwrap_or(&[])
    }

    /// Get the count of pending events for a specific type.
    pub fn count<E: Send + Sync + 'static>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.queues
            .get(&type_id)
            .map(|queue| queue.len())
            .unwrap_or(0)
    }

    /// Clear all event queues. Call this at the end of each frame.
    pub fn flush(&mut self) {
        for queue in self.queues.values_mut() {
            queue.clear();
        }
    }

    /// Check if there are any pending events of type `E`.
    pub fn has_events<E: Send + Sync + 'static>(&self) -> bool {
        self.count::<E>() > 0
    }

    /// Get the total number of registered event types.
    pub fn type_count(&self) -> usize {
        self.queues.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct CoinCollected {
        value: u32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct PlayerDied {
        player_id: u64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct DamageDealt {
        source: u64,
        target: u64,
        amount: f32,
    }

    #[test]
    fn test_emit_and_read_events() {
        let mut bus = EventBus::new();
        bus.emit(CoinCollected { value: 10 });
        bus.emit(CoinCollected { value: 5 });

        let events = bus.read::<CoinCollected>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].value, 10);
        assert_eq!(events[1].value, 5);
    }

    #[test]
    fn test_read_empty_returns_empty_slice() {
        let bus = EventBus::new();
        let events = bus.read::<CoinCollected>();
        assert!(events.is_empty());
    }

    #[test]
    fn test_flush_clears_all_events() {
        let mut bus = EventBus::new();
        bus.emit(CoinCollected { value: 10 });
        bus.emit(PlayerDied { player_id: 1 });

        bus.flush();

        assert!(bus.read::<CoinCollected>().is_empty());
        assert!(bus.read::<PlayerDied>().is_empty());
    }

    #[test]
    fn test_multiple_event_types_independent() {
        let mut bus = EventBus::new();
        bus.emit(CoinCollected { value: 10 });
        bus.emit(PlayerDied { player_id: 1 });
        bus.emit(CoinCollected { value: 20 });

        assert_eq!(bus.read::<CoinCollected>().len(), 2);
        assert_eq!(bus.read::<PlayerDied>().len(), 1);
    }

    #[test]
    fn test_has_events() {
        let mut bus = EventBus::new();
        assert!(!bus.has_events::<CoinCollected>());

        bus.emit(CoinCollected { value: 5 });
        assert!(bus.has_events::<CoinCollected>());
        assert!(!bus.has_events::<PlayerDied>());
    }

    #[test]
    fn test_count_events() {
        let mut bus = EventBus::new();
        assert_eq!(bus.count::<CoinCollected>(), 0);

        bus.emit(CoinCollected { value: 1 });
        bus.emit(CoinCollected { value: 2 });
        bus.emit(CoinCollected { value: 3 });

        assert_eq!(bus.count::<CoinCollected>(), 3);
    }

    #[test]
    fn test_type_count() {
        let mut bus = EventBus::new();
        assert_eq!(bus.type_count(), 0);

        bus.emit(CoinCollected { value: 1 });
        assert_eq!(bus.type_count(), 1);

        bus.emit(PlayerDied { player_id: 1 });
        assert_eq!(bus.type_count(), 2);
    }

    #[test]
    fn test_flush_preserves_queue_allocations() {
        let mut bus = EventBus::new();
        bus.emit(CoinCollected { value: 1 });
        bus.flush();

        // Queue type still registered, just empty
        assert_eq!(bus.type_count(), 1);
        assert!(!bus.has_events::<CoinCollected>());

        // Can still emit after flush
        bus.emit(CoinCollected { value: 2 });
        assert_eq!(bus.read::<CoinCollected>()[0].value, 2);
    }

    #[test]
    fn test_complex_event_data() {
        let mut bus = EventBus::new();
        bus.emit(DamageDealt {
            source: 1,
            target: 2,
            amount: 25.5,
        });

        let events = bus.read::<DamageDealt>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, 1);
        assert_eq!(events[0].target, 2);
        assert_eq!(events[0].amount, 25.5);
    }

    #[test]
    fn test_events_readable_multiple_times_before_flush() {
        let mut bus = EventBus::new();
        bus.emit(CoinCollected { value: 10 });

        // Read twice — same data both times
        assert_eq!(bus.read::<CoinCollected>().len(), 1);
        assert_eq!(bus.read::<CoinCollected>().len(), 1);
    }
}

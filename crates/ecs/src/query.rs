//! Query types for type-safe entity queries.
//!
//! This module provides the `QueryTypes` trait and concrete query types
//! (`Single`, `Pair`, `Triple`) used by `World::query_entities()`.

use std::any::TypeId;
use std::marker::PhantomData;

use crate::component::Component;

/// Trait for defining query types used by `World::query_entities()`
pub trait QueryTypes {
    /// Get the component types required by this query
    fn component_types() -> Vec<TypeId>;
}

/// Single component query
pub struct Single<T: Component> {
    _phantom: PhantomData<T>,
}

impl<T: Component> QueryTypes for Single<T> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

/// Two component query
pub struct Pair<T: Component, U: Component> {
    _phantom: PhantomData<(T, U)>,
}

impl<T: Component, U: Component> QueryTypes for Pair<T, U> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>(), TypeId::of::<U>()]
    }
}

/// Three component query
pub struct Triple<T: Component, U: Component, V: Component> {
    _phantom: PhantomData<(T, U, V)>,
}

impl<T: Component, U: Component, V: Component> QueryTypes for Triple<T, U, V> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>(), TypeId::of::<U>(), TypeId::of::<V>()]
    }
}

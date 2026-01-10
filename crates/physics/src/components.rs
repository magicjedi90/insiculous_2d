//! Physics components for ECS integration
//!
//! These components wrap rapier2d concepts and can be attached to ECS entities.

use glam::Vec2;

/// Body type for physics simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    /// A dynamic body affected by forces and collisions
    Dynamic,
    /// A static body that never moves
    Static,
    /// A kinematic body controlled directly by the user
    Kinematic,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        Self::Dynamic
    }
}

/// Rigid body component for physics simulation
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// Type of rigid body
    pub body_type: RigidBodyType,
    /// Linear velocity in units per second
    pub velocity: Vec2,
    /// Angular velocity in radians per second
    pub angular_velocity: f32,
    /// Gravity scale (1.0 = normal gravity, 0.0 = no gravity)
    pub gravity_scale: f32,
    /// Linear damping (velocity decay per second)
    pub linear_damping: f32,
    /// Angular damping (angular velocity decay per second)
    pub angular_damping: f32,
    /// Whether this body can rotate
    pub can_rotate: bool,
    /// Enable Continuous Collision Detection (prevents tunneling through thin objects)
    pub ccd_enabled: bool,
    /// Handle to the rapier rigid body (set by PhysicsWorld)
    pub(crate) handle: Option<rapier2d::dynamics::RigidBodyHandle>,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            gravity_scale: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_rotate: true,
            ccd_enabled: false,
            handle: None,
        }
    }
}

impl RigidBody {
    /// Create a new dynamic rigid body
    pub fn new_dynamic() -> Self {
        Self::default()
    }

    /// Create a new static rigid body
    pub fn new_static() -> Self {
        Self {
            body_type: RigidBodyType::Static,
            ..Default::default()
        }
    }

    /// Create a new kinematic rigid body
    pub fn new_kinematic() -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            ..Default::default()
        }
    }

    /// Set the body type
    pub fn with_body_type(mut self, body_type: RigidBodyType) -> Self {
        self.body_type = body_type;
        self
    }

    /// Set initial velocity
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set initial angular velocity
    pub fn with_angular_velocity(mut self, angular_velocity: f32) -> Self {
        self.angular_velocity = angular_velocity;
        self
    }

    /// Set gravity scale
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Set linear damping
    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping;
        self
    }

    /// Set angular damping
    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping;
        self
    }

    /// Set whether body can rotate
    pub fn with_rotation_locked(mut self, locked: bool) -> Self {
        self.can_rotate = !locked;
        self
    }

    /// Enable Continuous Collision Detection (prevents tunneling through thin objects)
    pub fn with_ccd(mut self, enabled: bool) -> Self {
        self.ccd_enabled = enabled;
        self
    }

    /// Apply an impulse to the center of mass
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.body_type == RigidBodyType::Dynamic {
            self.velocity += impulse;
        }
    }

    /// Apply a force to the center of mass (scaled by delta time)
    pub fn apply_force(&mut self, force: Vec2, delta_time: f32) {
        if self.body_type == RigidBodyType::Dynamic {
            self.velocity += force * delta_time;
        }
    }
}

/// Collider shape types
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// A box with half-extents (width/2, height/2)
    Box { half_extents: Vec2 },
    /// A circle with radius
    Circle { radius: f32 },
    /// A capsule aligned along the Y axis
    CapsuleY { half_height: f32, radius: f32 },
    /// A capsule aligned along the X axis
    CapsuleX { half_height: f32, radius: f32 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Box {
            half_extents: Vec2::new(16.0, 16.0),
        }
    }
}

impl ColliderShape {
    /// Create a box collider
    pub fn box_shape(width: f32, height: f32) -> Self {
        Self::Box {
            half_extents: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    /// Create a circle collider
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    /// Create a vertical capsule collider
    pub fn capsule_y(total_height: f32, radius: f32) -> Self {
        Self::CapsuleY {
            half_height: (total_height - 2.0 * radius).max(0.0) / 2.0,
            radius,
        }
    }

    /// Create a horizontal capsule collider
    pub fn capsule_x(total_width: f32, radius: f32) -> Self {
        Self::CapsuleX {
            half_height: (total_width - 2.0 * radius).max(0.0) / 2.0,
            radius,
        }
    }
}

/// Collider component for collision detection
#[derive(Debug, Clone)]
pub struct Collider {
    /// Shape of the collider
    pub shape: ColliderShape,
    /// Offset from entity position
    pub offset: Vec2,
    /// Whether this collider is a sensor (triggers events but doesn't cause collision response)
    pub is_sensor: bool,
    /// Friction coefficient (0.0 = no friction, 1.0 = high friction)
    pub friction: f32,
    /// Restitution (bounciness, 0.0 = no bounce, 1.0 = perfect bounce)
    pub restitution: f32,
    /// Collision groups (which groups this collider belongs to)
    pub collision_groups: u32,
    /// Collision filter (which groups this collider can collide with)
    pub collision_filter: u32,
    /// Handle to the rapier collider (set by PhysicsWorld)
    pub(crate) handle: Option<rapier2d::geometry::ColliderHandle>,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            offset: Vec2::ZERO,
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
            collision_groups: 0xFFFF_FFFF,
            collision_filter: 0xFFFF_FFFF,
            handle: None,
        }
    }
}

impl Collider {
    /// Create a new collider with the given shape
    pub fn new(shape: ColliderShape) -> Self {
        Self {
            shape,
            ..Default::default()
        }
    }

    /// Create a box collider
    pub fn box_collider(width: f32, height: f32) -> Self {
        Self::new(ColliderShape::box_shape(width, height))
    }

    /// Create a circle collider
    pub fn circle_collider(radius: f32) -> Self {
        Self::new(ColliderShape::circle(radius))
    }

    /// Set the offset
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Set as sensor (no collision response, just detection)
    pub fn as_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    /// Set friction
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    /// Set restitution (bounciness)
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Set collision groups
    pub fn with_collision_groups(mut self, groups: u32, filter: u32) -> Self {
        self.collision_groups = groups;
        self.collision_filter = filter;
        self
    }
}

/// Collision event data
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// First entity in the collision
    pub entity_a: ecs::EntityId,
    /// Second entity in the collision
    pub entity_b: ecs::EntityId,
    /// Whether the collision started this frame
    pub started: bool,
    /// Whether the collision ended this frame
    pub stopped: bool,
}

/// Contact point data for detailed collision information
#[derive(Debug, Clone)]
pub struct ContactPoint {
    /// Contact point in world space
    pub point: Vec2,
    /// Contact normal (pointing from entity_a to entity_b)
    pub normal: Vec2,
    /// Penetration depth
    pub depth: f32,
}

/// Detailed collision data with contact points
#[derive(Debug, Clone)]
pub struct CollisionData {
    /// Basic collision event
    pub event: CollisionEvent,
    /// Contact points (may be empty for sensors)
    pub contacts: Vec<ContactPoint>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_default() {
        let body = RigidBody::default();
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.velocity, Vec2::ZERO);
        assert_eq!(body.gravity_scale, 1.0);
    }

    #[test]
    fn test_rigid_body_builder() {
        let body = RigidBody::new_dynamic()
            .with_velocity(Vec2::new(10.0, 20.0))
            .with_gravity_scale(0.5)
            .with_linear_damping(0.1);

        assert_eq!(body.velocity, Vec2::new(10.0, 20.0));
        assert_eq!(body.gravity_scale, 0.5);
        assert_eq!(body.linear_damping, 0.1);
    }

    #[test]
    fn test_rigid_body_apply_impulse() {
        let mut body = RigidBody::new_dynamic();
        body.apply_impulse(Vec2::new(5.0, 10.0));
        assert_eq!(body.velocity, Vec2::new(5.0, 10.0));

        // Static bodies should not respond to impulses
        let mut static_body = RigidBody::new_static();
        static_body.apply_impulse(Vec2::new(5.0, 10.0));
        assert_eq!(static_body.velocity, Vec2::ZERO);
    }

    #[test]
    fn test_collider_builder() {
        let collider = Collider::box_collider(32.0, 64.0)
            .with_friction(0.8)
            .with_restitution(0.5)
            .as_sensor();

        assert!(collider.is_sensor);
        assert_eq!(collider.friction, 0.8);
        assert_eq!(collider.restitution, 0.5);

        if let ColliderShape::Box { half_extents } = collider.shape {
            assert_eq!(half_extents, Vec2::new(16.0, 32.0));
        } else {
            panic!("Expected box shape");
        }
    }

    #[test]
    fn test_collider_shapes() {
        let circle = ColliderShape::circle(25.0);
        if let ColliderShape::Circle { radius } = circle {
            assert_eq!(radius, 25.0);
        } else {
            panic!("Expected circle shape");
        }

        let capsule = ColliderShape::capsule_y(50.0, 10.0);
        if let ColliderShape::CapsuleY { half_height, radius } = capsule {
            assert_eq!(radius, 10.0);
            assert_eq!(half_height, 15.0); // (50 - 2*10) / 2 = 15
        } else {
            panic!("Expected capsule shape");
        }
    }
}

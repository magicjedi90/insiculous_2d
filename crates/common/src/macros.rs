//! Utility macros to reduce boilerplate.

/// Generate builder-style setter methods for struct fields.
///
/// # Example
/// ```ignore
/// struct Sprite {
///     offset: Vec2,
///     rotation: f32,
///     scale: Vec2,
/// }
///
/// impl Sprite {
///     with_fields! {
///         offset: Vec2,
///         rotation: f32,
///         scale: Vec2,
///     }
/// }
/// ```
#[macro_export]
macro_rules! with_fields {
    ($($field:ident : $type:ty),* $(,)?) => {
        $(
            #[doc = concat!("Set the `", stringify!($field), "` field (builder pattern).")]
            #[inline]
            pub fn $field(mut self, value: $type) -> Self {
                self.$field = value;
                self
            }
        )*
    };
}

/// Generate builder-style setter methods with `with_` prefix.
///
/// # Example
/// ```ignore
/// impl Sprite {
///     with_prefixed_fields! {
///         offset: Vec2,
///         rotation: f32,
///     }
/// }
/// // Generates: with_offset(Vec2), with_rotation(f32)
/// ```
#[macro_export]
macro_rules! with_prefixed_fields {
    ($($field:ident : $type:ty),* $(,)?) => {
        paste::paste! {
            $(
                #[doc = concat!("Set the `", stringify!($field), "` field (builder pattern).")]
                #[inline]
                pub fn [<with_ $field>](mut self, value: $type) -> Self {
                    self.$field = value;
                    self
                }
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    struct TestStruct {
        position: Vec2,
        scale: f32,
    }

    impl TestStruct {
        fn new() -> Self {
            Self {
                position: Vec2::ZERO,
                scale: 1.0,
            }
        }

        with_fields! {
            position: Vec2,
            scale: f32,
        }
    }

    #[test]
    fn test_with_fields_macro() {
        let s = TestStruct::new()
            .position(Vec2::new(10.0, 20.0))
            .scale(2.0);

        assert_eq!(s.position, Vec2::new(10.0, 20.0));
        assert_eq!(s.scale, 2.0);
    }
}

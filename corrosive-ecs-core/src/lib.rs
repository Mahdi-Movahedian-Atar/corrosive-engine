mod arch;
#[cfg(feature = "build")]
/// Used to create the engine at compile time.
pub mod build;
mod hierarchy;
mod locked;
mod locked_ref;
mod r_arch;
mod r#ref;
mod res;
mod reset;
mod signal;
mod state;
mod trigger;

/// Core functions for Corrosive ECS
#[cfg(feature = "core")]
pub mod ecs_core {

    pub use crate::arch::*;
    pub use crate::hierarchy::*;
    pub use crate::locked::*;
    pub use crate::locked_ref::*;
    pub use crate::r#ref::*;
    pub use crate::r_arch::*;
    pub use crate::res::*;
    pub use crate::reset::*;
    pub use crate::signal::*;
    pub use crate::state::*;
    pub use crate::trigger::*;

    /// A reference to a value that may or may not be expired.
    /// Values that use `Locked`,`LockedRef`, `Ref` or `Member` use this to hold their values.
    /// Removing these values will be `Expired`.
    #[derive(Debug)]
    pub enum Reference<T> {
        Some(T),
        Expired,
    }

    /// Input delta tile values use as input to tasks.
    pub type DeltaTime<'a> = &'a f64;

    /// Marks a component for using a task.
    #[macro_export]
    macro_rules! trait_for {
        (trait $e:ty => $($z:ty),+ ) => {};
    }
}

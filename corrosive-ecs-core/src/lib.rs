mod arch;
#[cfg(feature = "build")]
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

    #[derive(Debug)]
    pub enum Reference<T> {
        Some(T),
        Expired,
    }

    pub type DeltaTime<'a> = &'a f64;

    #[macro_export]
    macro_rules! trait_for {
        (trait $e:ty => $($z:ty),+ ) => {};
    }
}

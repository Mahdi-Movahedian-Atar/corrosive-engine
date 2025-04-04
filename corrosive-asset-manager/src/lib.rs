pub mod comp;

use corrosive_ecs_core_macro::corrosive_engine_builder;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

corrosive_engine_builder!();

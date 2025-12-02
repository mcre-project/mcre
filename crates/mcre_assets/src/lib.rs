#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod blockstates;
mod id;
mod models;

pub use blockstates::*;
pub use id::*;
pub use models::*;

use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;

pub(crate) type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

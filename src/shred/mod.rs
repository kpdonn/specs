//! **Sh**ared **re**source **d**ispatcher
//!
//! This library allows to dispatch
//! systems, which can have interdependencies,
//! shared and exclusive resource access, in parallel.
//!
//! # Examples
//!
//! ```rust
//! extern crate shred;
//! #[macro_use]
//! extern crate shred_derive;
//!
//! use shred::{DispatcherBuilder, Read, Resource, Resources, System, Write};
//!
//! #[derive(Debug, Default)]
//! struct ResA;
//!
//! #[derive(Debug, Default)]
//! struct ResB;
//!
//! #[derive(SystemData)]
//! struct Data<'a> {
//!     a: Read<'a, ResA>,
//!     b: Write<'a, ResB>,
//! }
//!
//! struct EmptySystem;
//!
//! impl<'a> System<'a> for EmptySystem {
//!     type SystemData = Data<'a>;
//!
//!     fn run(&mut self, bundle: Data<'a>) {
//!         println!("{:?}", &*bundle.a);
//!         println!("{:?}", &*bundle.b);
//!     }
//! }
//!
//!
//! fn main() {
//!     let mut resources = Resources::new();
//!     let mut dispatcher = DispatcherBuilder::new()
//!         .with(EmptySystem, "empty", &[])
//!         .build();
//!     resources.insert(ResA);
//!     resources.insert(ResB);
//!
//!     dispatcher.dispatch(&mut resources);
//! }
//! ```
//!
//! Once you are more familiar with how system data and parallelization works,
//! you can take look at a more flexible and performant way to dispatch: `ParSeq`.
//! Using it is bit trickier, but it allows dispatching without any virtual function calls.
//!

pub mod cell;

mod dispatch;
mod meta;
pub mod on_change_system;
mod res;
mod system;
pub use self::dispatch::{Dispatcher, DispatcherBuilder};
pub use self::on_change_system::*;

pub use self::meta::{CastFrom, MetaIter, MetaIterMut, MetaTable};
pub use self::res::{
    DefaultProvider, Entry, Fetch, FetchMut, PanicHandler, Read, ReadExpect, Resource, ResourceId,
    Resources, SetupHandler, Write, WriteExpect,
};
pub use self::system::{
    Accessor, AccessorCow, DynamicSystemData, RunNow, RunningTime, StaticAccessor, System,
    SystemData,
};

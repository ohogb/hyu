mod client;
mod compositor;
mod display;
mod registry;
mod shm;
mod sub_compositor;

pub use client::*;
pub use compositor::*;
pub use display::*;
pub use registry::*;
pub use shm::*;
pub use sub_compositor::*;

use crate::{wl, Result};

pub trait Global: std::fmt::Debug {
	fn get_name(&self) -> &'static str;
	fn get_version(&self) -> u32;
	fn bind(&self, client: &mut wl::Client, object_id: u32);
}

pub trait Object {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()>;
}

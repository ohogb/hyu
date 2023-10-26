mod client;
mod compositor;
mod data_device;
mod data_device_manager;
mod display;
mod output;
mod registry;
mod seat;
mod shm;
mod sub_compositor;
mod xdg_wm_base;

pub use client::*;
pub use compositor::*;
pub use data_device::*;
pub use data_device_manager::*;
pub use display::*;
pub use output::*;
pub use registry::*;
pub use seat::*;
pub use shm::*;
pub use sub_compositor::*;
pub use xdg_wm_base::*;

use crate::{wl, Result};

pub trait Global: std::fmt::Debug {
	fn get_name(&self) -> &'static str;
	fn get_version(&self) -> u32;
	fn bind(&self, client: &mut wl::Client, object_id: u32);
}

pub trait Object {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()>;
}

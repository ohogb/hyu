mod buffer;
mod callback;
mod client;
mod compositor;
mod data_device;
mod data_device_manager;
mod display;
mod id;
mod keyboard;
mod output;
mod pointer;
mod region;
mod registry;
mod resource;
mod seat;
mod shm;
mod shm_pool;
mod sub_compositor;
mod sub_surface;
mod surface;
mod xdg_positioner;
mod xdg_surface;
mod xdg_toplevel;
mod xdg_wm_base;
mod zwp_linux_buffer_params_v1;
mod zwp_linux_dmabuf_feedback_v1;
mod zwp_linux_dmabuf_v1;

pub use buffer::*;
pub use callback::*;
pub use client::*;
pub use compositor::*;
pub use data_device::*;
pub use data_device_manager::*;
pub use display::*;
pub use id::*;
pub use keyboard::*;
pub use output::*;
pub use pointer::*;
pub use region::*;
pub use registry::*;
pub use resource::*;
pub use seat::*;
pub use shm::*;
pub use shm_pool::*;
pub use sub_compositor::*;
pub use sub_surface::*;
pub use surface::*;
pub use xdg_positioner::*;
pub use xdg_surface::*;
pub use xdg_toplevel::*;
pub use xdg_wm_base::*;
pub use zwp_linux_buffer_params_v1::*;
pub use zwp_linux_dmabuf_feedback_v1::*;
pub use zwp_linux_dmabuf_v1::*;

use crate::{wl, Result};

pub trait Global {
	fn get_name(&self) -> &'static str;
	fn get_version(&self) -> u32;
	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()>;
}

pub trait Object {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()>;
}

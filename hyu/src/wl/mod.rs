mod buffer;
mod callback;
mod compositor;
mod data_device;
mod data_device_manager;
mod data_source;
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
mod wp_presentation;
mod wp_presentation_feedback;
mod xdg_popup;
mod xdg_positioner;
mod xdg_surface;
mod xdg_toplevel;
mod xdg_wm_base;
mod zwp_linux_buffer_params_v1;
mod zwp_linux_dmabuf_feedback_v1;
mod zwp_linux_dmabuf_v1;
mod zxdg_output_manager_v1;
mod zxdg_output_v1;

pub use buffer::*;
pub use callback::*;
pub use compositor::*;
pub use data_device::*;
pub use data_device_manager::*;
pub use data_source::*;
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
pub use wp_presentation::*;
pub use wp_presentation_feedback::*;
pub use xdg_popup::*;
pub use xdg_positioner::*;
pub use xdg_surface::*;
pub use xdg_toplevel::*;
pub use xdg_wm_base::*;
pub use zwp_linux_buffer_params_v1::*;
pub use zwp_linux_dmabuf_feedback_v1::*;
pub use zwp_linux_dmabuf_v1::*;
pub use zxdg_output_manager_v1::*;
pub use zxdg_output_v1::*;

use crate::{Client, Result};

pub trait Global {
	fn get_name(&self) -> &'static str;
	fn get_version(&self) -> u32;
	fn bind(&self, client: &mut Client, object_id: u32, version: u32) -> Result<()>;
}

pub trait Object {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()>;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 2],
	// pub color: [f32; 4],
	pub uv: [f32; 2],
}

pub struct State {
	pub buffer: Buffer,
	pub start_position: (i32, i32),
}

pub struct Buffer(pub Vec<u8>);

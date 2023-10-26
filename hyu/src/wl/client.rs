use crate::{wl, State};

pub struct Client {
	objects: std::collections::HashMap<u32, Box<dyn wl::Object>>,
	state: State,
}

impl Client {
	pub fn new(state: State) -> Self {
		Self {
			objects: std::collections::HashMap::new(),
			state,
		}
	}

	pub fn push_client_object(&mut self, id: u32, object: impl wl::Object + 'static) {
		self.objects.insert(id, Box::new(object));
	}

	pub fn get_object_mut(&mut self, id: u32) -> Option<&mut Box<dyn wl::Object>> {
		self.objects.get_mut(&id)
	}

	pub fn get_state(&mut self) -> &mut State {
		&mut self.state
	}
}

use crate::{wl, State};

pub struct Client {
	objects: std::collections::HashMap<u32, std::rc::Rc<dyn wl::Object>>,
	state: State,
}

impl Client {
	pub fn new(state: State) -> Self {
		Self {
			objects: std::collections::HashMap::new(),
			state,
		}
	}

	pub fn push_client_object(&mut self, id: u32, object: std::rc::Rc<dyn wl::Object>) {
		self.objects.insert(id, object);
	}

	pub fn get_object(&self, id: u32) -> Option<&std::rc::Rc<dyn wl::Object>> {
		self.objects.get(&id)
	}

	pub fn get_state(&mut self) -> &mut State {
		&mut self.state
	}
}

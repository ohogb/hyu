use crate::{egl, Result};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Context {
	ptr: std::num::NonZeroU64,
}

impl Context {
	pub fn access<'display>(
		&mut self,
		display: &'display egl::Display,
		surface: Option<&egl::Surface>,
	) -> Result<ContextHolder<'_, 'display>> {
		display.make_current(surface, Some(self))?;

		Ok(ContextHolder {
			_context: std::marker::PhantomData,
			display,
		})
	}

	pub fn as_ptr(&self) -> u64 {
		self.ptr.get()
	}
}

pub struct ContextHolder<'context, 'display> {
	_context: std::marker::PhantomData<&'context ()>,
	display: &'display egl::Display,
}

impl Drop for ContextHolder<'_, '_> {
	fn drop(&mut self) {
		self.display.make_current(None, None).unwrap()
	}
}

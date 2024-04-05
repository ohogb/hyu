pub struct Id<T> {
	pub id: u32,
	_phantom: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
	pub const fn new(id: u32) -> Self {
		Self {
			id,
			_phantom: std::marker::PhantomData,
		}
	}

	pub fn null() -> Self {
		Self::new(0)
	}

	pub fn is_null(&self) -> bool {
		self.id == 0
	}
}

impl<T> std::ops::Deref for Id<T> {
	type Target = u32;

	fn deref(&self) -> &Self::Target {
		&self.id
	}
}

impl<T> Clone for Id<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl<T> serde::Serialize for Id<T> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u32(self.id)
	}
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
	type Value = u32;

	fn visit_u32<E: serde::de::Error>(self, value: u32) -> std::result::Result<Self::Value, E> {
		Ok(value)
	}

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("an u32")
	}
}

impl<'de, T> serde::Deserialize<'de> for Id<T> {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Ok(Self::new(deserializer.deserialize_u32(Visitor)?))
	}
}

unsafe impl<T> Send for Id<T> {}
unsafe impl<T> Sync for Id<T> {}

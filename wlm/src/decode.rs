use crate::error::{Error, Result};

pub fn from_slice<'a, T: serde::Deserialize<'a>>(input: &'a [u8]) -> Result<T> {
	let mut deserializer = Deserializer { input };

	let ret = T::deserialize(&mut deserializer)?;

	if deserializer.input.is_empty() {
		Ok(ret)
	} else {
		Err(Error::Message("TrailingData".to_string()))
	}
}

pub struct Deserializer<'de> {
	input: &'de [u8],
}

impl<'de> Deserializer<'de> {}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_bool<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_i8<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_i16<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let value = i32::from_ne_bytes(self.input[..4].try_into()?);
		self.input = &self.input[4..];
		visitor.visit_i32(value)
	}

	fn deserialize_i64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_u8<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_u16<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let value = u32::from_ne_bytes(self.input[..4].try_into()?);
		self.input = &self.input[4..];
		visitor.visit_u32(value)
	}

	fn deserialize_u64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_f32<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_f64<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_char<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_str<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let size = u32::from_ne_bytes(self.input[..4].try_into()?);
		self.input = &self.input[4..];

		let value = String::from(std::str::from_utf8(&self.input[..size as usize - 1]).unwrap());
		self.input = &self.input[size as _..];

		if size % 4 != 0 {
			self.input = &self.input[(4 - (size % 4)) as _..];
		}

		visitor.visit_string(value)
	}

	fn deserialize_bytes<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_byte_buf<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_option<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_unit<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		_visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		_visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_seq<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_tuple<V>(
		self,
		_len: usize,
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_seq(self)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		_visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_map<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		_visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		_visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_identifier<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		todo!()
	}
}

impl<'de, 'a> serde::de::SeqAccess<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(&mut **self).map(Some)
	}
}

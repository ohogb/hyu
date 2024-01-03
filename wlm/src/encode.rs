use crate::error::{Error, Result};

pub struct Serializer {
	output: Vec<u8>,
}

pub fn to_vec(value: &impl serde::Serialize) -> Result<Vec<u8>> {
	let mut serializer = Serializer { output: Vec::new() };
	value.serialize(&mut serializer)?;

	Ok(serializer.output)
}

impl<'a> serde::ser::Serializer for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	type SerializeSeq = Self;

	type SerializeTuple = Self;

	type SerializeTupleStruct = Self;

	type SerializeTupleVariant = Self;

	type SerializeMap = Self;

	type SerializeStruct = Self;

	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<()> {
		todo!()
	}

	fn serialize_i8(self, v: i8) -> Result<()> {
		todo!()
	}

	fn serialize_i16(self, v: i16) -> Result<()> {
		todo!()
	}

	fn serialize_i32(self, v: i32) -> Result<()> {
		self.output.extend(v.to_ne_bytes());
		Ok(())
	}

	fn serialize_i64(self, v: i64) -> Result<()> {
		todo!()
	}

	fn serialize_u8(self, v: u8) -> Result<()> {
		todo!()
	}

	fn serialize_u16(self, v: u16) -> Result<()> {
		todo!()
	}

	fn serialize_u32(self, v: u32) -> Result<()> {
		self.output.extend(v.to_ne_bytes());
		Ok(())
	}

	fn serialize_u64(self, v: u64) -> Result<()> {
		todo!()
	}

	fn serialize_f32(self, v: f32) -> Result<()> {
		todo!()
	}

	fn serialize_f64(self, v: f64) -> Result<()> {
		todo!()
	}

	fn serialize_char(self, v: char) -> Result<()> {
		todo!()
	}

	fn serialize_str(self, v: &str) -> Result<()> {
		let size = v.len() as u32 + 1;

		self.output.extend(size.to_ne_bytes());

		self.output.extend(v.as_bytes());
		self.output.push(0);

		if size % 4 != 0 {
			self.output.extend((0..(4 - (size % 4))).map(|_| 0u8));
		}

		Ok(())
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<()> {
		todo!()
	}

	fn serialize_none(self) -> Result<()> {
		todo!()
	}

	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn serialize_unit(self) -> Result<()> {
		Ok(())
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
		todo!()
	}

	fn serialize_unit_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<()> {
		todo!()
	}

	fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn serialize_newtype_variant<T: ?Sized>(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn serialize_seq(
		self,
		len: Option<usize>,
	) -> std::result::Result<Self::SerializeSeq, Self::Error> {
		todo!()
	}

	fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
		Ok(self)
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		len: usize,
	) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
		todo!()
	}

	fn serialize_tuple_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
		todo!()
	}

	fn serialize_map(
		self,
		len: Option<usize>,
	) -> std::result::Result<Self::SerializeMap, Self::Error> {
		todo!()
	}

	fn serialize_struct(
		self,
		name: &'static str,
		len: usize,
	) -> std::result::Result<Self::SerializeStruct, Self::Error> {
		todo!()
	}

	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
		todo!()
	}
}

impl<'a> serde::ser::SerializeSeq for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}
impl<'a> serde::ser::SerializeTuple for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'a> serde::ser::SerializeMap for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'a> serde::ser::SerializeStruct for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut Serializer {
	type Ok = ();

	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		todo!()
	}

	fn end(self) -> Result<()> {
		todo!()
	}
}

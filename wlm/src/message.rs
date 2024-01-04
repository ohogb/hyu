use std::io::Write as _;

pub struct Message<T> {
	pub object_id: u32,
	pub op: u16,
	pub args: T,
}

impl<T: serde::Serialize> Message<T> {
	pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
		let mut ret = Vec::new();

		ret.write_all(&self.object_id.to_ne_bytes())?;
		ret.write_all(&self.op.to_ne_bytes())?;

		let args = crate::encode::to_vec(&self.args)?;

		ret.write_all(&(8u16 + args.len() as u16).to_ne_bytes())?;
		ret.extend(args);

		Ok(ret)
	}
}

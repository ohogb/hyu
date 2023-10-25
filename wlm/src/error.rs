pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Message(String),
}

impl serde::ser::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: std::fmt::Display,
	{
		Self::Message(msg.to_string())
	}
}

impl serde::de::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: std::fmt::Display,
	{
		Self::Message(msg.to_string())
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

impl std::error::Error for Error {}

impl From<std::array::TryFromSliceError> for Error {
	fn from(value: std::array::TryFromSliceError) -> Self {
		Self::Message(value.to_string())
	}
}

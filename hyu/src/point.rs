#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Point(pub i32, pub i32);

impl Point {
	pub fn mul_f32(&self, factor: (f32, f32)) -> Self {
		Self(
			(self.0 as f32 * factor.0) as i32,
			(self.1 as f32 * factor.1) as i32,
		)
	}
}

impl std::ops::Add for Point {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0, self.1 + rhs.1)
	}
}

impl std::ops::Sub for Point {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0, self.1 - rhs.1)
	}
}

impl std::ops::Mul for Point {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self(self.0 * rhs.0, self.1 * rhs.1)
	}
}

impl std::ops::AddAssign for Point {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
		self.1 += rhs.1;
	}
}

impl std::ops::SubAssign for Point {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
		self.1 -= rhs.1;
	}
}

impl std::ops::MulAssign for Point {
	fn mul_assign(&mut self, rhs: Self) {
		self.0 *= rhs.0;
		self.1 *= rhs.1;
	}
}

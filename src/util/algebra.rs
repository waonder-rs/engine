use std::ops::{Mul, Add};

pub trait MonoidMul: Sized + Mul<Output = Self> {
	const ONE: Self;
}

impl MonoidMul for u8 {
	const ONE: u8 = 1;
}

impl MonoidMul for u16 {
	const ONE: u16 = 1;
}

impl MonoidMul for u32 {
	const ONE: u32 = 1;
}

impl MonoidMul for u64 {
	const ONE: u64 = 1;
}

impl MonoidMul for i8 {
	const ONE: i8 = 1;
}

impl MonoidMul for i16 {
	const ONE: i16 = 1;
}

impl MonoidMul for i32 {
	const ONE: i32 = 1;
}

impl MonoidMul for i64 {
	const ONE: i64 = 1;
}

impl MonoidMul for f32 {
	const ONE: f32 = 1.0;
}

impl MonoidMul for f64 {
	const ONE: f64 = 1.0;
}

pub trait MonoidAdd: Sized + Add<Output = Self> {
	const ZERO: Self;
}

impl MonoidAdd for u8 {
	const ZERO: u8 = 0;
}

impl MonoidAdd for u16 {
	const ZERO: u16 = 0;
}

impl MonoidAdd for u32 {
	const ZERO: u32 = 0;
}

impl MonoidAdd for u64 {
	const ZERO: u64 = 0;
}

impl MonoidAdd for i8 {
	const ZERO: i8 = 0;
}

impl MonoidAdd for i16 {
	const ZERO: i16 = 0;
}

impl MonoidAdd for i32 {
	const ZERO: i32 = 0;
}

impl MonoidAdd for i64 {
	const ZERO: i64 = 0;
}

impl MonoidAdd for f32 {
	const ZERO: f32 = 0.0;
}

impl MonoidAdd for f64 {
	const ZERO: f64 = 0.0;
}

// 2d norm 2.
pub trait Norm: Sized {
	type Output;

	fn norm(vector: &[Self]) -> Self::Output;
}

impl Norm for f32 {
	type Output = f32;

	fn norm(v: &[f32]) -> f32 {
		let mut n = 0.0;
		for x in v.iter() {
			n += x*x;
		}

		n.sqrt()
	}
}

impl Norm for f64 {
	type Output = f64;

	fn norm(v: &[f64]) -> f64 {
		let mut n = 0.0;
		for x in v.iter() {
			n += x*x;
		}

		n.sqrt()
	}
}

/// Type on which the trigonometrics operation are defined.
pub trait Trigonometric: Sized {
	type Output;

	const PI: Self;

	fn sin(&self) -> Self::Output;

	fn cos(&self) -> Self::Output;

	fn tan(&self) -> Self::Output;

	fn asin(x: Self::Output) -> Self;

	fn acos(x: Self::Output) -> Self;

	fn atan(x: Self::Output) -> Self;
}

impl Trigonometric for f32 {
	type Output = f32;

	const PI: f32 = 3.1415;

	fn sin(&self) -> f32 {
		f32::sin(*self)
	}

	fn cos(&self) -> f32 {
		f32::cos(*self)
	}

	fn tan(&self) -> f32 {
		f32::tan(*self)
	}

	fn asin(f: f32) -> f32 {
		f32::asin(f)
	}

	fn acos(f: f32) -> f32 {
		f32::acos(f)
	}

	fn atan(f: f32) -> f32 {
		f32::atan(f)
	}
}

impl Trigonometric for f64 {
	type Output = f64;

	const PI: f64 = 3.1415;

	fn sin(&self) -> f64 {
		f64::sin(*self)
	}

	fn cos(&self) -> f64 {
		f64::cos(*self)
	}

	fn tan(&self) -> f64 {
		f64::tan(*self)
	}

	fn asin(f: f64) -> f64 {
		f64::asin(f)
	}

	fn acos(f: f64) -> f64 {
		f64::acos(f)
	}

	fn atan(f: f64) -> f64 {
		f64::atan(f)
	}
}

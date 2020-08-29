use std::ops::{Div, Mul, Add, Sub};
use super::Norm;

#[repr(packed)]
pub struct Vector2D<T: Copy> {
	pub x: T,
	pub y: T
}

pub type Vertex2D<T> = Vector2D<T>;

impl<T: Copy> Vector2D<T> {
	pub fn new(x: T, y: T) -> Vector2D<T> {
		Vector2D {
			x: x,
			y: y
		}
	}

	pub fn x(&self) -> T {
		self.x
	}

	pub fn y(&self) -> T {
		self.y
	}

	pub fn unit(x: T, y: T) -> Vector2D<T> where T: Copy + Norm<Output = T> + Div<Output = T> {
		let v = Vector2D::new(x, y);
		v.normal()
	}

	pub fn len<D>(&self) -> D where T: Norm<Output = D> {
		T::norm(&[self.x, self.y])
	}

	pub fn normal(&self) -> Vector2D<T> where T: Copy + Norm<Output = T> + Div<Output = T> {
		let l = self.len();
		Vector2D {
			x: self.x/l,
			y: self.y/l
		}
	}

	pub fn scalar(self, other: Self) -> T where T: Mul<Output = T> + Add<Output = T> {
		self.x * other.x + self.y * other.y
	}
}

impl<T: Copy> Clone for Vector2D<T> {
	fn clone(&self) -> Vector2D<T> {
		Vector2D {
			x: self.x(),
			y: self.y()
		}
	}
}

impl<T: Copy> Copy for Vector2D<T> { }

impl<T: Copy + Default> Default for Vector2D<T> {
	fn default() -> Vector2D<T> {
		Vector2D {
			x: T::default(),
			y: T::default()
		}
	}
}

impl<T: Copy + PartialEq> PartialEq for Vector2D<T> {
	fn eq(&self, other: &Vector2D<T>) -> bool {
		let x = self.x;
		let y = self.y;
		let other_x = other.x;
		let other_y = other.y;
		x == other_x && y == other_y
	}
}

impl<F: Sub, T: Copy + Mul<Output = F>> Mul for Vector2D<T> {
	type Output = F::Output;

	fn mul(self, other: Self) -> F::Output {
		self.x * other.y - self.y * other.x
	}
}

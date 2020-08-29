use std::ops::{Div, Mul, Add, Sub};
use super::Norm;

#[repr(packed)]
pub struct Vector3D<T: Copy> {
	pub x: T,
	pub y: T,
	pub z: T
}

pub type Vertex3D<T> = Vector3D<T>;

impl<T: Copy> Vector3D<T> {
	pub fn new(x: T, y: T, z: T) -> Vector3D<T> {
		Vector3D {
			x: x,
			y: y,
			z: z
		}
	}

	pub fn x(&self) -> T {
		self.x
	}

	pub fn y(&self) -> T {
		self.y
	}

	pub fn z(&self) -> T {
		self.z
	}

	pub fn normal(&self) -> Vector3D<T> where T: Copy + Norm<Output = T> + Div<Output = T> {
		let l = self.len();
		Vector3D {
			x: self.x/l,
			y: self.y/l,
			z: self.z/l
		}
	}

	pub fn len<D>(&self) -> D where T: Norm<Output = D> {
		T::norm(&[self.x, self.y, self.z])
	}

	pub fn scalar(self, other: Self) -> T where T: Mul<Output = T> + Add<Output = T> {
		self.x * other.x + self.y * other.y + self.z + other.z
	}
}

impl<T: Copy + Default> Default for Vector3D<T> {
	fn default() -> Vector3D<T> {
		Vector3D {
			x: T::default(),
			y: T::default(),
			z: T::default()
		}
	}
}

impl<T: Copy> Clone for Vector3D<T> {
	fn clone(&self) -> Vector3D<T> {
		Vector3D {
			x: self.x(),
			y: self.y(),
			z: self.z()
		}
	}
}

impl<T: Copy> Copy for Vector3D<T> { }

impl<T: Copy + PartialEq> PartialEq for Vector3D<T> {
	fn eq(&self, other: &Vector3D<T>) -> bool {
		let x = self.x;
		let y = self.y;
		let z = self.z;
		let other_x = other.x;
		let other_y = other.y;
		let other_z = other.z;
		x == other_x && y == other_y && z == other_z
	}
}

impl<T: Copy + Add> Add for Vector3D<T> where T::Output: Copy {
	type Output = Vector3D<T::Output>;

	fn add(self, other: Self) -> Vector3D<T::Output> {
		Vector3D {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z
		}
	}
}

impl<F: Sub, T: Copy + Mul<Output = F>> Mul for Vector3D<T> where F::Output: Copy {
	type Output = Vector3D<F::Output>;

	fn mul(self, other: Self) -> Vector3D<F::Output> {
		Vector3D {
			x: self.y * other.z - self.z * other.y,
			y: self.z * other.x - self.x * other.z,
			z: self.x * other.y - self.y * other.x
		}
	}
}

impl<T: Copy + Mul> Mul<T> for Vector3D<T> where T::Output: Copy {
	type Output = Vector3D<T::Output>;

	fn mul(self, f: T) -> Vector3D<T::Output> {
		Vector3D {
			x: self.x * f,
			y: self.y * f,
			z: self.z * f
		}
	}
}

impl<T: Copy + Div> Div<T> for Vector3D<T> where T::Output: Copy {
	type Output = Vector3D<T::Output>;

	fn div(self, f: T) -> Vector3D<T::Output> {
		Vector3D {
			x: self.x / f,
			y: self.y / f,
			z: self.z / f
		}
	}
}

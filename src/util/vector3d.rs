use std::fmt;
use std::ops::{Div, Mul, Add, Sub, Neg};
use super::Sqrt;

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

	pub fn normal(self) -> Vector3D<T> where T: Add<Output=T> + Sqrt<Output = T> + Mul<Output=T> + Div<Output = T> {
		let l = self.len();
		Vector3D {
			x: self.x/l,
			y: self.y/l,
			z: self.z/l
		}
	}

	pub fn normalize(&mut self) where T: Add<Output=T> + Sqrt<Output = T> + Mul<Output=T> + Div<Output = T> {
		let l = self.len();
		self.x = self.x / l;
		self.y = self.y / l;
		self.z = self.z / l;
	}

	pub fn cross_product<D: Copy>(self, other: Vector3D<T>) -> Vector3D<D> where T: Mul, T::Output: Sub<Output=D> {
		Vector3D {
			x: self.y * other.z - self.z * other.y,
			y: self.z * other.x - self.x * other.z,
			z: self.x * other.y - self.y * other.x
		}
	}

	pub fn len(&self) -> T where T: Add<Output=T> + Mul<Output=T> + Sqrt<Output=T> {
		(self.x * self.x + self.y * self.y + self.z * self.y).sqrt()
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

impl<T: Copy + Sub> Sub for Vector3D<T> where T::Output: Copy {
	type Output = Vector3D<T::Output>;

	fn sub(self, other: Self) -> Vector3D<T::Output> {
		Vector3D {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z
		}
	}
}

impl<T: Copy + Neg> Neg for Vector3D<T> where T::Output: Copy {
	type Output = Vector3D<T::Output>;

	fn neg(self) -> Vector3D<T::Output> {
		Vector3D {
			x: -self.x,
			y: -self.y,
			z: -self.z
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

impl<T: Copy + fmt::Display> fmt::Display for Vector3D<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {}, {})", self.x, self.y, self.z)
	}
}

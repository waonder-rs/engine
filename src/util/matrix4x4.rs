use std::fmt;
use std::ops::{
	Add,
	Sub,
	Mul,
	MulAssign,
	Div,
	Index
};
use super::{
	Vector3D,
	Vector4D,
	Tensor2d,
	MonoidMul,
	MonoidAdd,
	Sqrt,
	Trigonometric
};

macro_rules! transposed {
	( $a:expr, $b:expr, $c:expr, $d:expr,
	  $e:expr, $f:expr, $g:expr, $h:expr,
	  $i:expr, $j:expr, $k:expr, $l:expr,
	  $m:expr, $n:expr, $o:expr, $p:expr ) => {
		[
			$a, $e, $i, $m,
			$b, $f, $j, $n,
			$c, $g, $k, $o,
			$d, $h, $l, $p
		]
	};
}

/// 4x4 matrix.
///
/// Matrix is stored in column-major format, just as GLSL matrices.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix4x4<T>([T; 4*4]);

unsafe impl<T> Tensor2d for Matrix4x4<T> {
	const DIM: (usize, usize) = (4, 4);
}

impl<T> Matrix4x4<T> {
	pub fn identity() -> Matrix4x4<T> where T: MonoidMul + MonoidAdd {
		Matrix4x4([
			T::ONE, T::ZERO, T::ZERO, T::ZERO,
			T::ZERO, T::ONE, T::ZERO, T::ZERO,
			T::ZERO, T::ZERO, T::ONE, T::ZERO,
			T::ZERO, T::ZERO, T::ZERO, T::ONE
		])
	}

	pub fn orthographic(
		left: T, right: T,
		bottom: T, top: T,
		near: T, far: T
	) -> Matrix4x4<T> where T: Copy + MonoidMul + MonoidAdd + Sub<Output = T> + Div<Output = T> {
		let dx = right - left;
		let dy = top - bottom;
		let dz = far - near;

		let tx = T::ZERO - (right + left)/dx;
		let ty = T::ZERO - (top + bottom)/dy;
		let tz = T::ZERO - (far + near)/dz;

		Matrix4x4(transposed![
			(T::ONE + T::ONE)/(right - left), T::ZERO, T::ZERO, tx,
			T::ZERO, (T::ONE + T::ONE)/(top - bottom), T::ZERO, ty,
			T::ZERO, T::ZERO, (T::ONE + T::ONE)/(far - near), tz,
			T::ZERO, T::ZERO, T::ZERO, T::ONE
		])
	}

	pub fn perspective(
		left: T, right: T,
		bottom: T, top: T,
		near: T, far: T
	) -> Matrix4x4<T> where T: Copy + MonoidMul + MonoidAdd + Sub<Output = T> + Div<Output = T> {
		Matrix4x4(transposed![
			(near + near) / (right - left), T::ZERO, (right + left) / (right - left), T::ZERO,
			T::ZERO, (near + near) / (top - bottom), (top + bottom) / (top - bottom), T::ZERO,
			T::ZERO, T::ZERO, (T::ONE - far - near) / (far - near), (T::ZERO - far - far) * near / (far - near),
			T::ZERO, T::ZERO, T::ZERO - T::ONE, T::ZERO
		])
	}

	pub fn fovx_perspective(fovx: T, aspect: T, near: T, far: T) -> Matrix4x4<T> where T: Copy + Trigonometric<Output = T> + MonoidMul + MonoidAdd + Sub<Output = T> + Div<Output = T> {
		let right = near * (fovx / (T::ONE + T::ONE)).tan();
		let top = right / aspect;

		Matrix4x4::perspective(T::ZERO-right, right, T::ZERO-top, top, near, far)
	}

	pub fn looking_at(mut target: Vector3D<T>, up: Vector3D<T>) -> Matrix4x4<T> where T: Copy + MonoidAdd + Sub<Output=T> + MonoidMul + Div<Output=T> + Sqrt<Output=T> {
		target.normalize();
		let mut forward = Vector3D::new(T::ZERO - target.x, T::ZERO - target.y, T::ZERO - target.z);

		let side = Vector3D::cross_product(forward, up);
		let up = Vector3D::cross_product(side, forward);

		Matrix4x4(transposed![
			side.x, side.y, side.z, T::ZERO,
			up.x, up.y, up.z, T::ZERO,
			target.x, target.y, target.z, T::ZERO,
			T::ZERO, T::ZERO, T::ZERO, T::ONE
		])
	}

	pub fn translation(pos: Vector3D<T>) -> Matrix4x4<T> where T: Copy + MonoidMul + MonoidAdd {
		Matrix4x4(transposed![
			T::ONE, T::ZERO, T::ZERO, pos.x,
			T::ZERO, T::ONE, T::ZERO, pos.y,
			T::ZERO, T::ZERO, T::ONE, pos.z,
			T::ZERO, T::ZERO, T::ZERO, T::ONE
		])
	}

	pub fn translate(&mut self, pos: Vector3D<T>) -> &mut Self where T: Copy + Add<Output=T> + Mul<Output=T> {
		let m = self.0[3];
		let n = self.0[7];
		let o = self.0[11];
		let p = self.0[15];

		self.0[0] = self.0[0] + (pos.x * m);
		self.0[1] = self.0[1] + (pos.y * m);
		self.0[2] = self.0[2] + (pos.z * m);

		self.0[4] = self.0[4] + (pos.x * n);
		self.0[5] = self.0[5] + (pos.y * n);
		self.0[6] = self.0[6] + (pos.z * n);

		self.0[8] = self.0[8] + (pos.x * o);
		self.0[9] = self.0[9] + (pos.y * o);
		self.0[10] = self.0[10] + (pos.z * o);

		self.0[12] = self.0[12] + (pos.x * p);
		self.0[13] = self.0[13] + (pos.y * p);
		self.0[14] = self.0[14] + (pos.z * p);

		self
	}

	/// Rotation aroung the given axis.
	///
	/// The angle is given in radians.
	pub fn rotation(angle: T, axis: Vector3D<T>) -> Matrix4x4<T> where T: Copy + Trigonometric<Output=T> + MonoidAdd + Sub<Output=T> + MonoidMul {
		let c = angle.cos();
		let oc = T::ONE - c;
		let s = angle.sin();

		Matrix4x4(transposed![
			axis.x*axis.x*oc + c       , axis.x*axis.y*oc - axis.z*s, axis.x*axis.z*oc + axis.y*s, T::ZERO,
			axis.y*axis.x*oc + axis.z*s, axis.y*axis.y*oc + c       , axis.y*axis.z*oc - axis.x*s, T::ZERO,
			axis.z*axis.x*oc - axis.y*s, axis.z*axis.y*oc + axis.x*s, axis.z*axis.z*oc + c       , T::ZERO,
			T::ZERO                    , T::ZERO                    , T::ZERO                    , T::ONE
		])
	}

	pub fn transposed(&self) -> Self where T: Copy {
		Matrix4x4([
			self.0[0], self.0[4], self.0[8], self.0[12],
			self.0[1], self.0[5], self.0[9], self.0[13],
			self.0[2], self.0[6], self.0[10], self.0[14],
			self.0[3], self.0[7], self.0[11], self.0[15]
		])
	}

	/// Calculate the determinant of a 2x2 sub-matrix.
	///
	/// M = | A B |  det(M) = A * D - B * C
	///     | C D |
	#[inline]
	fn det2(&self, x0: usize, x1: usize, y0: usize, y1: usize) -> T where T: Copy + Sub<Output=T> + Mul<Output=T> {
		self[(x0, y0)] * self[(x1, y1)] - self[(x1, y0)] * self[(x0, y1)]
	}

	/// Calculate the determinant of a 3x3 sub-matrix.
	///
	///     | A B C |
	/// M = | D E F |  det(M) = A * (EI - HF) - B * (DI - GF) + C * (DH - GE)
	///     | G H I |                   | E F |            | D F |            | D E |
	///                det(M) = A * det(| H I |) - B * det(| G I |) + C * det(| G H |)
	#[inline]
	fn det3(&self, x0: usize, x1: usize, x2: usize, y0: usize, y1: usize, y2: usize) -> T where T: Copy + Add<Output=T> + Sub<Output=T> + Mul<Output=T> {
		self[(x0, y0)] * self.det2(x1, x2, y1, y2) -
		self[(x1, y0)] * self.det2(x0, x2, y1, y2) +
		self[(x2, y0)] * self.det2(x0, x1, y1, y2)
	}

	/// Calculate the determinant of the matrix.
	///
	///     | A B C D |                   | F G H |            | E G H |            | E F H |            | E F G |
	/// M = | E F G H |  det(M) = A * det(| J K L |) - B * det(| I K L |) + C * det(| I J L |) - D * det(| I J K |)
	///     | I J K L |                   | N O P |            | M O P |            | M N P |            | M N O |
	///     | M N O P |
	#[inline]
	pub fn determinant(&self) -> T where T: Copy + Add<Output=T> + Sub<Output=T> + Mul<Output=T> {
		self[(0, 0)] * self.det3(1, 2, 3, 1, 2, 3) -
		self[(1, 0)] * self.det3(0, 2, 3, 1, 2, 3) +
		self[(2, 0)] * self.det3(0, 1, 3, 1, 2, 3) -
		self[(3, 0)] * self.det3(0, 1, 2, 1, 2, 3)
	}

	pub fn inverted(&self) -> Option<Self> where T: Copy + MonoidAdd + MonoidMul + Sub<Output=T> + Div<Output=T> + PartialEq {
		let det = self.determinant();

		// check if the matrix is invertible.
		if det == T::ZERO {
			return None; // ... it is not.
		}

		let idet = T::ONE / det;

		Some(Matrix4x4(transposed![
			self.det3(1, 2, 3, 1, 2, 3) * idet,
			T::ZERO - self.det3(1, 2, 3, 0, 2, 3) * idet,
			self.det3(1, 2, 3, 0, 1, 3) * idet,
			T::ZERO - self.det3(1, 2, 3, 0, 1, 2) * idet,

			T::ZERO - self.det3(0, 2, 3, 1, 2, 3) * idet,
			self.det3(0, 2, 3, 0, 2, 3) * idet,
			T::ZERO - self.det3(0, 2, 3, 0, 1, 3) * idet,
			self.det3(0, 2, 3, 0, 1, 2) * idet,

			self.det3(0, 1, 3, 1, 2, 3) * idet,
			T::ZERO - self.det3(0, 1, 3, 0, 2, 3) * idet,
			self.det3(0, 1, 3, 0, 1, 3) * idet,
			T::ZERO - self.det3(0, 1, 3, 0, 1, 2) * idet,

			T::ZERO - self.det3(0, 1, 2, 1, 2, 3) * idet,
			self.det3(0, 1, 2, 0, 2, 3) * idet,
			T::ZERO - self.det3(0, 1, 2, 0, 1, 3) * idet,
			self.det3(0, 1, 2, 0, 1, 2) * idet
		]))
	}

	/// Invert the matrix if it is inversible.
	///
	/// Return `true` if the matrix is invertible (in which case it has been inverted),
	/// or `false` if it is not, in which case it is left unchanged.
	pub fn invert(&mut self) -> bool where T: Copy + MonoidAdd + MonoidMul + Sub<Output=T> + Div<Output=T> + PartialEq {
		match self.inverted() {
			Some(inv) => {
				*self = inv;
				true
			},
			None => false
		}
	}
}

impl<T> Index<(u8, u8)> for Matrix4x4<T> {
	type Output = T;

	#[inline]
	fn index(&self, (i, j): (u8, u8)) -> &T {
		&self.0[(i * 4 + j) as usize]
	}
}

impl<T> Index<(i32, i32)> for Matrix4x4<T> {
	type Output = T;

	#[inline]
	fn index(&self, (i, j): (i32, i32)) -> &T {
		&self.0[(i * 4 + j) as usize]
	}
}

impl<T> Index<(usize, usize)> for Matrix4x4<T> {
	type Output = T;

	#[inline]
	fn index(&self, (i, j): (usize, usize)) -> &T {
		&self.0[(i * 4 + j)]
	}
}

impl<T> Index<u8> for Matrix4x4<T> {
	type Output = T;

	#[inline]
	fn index(&self, i: u8) -> &T {
		&self.0[i as usize]
	}
}

impl<T> Index<usize> for Matrix4x4<T> {
	type Output = T;

	#[inline]
	fn index(&self, i: usize) -> &T {
		&self.0[i]
	}
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Mul for Matrix4x4<T> {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: Self) -> Self {
		Matrix4x4(transposed![
			self[(0, 0)] * rhs[(0, 0)] + self[(1, 0)] * rhs[(0, 1)] + self[(2, 0)] * rhs[(0, 2)] + self[(3, 0)] * rhs[(0, 3)],
			self[(0, 0)] * rhs[(1, 0)] + self[(1, 0)] * rhs[(1, 1)] + self[(2, 0)] * rhs[(1, 2)] + self[(3, 0)] * rhs[(1, 3)],
			self[(0, 0)] * rhs[(2, 0)] + self[(1, 0)] * rhs[(2, 1)] + self[(2, 0)] * rhs[(2, 2)] + self[(3, 0)] * rhs[(2, 3)],
			self[(0, 0)] * rhs[(3, 0)] + self[(1, 0)] * rhs[(3, 1)] + self[(2, 0)] * rhs[(3, 2)] + self[(3, 0)] * rhs[(3, 3)],

			self[(0, 1)] * rhs[(0, 0)] + self[(1, 1)] * rhs[(0, 1)] + self[(2, 1)] * rhs[(0, 2)] + self[(3, 1)] * rhs[(0, 3)],
			self[(0, 1)] * rhs[(1, 0)] + self[(1, 1)] * rhs[(1, 1)] + self[(2, 1)] * rhs[(1, 2)] + self[(3, 1)] * rhs[(1, 3)],
			self[(0, 1)] * rhs[(2, 0)] + self[(1, 1)] * rhs[(2, 1)] + self[(2, 1)] * rhs[(2, 2)] + self[(3, 1)] * rhs[(2, 3)],
			self[(0, 1)] * rhs[(3, 0)] + self[(1, 1)] * rhs[(3, 1)] + self[(2, 1)] * rhs[(3, 2)] + self[(3, 1)] * rhs[(3, 3)],

			self[(0, 2)] * rhs[(0, 0)] + self[(1, 2)] * rhs[(0, 1)] + self[(2, 2)] * rhs[(0, 2)] + self[(3, 2)] * rhs[(0, 3)],
			self[(0, 2)] * rhs[(1, 0)] + self[(1, 2)] * rhs[(1, 1)] + self[(2, 2)] * rhs[(1, 2)] + self[(3, 2)] * rhs[(1, 3)],
			self[(0, 2)] * rhs[(2, 0)] + self[(1, 2)] * rhs[(2, 1)] + self[(2, 2)] * rhs[(2, 2)] + self[(3, 2)] * rhs[(2, 3)],
			self[(0, 2)] * rhs[(3, 0)] + self[(1, 2)] * rhs[(3, 1)] + self[(2, 2)] * rhs[(3, 2)] + self[(3, 2)] * rhs[(3, 3)],

			self[(0, 3)] * rhs[(0, 0)] + self[(1, 3)] * rhs[(0, 1)] + self[(2, 3)] * rhs[(0, 2)] + self[(3, 3)] * rhs[(0, 3)],
			self[(0, 3)] * rhs[(1, 0)] + self[(1, 3)] * rhs[(1, 1)] + self[(2, 3)] * rhs[(1, 2)] + self[(3, 3)] * rhs[(1, 3)],
			self[(0, 3)] * rhs[(2, 0)] + self[(1, 3)] * rhs[(2, 1)] + self[(2, 3)] * rhs[(2, 2)] + self[(3, 3)] * rhs[(2, 3)],
			self[(0, 3)] * rhs[(3, 0)] + self[(1, 3)] * rhs[(3, 1)] + self[(2, 3)] * rhs[(3, 2)] + self[(3, 3)] * rhs[(3, 3)]
		])
	}
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> MulAssign for Matrix4x4<T> {
	fn mul_assign(&mut self, rhs: Self) {
		let m = *self * rhs;
		*self = m
	}
}

impl<'a, T: Copy + Add<Output = T> + Mul<Output = T>> Mul<&'a Self> for Matrix4x4<T> {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: &'a Self) -> Self {
		self * *rhs
	}
}

impl<'a, T: Copy + Add<Output = T> + Mul<Output = T>> Mul<Matrix4x4<T>> for &'a Matrix4x4<T> {
	type Output = Matrix4x4<T>;

	#[inline]
	fn mul(self, rhs: Matrix4x4<T>) -> Matrix4x4<T> {
		*self * rhs
	}
}

impl<'a, 'b, T: Copy + Add<Output = T> + Mul<Output = T>> Mul<&'b Matrix4x4<T>> for &'a Matrix4x4<T> {
	type Output = Matrix4x4<T>;

	#[inline]
	fn mul(self, rhs: &'b Matrix4x4<T>) -> Matrix4x4<T> {
		*self * *rhs
	}
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Mul<Vector4D<T>> for Matrix4x4<T> {
	type Output = Vector4D<T>;

	#[inline]
	fn mul(self, rhs: Vector4D<T>) -> Vector4D<T> {
		Vector4D::new(
			self[(0, 0)] * rhs.x + self[(1, 0)] * rhs.y + self[(2, 0)] * rhs.z + self[(3, 0)] * rhs.w,
			self[(0, 1)] * rhs.x + self[(1, 1)] * rhs.y + self[(2, 1)] * rhs.z + self[(3, 1)] * rhs.w,
			self[(0, 2)] * rhs.x + self[(1, 2)] * rhs.y + self[(2, 2)] * rhs.z + self[(3, 2)] * rhs.w,
			self[(0, 3)] * rhs.x + self[(1, 3)] * rhs.y + self[(2, 3)] * rhs.z + self[(3, 3)] * rhs.w
		)
	}
}

impl<T: Copy + Add<Output = T> + Mul<Output = T> + Div<Output=T>> Mul<Vector3D<T>> for Matrix4x4<T> {
	type Output = Vector3D<T>;

	#[inline]
	fn mul(self, rhs: Vector3D<T>) -> Vector3D<T> {
		let w = self[(0, 3)] * rhs.x + self[(1, 3)] * rhs.y + self[(2, 3)] * rhs.z + self[(3, 3)];

		Vector3D::new(
			(self[(0, 0)] * rhs.x + self[(1, 0)] * rhs.y + self[(2, 0)] * rhs.z + self[(3, 0)]) / w,
			(self[(0, 1)] * rhs.x + self[(1, 1)] * rhs.y + self[(2, 1)] * rhs.z + self[(3, 1)]) / w,
			(self[(0, 2)] * rhs.x + self[(1, 2)] * rhs.y + self[(2, 2)] * rhs.z + self[(3, 2)]) / w
		)
	}
}

impl<T: fmt::Display> fmt::Display for Matrix4x4<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f,
			"[{}, {}, {}, {}]\n[{}, {}, {}, {}]\n[{}, {}, {}, {}]\n[{}, {}, {}, {}]",
			self.0[0], self.0[4], self.0[8], self.0[12],
			self.0[1], self.0[5], self.0[9], self.0[13],
			self.0[2], self.0[6], self.0[10], self.0[14],
			self.0[3], self.0[7], self.0[11], self.0[15]
		)
	}
}

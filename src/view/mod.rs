pub mod geometry;
pub mod object;

pub use geometry::Geometry;
pub use object::Object;

/// Object graphical representation.
pub enum View {
	Object(Object)
}
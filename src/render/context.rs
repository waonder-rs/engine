use magma::device;
use crate::sync::Loader;
use super::Target;

/// Render context.
pub trait Context {
	/// Render target type.
	type Target: Target;

	/// Render target.
	fn target(&self) -> &Self::Target;

	/// Queue used to execute graphics commands.
	fn graphics_queue(&self) -> &device::Queue;

	/// Resources loader.
	fn loader(&self) -> &Loader;
}
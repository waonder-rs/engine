mod thread;
mod conductor;

pub use thread::Thread;
pub use conductor::Conductor;

pub trait Worker<T> {
	fn cycle(&mut self, state: &T);

	fn apply(&mut self, state: &mut T);
}

use std::sync::Arc;
use magma::sync::{
	fence,
	Fence
};
use crossbeam_queue::SegQueue;

type Fences<T> = SegQueue<(Box<dyn Fence>, Box<dyn Send + FnOnce(&mut T) -> ()>)>;

pub struct Waiter<T> {
	fences: Arc<Fences<T>>
}

impl<T> Waiter<T> {
	pub fn new() -> (Waiter<T>, Worker<T>) {
		let fences = Arc::new(Fences::new());

		let waiter = Waiter {
			fences: fences.clone()
		};

		let worker = Worker {
			pending_fences: fences,
			fences: Vec::new(),
			signaled: Vec::new()
		};

		(waiter, worker)
	}
}

/// Fence synchronization.
pub struct Worker<T> {
	pending_fences: Arc<Fences<T>>,
	fences: Vec<(Box<dyn Fence>, Box<dyn Send + FnOnce(&mut T) -> ()>)>,
	signaled: Vec<Box<dyn Send + FnOnce(&mut T) -> ()>>
}

impl<T> cycles::Worker<T> for Worker<T> {
	fn cycle(&mut self, _: &T) {
		self.signaled = self.fences.drain_filter(|(fence, _)| fence.is_signaled().expect("fence error")).map(|(_, f)| f).collect();
	}

	fn apply(&mut self, state: &mut T) {
		while let Some(new_fence) = self.pending_fences.pop() {
			self.fences.push(new_fence)
		}

		for f in self.signaled.drain(..) {
			f(state)
		}
	}
}
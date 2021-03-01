use std::sync::Arc;
use magma::{
	sync::{
		future::SignalFence
	}
};
use crossbeam_queue::SegQueue;

pub(crate) struct Future {
	inner: Box<dyn SignalFence>
}

impl Future {
	pub fn new<F: 'static + SignalFence>(future: F) -> Self {
		Self {
			inner: Box::new(future)
		}
	}

	pub fn is_signaled(&self) -> bool {
		self.inner.is_signaled().expect("fence error")
	}
}

pub(crate) type Futures = Arc<SegQueue<Future>>;

/// Fence synchronization.
pub struct Worker {
	pending_futures: Futures,
	futures: Vec<Future>,
	signaled: Vec<Future>
}

impl Worker {
	pub fn new() -> Self {
		Worker {
			pending_futures: Arc::new(SegQueue::new()),
			futures: Vec::new(),
			signaled: Vec::new()
		}
	}

	pub(crate) fn pending_futures(&self) -> &Futures {
		&self.pending_futures
	}
}

impl<T> cycles::Worker<T> for Worker {
	fn cycle(&mut self, _: &T) {
		self.signaled = self.futures.drain_filter(|f| f.is_signaled()).collect();
	}

	fn apply(&mut self, _: &mut T) {
		while let Some(new_future) = self.pending_futures.pop() {
			self.futures.push(new_future)
		}
		
		for future in self.signaled.drain(..) {
			std::mem::drop(future)
		}
	}
}
use std::sync::{
	Arc,
	atomic::{
		AtomicBool,
		Ordering
	}
};
use std::pin::Pin;
use std::task::{
	Poll,
	Context,
	Waker
};
use parking_lot::Mutex;
use bottle::{
	Remote,
	Handler,
	Receiver,
	Event,
	EventQueue,
	Output
};
use vulkano::{
	device::{
		Device,
		Queue
	},
	sync::GpuFuture
};

// pub enum Delayed<T> {
// 	NotReady(Future<T>, Option<T>),
// 	Ready(T)
// }

/// In charge of transfering data from/to the GPU.
pub struct Loader {
	// The transfert queue
	queue: Arc<Queue>,
	waiter: Remote<Waiter>,

	p: Mutex<Inner>
}

struct Inner {
	futures: Vec<Arc<State>>,
	gpu_future: Option<Box<dyn GpuFuture + Send + Sync>>,
}

impl Loader {
	pub fn new(transfert_queue: &Arc<Queue>) -> (Loader, EventQueue) {
		let waiter_queue = EventQueue::new();
		let waiter = Remote::new(waiter_queue.reference(), Waiter);

		let loader = Loader {
			queue: transfert_queue.clone(),
			waiter,

			p: Mutex::new(Inner {
				futures: Vec::new(),
				gpu_future: None
			})
		};

		(loader, waiter_queue)
	}

	/// Vulkan transfert queue.
	pub fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}

	/// Load a resource represented by the given future and then return the given value when finished.
	///
	/// The future must execute the loader transfert queue.
	pub fn load<F: 'static + GpuFuture + Sync + Send, T>(&self, f: F, value: T) -> Future<T> {
		assert!(f.queue().unwrap() == self.queue);
		let mut p = self.p.lock();

		let gpu_future = match p.gpu_future.take() {
			Some(gpu_future) => {
				Box::new(gpu_future.join(f)) as Box<dyn GpuFuture + Sync + Send>
			},
			None => {
				Box::new(f) as Box<dyn GpuFuture + Sync + Send>
			}
		};

		p.gpu_future.replace(gpu_future);

		let state = Arc::new(State {
			completed: AtomicBool::new(false),
			waker: Mutex::new(None)
		});

		p.futures.push(state.clone());
		Future {
			value: Some(value),
			state
		}
	}

	/// Put a fence after the currently pending futures.
	///
	/// This method must be called regularly so the CPU can be notified when futures are completed.
	pub fn flush(&self) {
		let mut p = self.p.lock();
		if let Some(future) = p.gpu_future.take() {
			let mut states = Vec::new();
			std::mem::swap(&mut states, &mut p.futures);
			self.waiter.send(Flush(future, states));
		}
	}
}

struct Waiter;

struct Flush(Box<dyn GpuFuture + Send + Sync>, Vec<Arc<State>>);

impl Event for Flush {
	type Response = ();
}

impl Handler<Flush> for Waiter {
	fn handle<'a>(self: Receiver<'a, Self>, Flush(future, states): Flush) -> Output<'a, ()> {
		future.then_signal_fence_and_flush().unwrap().wait(None);

		for state in states {
			state.completed.store(true, Ordering::Relaxed);
			let mut waker = state.waker.lock();
			if let Some(waker) = waker.take() {
				waker.wake();
			}
		}

		Output::Now(())
	}
}

pub struct State {
	completed: AtomicBool,
	waker: Mutex<Option<Waker>>
}

pub struct Future<T> {
	value: Option<T>,
	state: Arc<State>
}

impl<T: Unpin> std::future::Future for Future<T> {
	type Output = T;

	fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<T> {
		let this = self.get_mut();
		let mut waker = this.state.waker.lock();
		waker.replace(ctx.waker().clone());

		if this.state.completed.load(Ordering::Relaxed) {
			match this.value.take() {
				Some(value) => Poll::Ready(value),
				None => Poll::Pending
			}
		} else {
			Poll::Pending
		}
	}
}

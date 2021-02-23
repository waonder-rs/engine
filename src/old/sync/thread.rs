use std::sync::Arc;
use std::cell::UnsafeCell;
use crossbeam_channel::{
	Sender,
	Receiver
};
use super::Worker;

struct UnsafeWorker<S>(Arc<UnsafeCell<dyn Worker<S> + Send>>);

unsafe impl<S> Send for UnsafeWorker<S> {}

impl<S> Clone for UnsafeWorker<S> {
	fn clone(&self) -> UnsafeWorker<S> {
		UnsafeWorker(self.0.clone())
	}
}

impl<S> UnsafeWorker<S> {
	fn new<T: 'static + Worker<S> + Send>(worker: T) -> UnsafeWorker<S> {
		UnsafeWorker(Arc::new(UnsafeCell::new(worker)) as Arc<UnsafeCell<dyn Worker<S> + Send>>)
	}

	unsafe fn get(&self) -> &mut dyn Worker<S> {
		&mut *self.0.get()
	}
}

pub struct Thread<S> {
	workers: Vec<UnsafeWorker<S>>,
	processor: Option<Processor<S>>,
	wake_signal: Sender<UnsafeRef<S>>,
	end_signal: Receiver<()>
}

impl<S> Thread<S> {
	pub fn new() -> Thread<S> {
		let (ws, wr) = crossbeam_channel::bounded(1);
		let (es, er) = crossbeam_channel::bounded(1);
		Thread {
			workers: Vec::new(),
			processor: Some(Processor {
				workers: Vec::new(),
				wake_signal: wr,
				end_signal: es
			}),
			wake_signal: ws,
			end_signal: er
		}
	}

	pub fn add<T: 'static + Worker<S> + Send>(&mut self, worker: T) {
		if let Some(proc) = self.processor.as_mut() {
			let worker = UnsafeWorker::new(worker);
			proc.workers.push(worker.clone());
			self.workers.push(worker);
		} else {
			panic!("thread already started")
		}
	}

	/// Take the thread processor.
	pub fn process(&mut self) -> Option<Processor<S>> {
		self.processor.take()
	}

	/// Start the thread.
	pub fn start(&mut self) where S: 'static + Sync {
		if let Some(mut proc) = self.process() {
			std::thread::spawn(move || {
				proc.start()
			});
		} else {
			panic!("thread already started")
		}
	}

	/// Start a cycle.
	///
	/// ## Safety
	/// The state reference must live until the cycle is finished,
	/// when the next call to [wait] returns.
	pub(crate) unsafe fn cycle(&self, state: &S) {
		self.wake_signal.send(UnsafeRef(state)).unwrap()
	}

	pub(crate) fn wait(&self) {
		self.end_signal.recv().unwrap()
	}

	/// Apply changes to the state.
	///
	/// ## Safety
	/// The thread must by pause (between two cycles) while this method is called.
	pub(crate) unsafe fn apply(&self, state: &mut S) {
		for worker in &self.workers {
			// Nobody is using the worker right now, we can safely transmute it to a mutable ref.
			let worker_mut: &mut dyn Worker<S> = worker.get();

			worker_mut.apply(state);
		}
	}
}

struct UnsafeRef<S>(*const S);

impl<S> UnsafeRef<S> {
	unsafe fn safe(&self) -> &S {
		&*self.0
	}
}

unsafe impl<S: Sync> Send for UnsafeRef<S> {}

pub struct Processor<S> {
	workers: Vec<UnsafeWorker<S>>,
	wake_signal: Receiver<UnsafeRef<S>>,
	end_signal: Sender<()>
}

impl<S> Processor<S> {
	/// Start the thread.
	pub fn start(&mut self) {
		loop {
			unsafe { // safety is ensured by the [Thread::cycle] and [Thread::apply].
				let unsafe_state = self.wake_signal.recv().unwrap();
				let state = unsafe_state.safe();

				for worker in &self.workers {
					// Nobody is using the worker right now, we can safely transmute it to a mutable ref.
					let worker_mut: &mut dyn Worker<S> = worker.get();

					worker_mut.cycle(state);
				}

				self.end_signal.send(()).unwrap()
			}
		}
	}
}

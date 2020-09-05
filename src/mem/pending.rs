use std::sync::{
	Arc,
	atomic::{
		AtomicBool,
		Ordering
	}
};

pub struct PendingState(AtomicBool);

impl PendingState {
	pub fn new() -> PendingState {
		PendingState(AtomicBool::new(false))
	}

	pub fn set_ready(&self) {
		self.0.store(true, Ordering::Relaxed);
	}

	pub fn is_ready(&self) -> bool {
		self.0.load(Ordering::Relaxed)
	}
}

/// Values that may not be ready to use.
pub struct Pending<T> {
	value: T,
	state: Arc<PendingState>
}

impl<T> Pending<T> {
	/// Create a new pending value along with its pending state.
	///
	/// The given value will only be accessible when the pending state is set ready.
	pub fn new(value: T) -> (Pending<T>, Arc<PendingState>) {
		let state = Arc::new(PendingState::new());
		let pending = Pending {
			value,
			state: state.clone()
		};
		(pending, state)
	}
}

// pub struct Volatile<T> {
// 	current: Option<Pending<T>>,
// 	previous: Option<Pending<T>>
// }
//
// impl Volatile<T> {
// 	pub fn new() -> Volatile<T> {
// 		Volatile {
// 			current: None,
// 			previous: None
// 		}
// 	}
//
// 	pub fn set(&mut self, value: Pending<T>) {
// 		self.previous = self.current.take();
// 		self.current = Some(value);
// 	}
//
// 	pub fn get(&self) -> Option<&T> {
// 		match self.current.as_ref() {
// 			Some(pending_value) => {
// 				match pending_value.get() {
// 					Some(value) => value,
// 					None => {
// 						match self.previous.as_ref() {
// 							Some(pending_value) => {
// 								pending_value.get()
// 							},
// 							None => None
// 						}
// 					}
// 				}
// 			},
// 			None => None
// 		}
// 	}
// }

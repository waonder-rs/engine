use super::Thread;

pub struct Conductor<S> {
	state: S,
	threads: Vec<Thread<S>>
}

impl<S> Conductor<S> {
	pub fn new(value: S) -> Conductor<S> {
		Conductor {
			state: value,
			threads: Vec::new()
		}
	}

	pub fn add(&mut self, thread: Thread<S>) {
		self.threads.push(thread)
	}

	/// Cycle.
	///
	/// Wake every threads for a new cycle,
	/// then apply changes.
	pub fn cycle(&mut self) {
		unsafe {
			for thread in &self.threads {
				thread.cycle(&self.state);
			}

			for thread in &self.threads {
				thread.wait();
			}

			for thread in &self.threads {
				thread.apply(&mut self.state);
			}
		}
	}

	/// Inverse cycle.
	///
	/// Apply changes, then wake every threads for a new cycle.
	pub fn inverse_cycle(&mut self) {
		unsafe {
			for thread in &self.threads {
				thread.apply(&mut self.state);
			}

			for thread in &self.threads {
				thread.cycle(&self.state);
			}

			for thread in &self.threads {
				thread.wait();
			}
		}
	}

	pub fn start(&mut self) {
		loop {
			self.cycle()
		}
	}
}

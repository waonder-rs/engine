use crossbeam_channel::{
	Sender,
};
use magma::{
	device,
	mem::{
		Allocator,
		buffer
	},
	sync
};

mod query;
mod thread;
mod worker;
pub mod loading;

pub use query::Query;
pub use thread::Thread;
pub use worker::Worker;
pub use loading::Loading;

pub struct Loader {
	channel: Sender<Query>
}

impl Loader {
	pub fn new<A: Allocator>(allocator: A, transfert_queue: device::Queue) -> (Self, Thread<A>, Worker) {
		let (queries_sender, queries_receiver) = crossbeam_channel::unbounded();
		
		let worker = Worker::new();

		let thread = Thread::new(
			allocator,
			transfert_queue,
			queries_receiver,
			worker.pending_futures()
		);

		let loader = Self {
			channel: queries_sender
		};

		(loader, thread, worker)
	}

	pub fn load_untyped<B: 'static + AsRef<[u8]>, U: Into<buffer::Usages>, S: Into<sync::SharingQueues>>(
		&self,
		data: B,
		usage: U,
		sharing_queues: S
	) -> Loading<buffer::Bound> {
		let (loading, handle) = Loading::new();

		self.channel.send(Query::Load {
			data: Box::new(data),
			usage: usage.into(),
			sharing_queues: sharing_queues.into(),
			buffer: handle
		}).expect("unable to send loader query");

		loading
	}

	pub fn load<T: 'static + Copy, B: 'static + std::ops::Deref<Target=[T]>, U: Into<buffer::Usages>, S: Into<sync::SharingQueues>>(
		&self,
		data: B,
		usage: U,
		sharing_queues: S
	) -> Loading<buffer::Typed<T>> {
		let (loading, handle) = Loading::mapped(|bound: buffer::Bound| unsafe {
			// Safe because the original buffer is of type T.
			bound.into_typed::<T>()
		});

		self.channel.send(Query::Load {
			data: Box::new(DataSource::new(data)),
			usage: usage.into(),
			sharing_queues: sharing_queues.into(),
			buffer: handle
		}).expect("unable to send loader query");

		loading
	}
}

pub struct DataSource<S> {
	source: S
}

impl<S> DataSource<S> {
	pub fn new(source: S) -> Self {
		Self {
			source
		}
	}
}

impl<T: Copy, S: std::ops::Deref<Target=[T]>> AsRef<[u8]> for DataSource<S> {
	fn as_ref(&self) -> &[u8] {
		let src = self.source.deref().as_ref();
		let ptr = src.as_ptr();
		unsafe {
			std::slice::from_raw_parts(ptr as *const u8, src.len() * std::mem::size_of::<T>())
		}
	}
}
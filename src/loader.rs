use std::sync::{
	Arc,
	// atomic::{
	// 	AtomicBool,
	// 	Ordering
	// }
};
use once_cell::sync::OnceCell;
use magma::{
	Device,
	DeviceOwned,
	device,
	command,
	sync::{
		fence
	},
	buffer,
	alloc::{
		Allocator,
		Buffer
	},
	sync
};
// use std::pin::Pin;
// use std::task::{
// 	Poll,
// 	Context,
// 	Waker
// };
// use parking_lot::Mutex;
// use bottle::{
// 	Remote,
// 	Handler,
// 	Receiver,
// 	Event,
// 	EventQueue,
// 	Output
// };

pub struct Loading<T> {
	inner: Arc<OnceCell<T>>
}

pub enum Query<A: Allocator> {
	Load {
		data: Vec<u8>,
		usage: buffer::Usages,
		sharing_queues: sync::SharingQueues,
		buffer: Arc<OnceCell<Buffer<A>>>
	},
	CopyBuffer {
		src: Arc<dyn magma::Buffer>,
		dst: Arc<dyn magma::Buffer>
	}
}

pub struct Loader<A: Allocator> {
	// ...
}

impl<A: Allocator> Loader<A> {
	pub fn load<'a, U: Into<buffer::Usages>, S: Into<sync::SharingQueues>>(
		&self,
		data: Vec<u8>,
		usage: U,
		sharing_queues: S
	) -> Loading<Buffer<A>> {
		panic!("TODO")
	}

	pub fn copy_buffer<S: 'static + magma::Buffer, D: 'static + magma::Buffer>(&mut self, src: &Arc<S>, dst: &Arc<D>) -> Future {
		// self.queries.push(Query::CopyBuffer {
		// 	src: src.clone(),
		// 	dst: dst.clone()
		// });

		panic!("TODO")
	}
}

pub struct Worker<A> {
	allocator: A,
	transfert_queue: device::Queue,
	command_buffer: command::Buffer<'static>,
	queries: Vec<Query<A>>
}

impl<A: Allocator> Worker<A> {
	fn device(&self) -> &Arc<Device> {
		self.transfert_queue.device()
	}

	fn flush(&self) {
		self.transfert_queue.submit(self.command_buffer)
	}

	/// Process a single query.
	fn process_query(&self, commands: command::buffer::Recorder, query: Query<A>) {
		match query {
			Query::Load { data, usage, sharing_queues, buffer } => {
				let staging_buffer = buffer::Unbound::new(
					self.device(),
					data.len() as u64,
					buffer::Usage::TransferSource,
					Some(&self.transfert_queue)
				).expect("unable to create staging buffer");
		
				let mut sharing_queues = sharing_queues.into();
				sharing_queues.insert(&self.transfert_queue);
		
				let remote_buffer = buffer::Unbound::new(
					self.device(),
					data.len() as u64,
					usage.into() | buffer::Usage::TransferDestination,
					sharing_queues.into_iter().chain(Some(&self.transfert_queue).into_iter())
				).expect("unable to create buffer");
		
				let staging_slot = self.allocator.allocate(staging_buffer.memory_requirements());
				let remote_slot = self.allocator.allocate(staging_buffer.memory_requirements());
		
				let staging_buffer = match staging_buffer.bind(staging_slot) {
					Ok(bound) => Arc::new(bound),
					Err((_, e)) => panic!("unable to bind staging buffer memory: {:?}", e)
				};

				let remote_buffer = match remote_buffer.bind(remote_slot) {
					Ok(bound) => Arc::new(bound),
					Err((_, e)) => panic!("unable to bind remote buffer memory: {:?}", e)
				};

				commands.copy_buffer(&staging_buffer, &remote_buffer, &[])
			}
		}
	}
}

// pub struct Worker {
// 	transfert_queue: device::Queue,
// 	command_buffer: command::Buffer<'static>,
// 	fence: fence::Raw
// }

// impl Worker {
// 	pub fn flush(&mut self) {
// 		let ((), all_done) = self.transfert_queue.submit(self.command_buffer).then_signal_fence(&self.fence).expect("unable to submit");
// 		all_done.wait(None)
// 	}
// }

// struct Flush;

// impl Event for Flush {
// 	type Response = ();
// }

// impl Handler<Flush> for Worker {
// 	fn handle<'a>(self: Receiver<'a, Self>, _: Flush) -> Output<'a, ()> {
// 		self.flush();
// 		Output::Now(())
// 	}
// }
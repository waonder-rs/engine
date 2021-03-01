use std::sync::Arc;
use crossbeam_channel::{
	Receiver
};
use magma::{
	device,
	Device,
	DeviceOwned,
	mem::Allocator,
	command::Buffer,
	sync::Task
};
use crate::sync::{
	FencePool,
	CommandBufferPool
};
use super::{
	Query,
	worker
};

/// Loader thread.
/// 
/// The loader thread is in charge of executing queries, recording and sending command buffers.
pub struct Thread<A: Allocator> {
	allocator: A,
	transfert_queue: device::Queue,
	queries: Receiver<Query>,
	worker_futures: worker::Futures,
	fence_pool: FencePool,
	command_buffer_pool: CommandBufferPool,
	prepared_queries: Vec<Query>
}

impl<A: Allocator> Thread<A> {
	pub(crate) fn new(
		allocator: A,
		transfert_queue: device::Queue,
		queries: Receiver<Query>,
		worker_futures: &worker::Futures
	) -> Self {
		let fence_pool = FencePool::new(transfert_queue.device());
		let command_buffer_pool = CommandBufferPool::new(&transfert_queue).expect("unable to create command buffer pool");

		Self {
			allocator,
			transfert_queue,
			queries,
			worker_futures: worker_futures.clone(),
			fence_pool,
			command_buffer_pool,
			prepared_queries: Vec::new()
		}
	}

	fn device(&self) -> &Arc<Device> {
		self.transfert_queue.device()
	}

	fn flush(&mut self) {
		let device = self.device().clone();
		let command_buffer = self.command_buffer_pool.get().expect("unable to allocate command buffer");
		let recorded_command_buffer = command_buffer.record(|commands| {
			for query in self.prepared_queries.drain(..) {
				query.process(
					&device,
					&self.transfert_queue,
					&mut self.allocator,
					commands
				)
			}
		}).expect("unable to record command buffer");

		let fence = self.fence_pool.get().expect("unable to create fence");
		let (_, future) = self.transfert_queue.submit(recorded_command_buffer).then_signal_fence(fence).expect("unable to submit command buffer");
		self.worker_futures.push(worker::Future::new(future))
	}

	fn prepare_query(&mut self, query: &Query) {
		// ...
	}

	fn run(&mut self) {
		loop {
			match self.queries.recv() {
				Ok(Query::Flush) => {
					self.flush()
				},
				Ok(query) => {
					self.prepare_query(&query);
					self.prepared_queries.push(query);
				},
				Err(e) => {
					log::error!("loader Thread error: {}", e);
					break
				}
			}
		}
	}
}
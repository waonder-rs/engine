use std::{
	sync::{
		Arc,
	},
	convert::TryInto
};
use magma::{
	Device,
	device,
	command,
	mem::{
		Allocator,
		HostVisibleSlot,
		buffer,
	},
	sync
};
use super::loading;

pub enum Query {
	Flush,
	Load {
		data: Box<dyn AsRef<[u8]>>,
		usage: buffer::Usages,
		sharing_queues: sync::SharingQueues,
		buffer: loading::Handle<buffer::Bound>
	},
	// CopyBuffer {
	// 	src: Arc<dyn magma::Buffer>,
	// 	dst: Arc<dyn magma::Buffer>
	// }
}

impl Query {
	/// Process a single query.
	pub fn process<A: Allocator, B: command::Buffer>(
		self,
		device: &Arc<Device>,
		transfert_queue: &device::Queue,
		allocator: &mut A,
		commands: &mut command::buffer::Recorder<B>
	) {
		match self {
			Query::Load { data, usage, mut sharing_queues, buffer } => {
				let src = (*data).as_ref();

				let staging_buffer = buffer::Unbound::new(
					device,
					src.len() as u64,
					buffer::Usage::TransferSource,
					transfert_queue
				).expect("unable to create staging buffer");
		
				sharing_queues.insert(transfert_queue);
		
				let remote_buffer = buffer::Unbound::new(
					device,
					src.len() as u64,
					usage | buffer::Usage::TransferDestination,
					sharing_queues
				).expect("unable to create buffer");
		
				let staging_slot: A::HostVisibleSlot = allocator.allocate(staging_buffer.memory_requirements()).try_into().ok().unwrap();
				let dst = staging_slot.ptr().expect("unable to map staging buffer memory") as *mut u8;
				
				let remote_slot = allocator.allocate(remote_buffer.memory_requirements());
		
				let staging_buffer: buffer::Bound = match unsafe { staging_buffer.bind(staging_slot) } {
					Ok(bound) => bound,
					Err((_, e)) => panic!("unable to bind staging buffer memory: {:?}", e)
				};

				let remote_buffer: loading::Prepared<buffer::Bound> = match unsafe { remote_buffer.bind(remote_slot) } {
					Ok(bound) => buffer.prepare(bound),
					Err((_, e)) => panic!("unable to bind remote buffer memory: {:?}", e)
				};

				unsafe {
					std::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len())
				}

				commands.copy_buffer(staging_buffer, remote_buffer, &[command::buffer::BufferCopy {
					src_offset: 0,
					dst_offset: 0,
					size: src.len() as u64
				}])
			},
			Query::Flush => ()
		}
	}
}
pub struct Vertices<T> {
	inner: UnsafeBuffer,
	_t: PhantomData<T>
}

impl Vertices<T> {
	pub fn new(data: &[T], loader: &Loader) -> Future<Vertices> {
		let buffer = loader.alloc(data.len() * size_of::<T>())?;

		// transfert queue.
		let (vertex_buffer, vertex_future) = ImmutableBuffer::from_data(vertices, BufferUsage::vertex_buffer(), loader.queue().clone()).unwrap();
	}
}

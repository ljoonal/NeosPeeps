use rayon::ThreadPool;

#[derive(Debug)]
pub struct Pools {
	data: ThreadPool,
	// Also logout operations
	login: ThreadPool,
}

impl Default for Pools {
	fn default() -> Self {
		Self {
			data: rayon::ThreadPoolBuilder::new()
				.panic_handler(move |m| {
					println!("WARNING: Data thread panicked! {:?}", m);
				})
				.build()
				.unwrap(),
			login: rayon::ThreadPoolBuilder::new()
				.num_threads(1)
				.panic_handler(move |m| {
					println!("WARNING: Login thread panicked! {:?}", m);
				})
				.build()
				.unwrap(),
		}
	}
}

use rayon::ThreadPool;

mod channels;
mod pools;

use channels::Channels;

#[derive(Debug)]
pub struct Manager {
	pub channels: Channels,
	data: ThreadPool,
	// Also logout operations
	login: ThreadPool,
}

impl Default for Manager {
	fn default() -> Self {
		Self {
			channels: Channels::default(),
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

impl Manager {
	/// Also for logout operations
	pub fn spawn_login_op<OP>(&self, op: OP)
	where
		OP: FnOnce() + Send + 'static,
	{
		self.login.spawn(op);
	}

	/// Spawns a thread for fetching data from the API & so on.
	pub fn spawn_data_op<OP>(&self, op: OP)
	where
		OP: FnOnce() + Send + 'static,
	{
		self.data.spawn_fifo(op);
	}
}

use std::sync::{Arc, Mutex};
use steel::{
	declare_module,
	rvals::Custom,
	steel_vm::ffi::{FFIModule, RegisterFFIFn},
};

declare_module!(module);

struct GitRepo(Arc<Mutex<gix::Repository>>);

impl Custom for GitRepo {}

impl GitRepo {
	fn discover() -> Self {
		// TODO: pass path
		let repo = gix::discover(".").unwrap();

		GitRepo(Arc::new(Mutex::new(repo)))
	}

	fn blame(&self, file: &str, line: isize) -> Option<String> {
		let repo = self.0.lock().ok()?;
		let head = repo.head().ok()?.peel_to_object().ok()?.id;

		let blame = repo
			.blame_file(file.into(), head, gix::repository::blame_file::Options::default())
			.ok()?;

		let thing = blame.entries.into_iter().find(|blame| {
			(blame.start_in_source_file..blame.start_in_source_file + blame.len.get())
				.contains(&(line as u32))
		})?;

		let thing = repo.find_commit(thing.commit_id).ok()?;

		let hash = thing.short_id().unwrap();

		let author = thing.author().unwrap();
		let author = author.name;

		let message = thing.message().unwrap();
		let message = message.title.to_string();
		let message = message.trim();

		let time = thing.author().unwrap().time().unwrap();
		let time = time.format(gix::date::time::format::ISO8601).unwrap();

		Some(format!("{author}, {time} • {message} • {hash}"))
	}
}

fn module() -> FFIModule {
	let mut module = FFIModule::new("may/git-diff");

	module
		.register_fn("gix::discover", GitRepo::discover)
		.register_fn("gix/blame", GitRepo::blame);

	module
}

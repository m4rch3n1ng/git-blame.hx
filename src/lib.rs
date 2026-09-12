use self::fmt::{Format, Fragment, Variable};
use std::{
	str::FromStr,
	sync::{Arc, Mutex},
};
use steel::{
	declare_module,
	rvals::Custom,
	steel_vm::ffi::{FFIModule, RegisterFFIFn},
};

mod fmt;

declare_module!(module);

#[derive(Clone)]
struct GitRepo(Arc<Mutex<gix::Repository>>);

impl Custom for GitRepo {}

impl GitRepo {
	fn discover() -> Self {
		// TODO: pass path
		let repo = gix::discover(".").unwrap();

		GitRepo(Arc::new(Mutex::new(repo)))
	}

	fn blame(self, format: Format, file: &str, line: isize) -> Option<String> {
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
		let title = message.title.to_string();
		let title = title.trim();

		let date = thing.author().unwrap().time().unwrap();
		let date = date.format(gix::date::time::format::SHORT).unwrap();

		let info = Info {
			hash: hash.to_string(),
			author: Some(author.to_string()),
			title: Some(title.to_owned()),
			date: Some(date),
		};

		Some(info.format(format))
	}
}

struct Info {
	hash: String,
	author: Option<String>,
	title: Option<String>,
	date: Option<String>,
}

impl Info {
	fn get(&self, var: Variable) -> Option<&str> {
		match var {
			Variable::Hash => Some(&self.hash),
			Variable::Author => self.author.as_deref(),
			Variable::Title => self.title.as_deref(),
			Variable::Date => self.date.as_deref(),
		}
	}

	fn format(self, format: Format) -> String {
		let mut string = String::new();

		for fragment in &*format.0 {
			match fragment {
				Fragment::Verbatim(v) => string.push_str(v),
				Fragment::Variable(v) if let Some(t) = self.get(*v) => string.push_str(t),
				Fragment::Variable(_) => todo!(),
			}
		}

		string
	}
}

fn module() -> FFIModule {
	let mut module = FFIModule::new("may/git-diff");

	module
		.register_fn("gix::discover", GitRepo::discover)
		.register_fn("blame/default-format", Format::default)
		.register_fn("blame/format", Format::from_str)
		.register_fn("gix/blame", GitRepo::blame);

	module
}

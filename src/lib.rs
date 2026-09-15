use self::fmt::{Format, Fragment, Variable};
use gix::ThreadSafeRepository;
use std::{borrow::Cow, str::FromStr};
use steel::{
	declare_module,
	rvals::Custom,
	steel_vm::ffi::{FFIModule, RegisterFFIFn},
};

mod fmt;

declare_module!(module);

#[derive(Clone)]
struct GitRepo(ThreadSafeRepository);

impl Custom for GitRepo {}

impl GitRepo {
	fn discover(path: &str) -> Option<Self> {
		let repo = ThreadSafeRepository::discover(path).ok()?;
		Some(GitRepo(repo))
	}

	fn blame(self, format: Format, file_path: &str, line: isize) -> Option<String> {
		let repo = self.0.to_thread_local();
		let head = repo.head().ok()?.peel_to_object().ok()?.id;

		let Ok(file) = repo.normalize_path(file_path) else {
			return Some("File not in current git repo".to_owned());
		};

		let start = if let Ok(content) = std::fs::read(file_path) {
			gix::blame::Start::Contents {
				first_suspect: head,
				contents: Cow::Owned(content),
			}
		} else {
			gix::blame::Start::Commit(head)
		};

		let cache = repo.commit_graph_if_enabled().ok()?;
		let mut resource_cache = repo.diff_resource_cache_for_tree_diff().ok()?;

		let opts = gix::blame::Options::default();
		let blame = gix::blame::file(&repo.objects, start, cache, &mut resource_cache, &file, opts).ok()?;

		let entry = blame.entries.into_iter().find(|blame| {
			(blame.start_in_blamed_file..blame.start_in_blamed_file + blame.len.get())
				.contains(&(line as u32))
		})?;

		// gix blame returns a null commit id for uncommited changes
		if entry.commit_id.is_null() {
			return Some("Not commited yet".to_owned());
		}

		let commit = repo.find_commit(entry.commit_id).ok()?;

		let hash = commit.short_id().map(|short| short.to_string());
		let hash = hash.unwrap_or_else(|_| commit.id.to_string());

		let author = commit.author().map(|author| author.name.to_string()).ok();
		let title = commit.message().map(|message| message.title.to_string()).ok();

		let date = commit.author().ok().and_then(|author| author.time().ok());
		let date = date.and_then(|date| date.format(gix::date::time::format::SHORT).ok());

		let info = Info {
			commit: hash,
			author,
			title,
			date,
		};

		Some(info.format(format))
	}
}

struct Info {
	commit: String,
	author: Option<String>,
	title: Option<String>,
	date: Option<String>,
}

impl Info {
	fn get(&self, var: Variable) -> Option<&str> {
		match var {
			Variable::Commit => Some(&self.commit),
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
				Fragment::Variable(_) => (),
			}
		}

		string
	}
}

fn module() -> FFIModule {
	let mut module = FFIModule::new("may/git-diff");

	module
		.register_fn("git-blame::discover", GitRepo::discover)
		.register_fn("git-blame::blame", GitRepo::blame)
		.register_fn("git-blame::default-format", Format::default)
		.register_fn("git-blame::format", Format::from_str)
		.register_fn("git-blame::format?", Format::is_format);

	module
}

use self::fmt::{Format, Fragment, Variable};
use gix::ThreadSafeRepository;
use std::{path::Path, str::FromStr};
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
	fn discover() -> Option<Self> {
		// TODO: pass path
		let path = Path::new(".").canonicalize().ok()?;
		let repo = ThreadSafeRepository::discover(path).ok()?;
		Some(GitRepo(repo))
	}

	fn blame(self, format: Format, file: &str, line: isize) -> Option<String> {
		let repo = self.0.to_thread_local();
		let head = repo.head().ok()?.peel_to_object().ok()?.id;

		let file = pathdiff::diff_utf8_paths(file, repo.workdir()?.to_str()?)?;
		let blame = repo
			.blame_file(
				file.as_str().into(),
				head,
				gix::repository::blame_file::Options::default(),
			)
			.ok()?;

		let entry = blame.entries.into_iter().find(|blame| {
			(blame.start_in_blamed_file..blame.start_in_blamed_file + blame.len.get())
				.contains(&(line as u32))
		})?;
		let commit = repo.find_commit(entry.commit_id).ok()?;

		let hash = commit.short_id().map(|short| short.to_string());
		let hash = hash.unwrap_or_else(|_| commit.id.to_string());

		let author = commit.author().map(|author| author.name.to_string()).ok();

		let title = commit.message().map(|message| message.title.to_string()).ok();
		// see <https://github.com/GitoxideLabs/gitoxide/issues/2991>
		let title = title.map(|title| title.trim().to_owned());

		let date = commit.author().ok().and_then(|author| author.time().ok());
		let date = date.and_then(|date| date.format(gix::date::time::format::SHORT).ok());

		let info = Info {
			hash: hash.to_string(),
			author,
			title,
			date,
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
				Fragment::Variable(_) => (),
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

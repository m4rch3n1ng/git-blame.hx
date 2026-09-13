use std::{
	fmt::{Debug, Display},
	str::FromStr,
	sync::Arc,
};
use steel::rvals::Custom;

#[derive(Clone)]
pub struct Format(pub Arc<[Fragment]>);

#[derive(Clone)]
pub enum Fragment {
	Verbatim(String),
	Variable(Variable),
}

#[derive(Debug, Clone, Copy)]
pub enum Variable {
	Hash,
	Author,
	Title,
	Date,
}

#[derive(thiserror::Error)]
pub enum Error {
	#[error("unknown variable {0:?}")]
	UnknownVariable(String),
	#[error("missing closing '}}'")]
	UnclosedVariable,
}

impl Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self, f)
	}
}

impl Custom for Error {}

impl FromStr for Variable {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"hash" => Ok(Variable::Hash),
			"author" => Ok(Variable::Author),
			"title" => Ok(Variable::Title),
			"date" => Ok(Variable::Date),
			_ => Err(Error::UnknownVariable(s.to_owned())),
		}
	}
}

impl Custom for Fragment {}
impl Custom for Format {}

impl Default for Format {
	fn default() -> Self {
		Format::from_str("{author}, {date} • {title} • {hash}").expect("should always be valid")
	}
}

impl FromStr for Format {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut format = Vec::new();

		let mut verbatim = String::new();
		let mut chars = s.chars().peekable();
		while let Some(char) = chars.next() {
			if char == '{' {
				let verbatim = std::mem::take(&mut verbatim);
				if !verbatim.is_empty() {
					format.push(Fragment::Verbatim(verbatim));
				}

				let mut variable = String::new();
				while let Some(ch) = chars.next_if(|ch| *ch != '}') {
					variable.push(ch);
				}

				if chars.next().is_none() {
					return Err(Error::UnclosedVariable);
				}

				let variable = Variable::from_str(&variable)?;
				format.push(Fragment::Variable(variable));
			} else {
				verbatim.push(char);
			}
		}

		if !verbatim.is_empty() {
			format.push(Fragment::Verbatim(verbatim));
		}

		Ok(Format(Arc::from(format)))
	}
}

#[cfg(test)]
mod test {
	use super::Format;

	#[test]
	fn default_format() {
		let _ = Format::default();
	}
}

use std::{str::FromStr, sync::Arc};
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

impl FromStr for Variable {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"hash" => Ok(Variable::Hash),
			"author" => Ok(Variable::Author),
			"title" => Ok(Variable::Title),
			"date" => Ok(Variable::Date),
			_ => Err(()),
		}
	}
}

impl Custom for Fragment {}
impl Custom for Format {}

impl Default for Format {
	fn default() -> Self {
		Format::from_str("{author}, {date} • {title} • {hash}").unwrap()
	}
}

impl FromStr for Format {
	type Err = ();
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
					todo!();
				}

				let variable = Variable::from_str(&variable).unwrap();
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

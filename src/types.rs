use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

use dbus::arg::{Append, Arg, ArgType, Dict, IterAppend, PropMap, RefArg, Variant};
use dbus::Signature;

pub type AnyVariant = Variant<Box<dyn RefArg + 'static>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config<A> {
	pub match_regex: String,
	pub min_letter_count: i32,
	pub trigger_words: Vec<String>,
	pub actions: Vec<A>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match<A> {
	pub id: String,
	pub text: String,
	pub icon: String,
	pub ty: MatchType,
	pub relevance: f64,
	pub urls: Vec<String>,
	pub category: Option<String>,
	pub subtext: Option<String>,
	pub multiline: bool,
	pub actions: Vec<A>,
	pub icon_data: Option<RemoteImage>,
}

impl<A: FromStr + ToString> Match<A> {
	pub fn new(id: String) -> Self {
		Self {
			id,
			text: String::new(),
			icon: String::new(),
			ty: MatchType::NoMatch,
			relevance: 0.0,
			urls: vec![],
			category: None,
			subtext: None,
			multiline: false,
			actions: vec![],
			icon_data: None,
		}
	}

	pub fn text(self, text: String) -> Self {
		Self { text, ..self }
	}

	pub fn icon(self, icon: String) -> Self {
		Self { icon, ..self }
	}

	pub fn ty(self, ty: MatchType) -> Self {
		Self { ty, ..self }
	}

	pub fn relevance(self, relevance: f64) -> Self {
		Self { relevance, ..self }
	}

	pub fn url(mut self, url: String) -> Self {
		self.urls.push(url);
		self
	}

	pub fn category(self, category: String) -> Self {
		Self {
			category: Some(category),
			..self
		}
	}

	pub fn subtext(self, subtext: String) -> Self {
		Self {
			subtext: Some(subtext),
			..self
		}
	}

	pub fn multiline(self) -> Self {
		Self {
			multiline: true,
			..self
		}
	}

	pub fn action(mut self, action: A) -> Self {
		self.actions.push(action);
		self
	}

	pub fn icon_data(self, icon_data: RemoteImage) -> Self {
		Self {
			icon_data: Some(icon_data),
			..self
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionInfo {
	pub text: String,
	pub icon_source: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RemoteImage {
	pub width: i32,
	pub height: i32,
	pub row_stride: i32,
	pub has_alpha: bool,
	pub bits_per_sample: i32,
	pub channels: i32,
	pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum MatchType {
	/// Null match.
	NoMatch = 0,
	/// Possible completion for the data of the query.
	CompletionMatch = 10,
	/// Something that may match the query.
	PossibleMatch = 30,

	/// A purely informational, non-runnable match, such as the answer
	/// to a question or calculation.
	///
	/// The data of the match will be converted to a string and set in
	/// the search field.
	///
	/// **Deprecated** since KDE Frameworks version 5.99.
	#[deprecated]
	InformationalMatch = 50,

	/// A match that represents an action not directly related to activating
	/// the given search term, such as a search in an external tool or command
	/// learning trigger.
	///
	/// Helper matches tend to be generic to the query and should not be
	/// autoactivated just because the user hits "Enter" while typing.
	/// They must be explicitly selected to be activated, but unlike
	/// [`InformationalMatch`] cause an action to be triggered.
	HelperMatch = 70,

	/// An exact match to the query.
	ExactMatch = 100,
}

//= IMPL =//

fn assert_sig<T: Arg>(expected: &'static str) -> Signature<'static> {
	let sig = <T as Arg>::signature();
	debug_assert_eq!(&*sig, expected);
	sig
}

impl<A: FromStr + ToString> Arg for Config<A> {
	const ARG_TYPE: ArgType = ArgType::Array;

	fn signature() -> Signature<'static> {
		assert_sig::<PropMap>("a{sv}")
	}
}
impl<A: FromStr + ToString> Append for Config<A> {
	fn append_by_ref(&self, i: &mut IterAppend) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();
		fields.insert("MatchRegex", Variant(self.match_regex.box_clone()));
		fields.insert("MinLetterCount", Variant(self.min_letter_count.box_clone()));
		fields.insert("TriggerWords", Variant(self.trigger_words.box_clone()));

		let actions: Vec<_> = self.actions.iter().map(A::to_string).collect();
		fields.insert("Actions", Variant(actions.box_clone()));

		Dict::new(fields.iter()).append_by_ref(i)
	}
}

impl<A: FromStr + ToString> Arg for Match<A> {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(String, String, String, MatchType, f64, PropMap)>("(sssida{sv})")
	}
}
impl<A: FromStr + ToString> Append for Match<A> {
	fn append_by_ref(&self, i: &mut IterAppend) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();

		if !self.urls.is_empty() {
			fields.insert("urls", Variant(self.urls.box_clone()));
		}
		if let Some(category) = &self.category {
			fields.insert("category", Variant(category.box_clone()));
		}
		if let Some(subtext) = &self.subtext {
			fields.insert("subtext", Variant(subtext.box_clone()));
		}
		if self.multiline {
			fields.insert("multiline", Variant(self.multiline.box_clone()));
		}
		if !self.actions.is_empty() {
			let actions: Vec<_> = self.actions.iter().map(A::to_string).collect();
			fields.insert("actions", Variant(actions.box_clone()));
		}
		if let Some(icon_data) = &self.icon_data {
			fields.insert("icon-data", Variant(icon_data.box_clone()));
		}

		let fields = Dict::new(fields.iter());

		i.append((
			&self.id,
			&self.text,
			&self.icon,
			&self.ty,
			&self.relevance,
			&fields,
		));
	}
}

impl Arg for MatchType {
	const ARG_TYPE: ArgType = i32::ARG_TYPE;

	fn signature() -> Signature<'static> {
		<i32 as Arg>::signature()
	}
}
impl Append for MatchType {
	fn append_by_ref(&self, i: &mut IterAppend) {
		(*self as i32).append_by_ref(i)
	}
}

impl Arg for RemoteImage {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(i32, i32, i32, bool, i32, i32, Vec<u8>)>("(iiibiiay)")
	}
}
impl RefArg for RemoteImage {
	fn arg_type(&self) -> ArgType {
		Self::ARG_TYPE
	}

	fn signature(&self) -> Signature<'static> {
		<Self as Arg>::signature()
	}

	fn append(&self, i: &mut IterAppend) {
		self.append_by_ref(i)
	}

	fn as_any(&self) -> &dyn Any
	where
		Self: 'static,
	{
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any
	where
		Self: 'static,
	{
		self
	}

	fn box_clone(&self) -> Box<dyn RefArg + 'static> {
		Box::new(self.clone())
	}
}

impl Append for RemoteImage {
	fn append_by_ref(&self, i: &mut IterAppend) {
		i.append((
			&self.width,
			&self.height,
			&self.row_stride,
			&self.has_alpha,
			&self.bits_per_sample,
			&self.channels,
			&self.data,
		))
	}
}

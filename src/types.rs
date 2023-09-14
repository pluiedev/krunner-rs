use std::any::Any;
use std::collections::HashMap;

use dbus::arg::{Append, Arg, ArgType, Dict, IterAppend, PropMap, RefArg, Variant};
use dbus::Signature;

pub type AnyVariant = Variant<Box<dyn RefArg + 'static>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
	pub match_regex: String,
	pub min_letter_count: i32,
	pub trigger_words: Vec<String>,
	pub actions: Vec<Action>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
	pub id: String,
	pub text: String,
	pub icon: String,
	pub ty: MatchType,
	pub relevance: f64,
	pub properties: Properties,
}

// These properties aren't really documented anywhere;
// please read DBusRunner::convertMatches for information on them
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Properties {
	pub urls: Vec<String>,
	pub category: String,
	pub subtext: String,
	pub multiline: bool,
	pub actions: Vec<Action>,
	pub icon_data: Option<RemoteImage>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Action {
	pub id: String,
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

impl Arg for Config {
	const ARG_TYPE: ArgType = ArgType::Array;

	fn signature() -> Signature<'static> {
		assert_sig::<PropMap>("a{sv}")
	}
}
impl Append for Config {
	fn append_by_ref(&self, i: &mut IterAppend) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();
		fields.insert("MatchRegex", Variant(self.match_regex.box_clone()));
		fields.insert("MinLetterCount", Variant(self.min_letter_count.box_clone()));
		fields.insert("TriggerWords", Variant(self.trigger_words.box_clone()));
		fields.insert("Actions", Variant(self.actions.box_clone()));

		Dict::new(fields.iter()).append_by_ref(i)
	}
}

impl Arg for Match {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(String, String, String, MatchType, f64, PropMap)>("(sssida{sv})")
	}
}
impl Append for Match {
	fn append_by_ref(&self, i: &mut IterAppend) {
		i.append((
			&self.id,
			&self.text,
			&self.icon,
			&self.ty,
			&self.relevance,
			&self.properties,
		));
	}
}

impl Default for Properties {
	fn default() -> Self {
		Self {
			urls: vec![],
			category: String::new(),
			subtext: String::new(),
			multiline: false,
			actions: vec![],
			icon_data: None,
		}
	}
}
impl Arg for Properties {
	const ARG_TYPE: ArgType = ArgType::Array;

	fn signature() -> Signature<'static> {
		assert_sig::<PropMap>("a{sv}")
	}
}
impl Append for Properties {
	fn append_by_ref(&self, i: &mut IterAppend) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();
		fields.insert("urls", Variant(self.urls.box_clone()));
		fields.insert("category", Variant(self.category.box_clone()));
		fields.insert("subtext", Variant(self.subtext.box_clone()));
		fields.insert("multiline", Variant(self.multiline.box_clone()));
		fields.insert("actions", Variant(self.actions.box_clone()));

		if let Some(icon_data) = &self.icon_data {
			fields.insert("icon-data", Variant(icon_data.box_clone()));
		}

		Dict::new(fields.iter()).append_by_ref(i)
	}
}

impl Arg for Action {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(String, String, String)>("(sss)")
	}
}
impl RefArg for Action {
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
impl Append for Action {
	fn append_by_ref(&self, i: &mut IterAppend) {
		i.append((&self.id, &self.text, &self.icon_source));
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

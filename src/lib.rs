#![doc = include_str!("../README.md")]
#![cfg_attr(docs_rs, feature(doc_cfg, async_fn_in_trait))]
#![deny(rust_2018_idioms)]

#[path = "async.rs"]
#[cfg(feature = "tokio")]
#[cfg_attr(docs_rs, doc(cfg(feature = "tokio")))]
mod _async;
mod sync;

use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

#[cfg(feature = "tokio")]
pub use _async::*;
use dbus::arg::{Append, Arg, ArgType, Dict, IterAppend, PropMap, RefArg, Variant};
use dbus::Signature;
#[cfg(feature = "derive")]
#[cfg_attr(docs_rs, doc(cfg(feature = "derive")))]
/// Derive macro allowing users to easily generate [`Action`s](Action) for their
/// runners.
///
/// Currently, only enums with unit variants are supported. Each variant has to
/// be tagged with a `#[action]` attribute, which accepts three fields: `id`,
/// for the unique identifier of the action, `title` for the human-friendly name
/// of the action, and `icon` for the name of the action's icon.
///
/// # Example
/// ```
/// #[derive(krunner::Action)]
/// pub enum Action {
/// 	#[action(
/// 		id = "open-in-browser",
/// 		title = "Open in Browser",
/// 		icon = "internet-web-browser"
/// 	)]
/// 	OpenInBrowser,
/// 	#[action(
/// 		id = "save-to-folder",
/// 		title = "Save to Folder",
/// 		icon = "document-save-symbolic"
/// 	)]
/// 	SaveToFolder,
/// }
/// ```
pub use krunner_derive::Action;
pub use sync::*;

/// Trait for actions that the user can perform.
///
/// # Example
/// Using the [derive macro](derive@Action):
/// ```
/// #[derive(krunner::Action)]
/// pub enum Action {
/// 	#[action(
/// 		id = "open-in-browser",
/// 		title = "Open in Browser",
/// 		icon = "internet-web-browser"
/// 	)]
/// 	OpenInBrowser,
/// 	#[action(
/// 		id = "save-to-folder",
/// 		title = "Save to Folder",
/// 		icon = "document-save-symbolic"
/// 	)]
/// 	SaveToFolder,
/// }
/// ```
///
/// The equivalent without using the derive macro:
/// ```
/// use krunner::ActionInfo;
///
/// pub enum Action {
/// 	OpenInBrowser,
/// 	SaveToFolder,
/// }
/// impl krunner::Action for Action {
/// 	fn all() -> &'static [Self] {
/// 		&[Self::OpenInBrowser, Self::SaveToFolder]
/// 	}
///
/// 	fn from_id(s: &str) -> Option<Self> {
/// 		Some(match s {
/// 			"open-in-browser" => Self::OpenInBrowser,
/// 			"save-to-folder" => Self::SaveToFolder,
/// 			_ => return None,
/// 		})
/// 	}
///
/// 	fn to_id(&self) -> String {
/// 		match self {
/// 			Self::OpenInBrowser => "open-in-browser",
/// 			Self::SaveToFolder => "save-to-folder",
/// 		}
/// 		.to_owned()
/// 	}
///
/// 	fn info(&self) -> ActionInfo {
/// 		match self {
/// 			Self::OpenInBrowser => ActionInfo {
/// 				title: "Open in Browser".to_owned(),
/// 				icon: "internet-web-browser".to_owned(),
/// 			},
/// 			Self::SaveToFolder => ActionInfo {
/// 				title: "Save to Folder".to_owned(),
/// 				icon: "document-save-symbolic".to_owned(),
/// 			},
/// 		}
/// 	}
/// }
/// ```
pub trait Action: Sized {
	/// Every action possible of this type.
	fn all() -> &'static [Self];

	/// Tries to get an action by its unique ID.
	fn from_id(s: &str) -> Option<Self>;
	/// Returns the unique ID of the action.
	fn to_id(&self) -> String;
	/// Returns associated information about the action.
	fn info(&self) -> ActionInfo;
}

/// Configuration for a runner.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config<A> {
	/// The filter that a query must pass before attempting a match.
	pub match_filter: Option<MatchFilter>,

	/// The minimum length a query must be before attempting a match.
	/// Queries shorter than this minimum letter count will not be matched
	/// against.
	pub min_letter_count: Option<u32>,

	_phan: PhantomData<A>,
}

/// A filter that a query must pass through before attempting a match.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MatchFilter {
	/// A list of keywords that, if found at the start of a query, would trigger
	/// a match. Also known in KDE docs as "trigger words".
	///
	/// Useful for runners that are only triggered by an initial keyword.
	Keywords(Vec<String>),
	/// A regex that a query must match before attempting a match.
	///
	/// Useful for runners that are only interested in queries of a certain
	/// pattern.
	Regex(String),
}

/// A query match.
#[derive(Debug, Clone, PartialEq)]
pub struct Match<A> {
	/// The unique identifier of this match.
	pub id: String,
	/// The main title text for this match; should be short enough to fit nicely
	/// on one line in a user interface.
	#[doc(alias = "text")]
	pub title: String,
	/// The subtitle of this match.
	///
	/// This is typically a description of the match, or other helpful text.
	#[doc(alias = "subtext")]
	pub subtitle: Option<String>,
	/// The icon of this match.
	pub icon: MatchIcon,
	/// The type of this match.
	pub ty: MatchType,
	/// The relevance of this match, ranging from 0 to 1. Used for sorting
	/// results.
	pub relevance: f64,
	/// URLs associated with this match.
	pub urls: Vec<String>,
	/// The category of this match.
	///
	/// If the category is set to `None`, the name of the runner would be used
	/// as the category instead.
	pub category: Option<String>,
	/// Whether the text should be displayed as a multiline string.
	pub multiline: bool,
	/// List of [actions](crate::Action) that the user can perform for this
	/// match.
	pub actions: Vec<A>,
}

/// The icon displayed for a match.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MatchIcon {
	/// An icon specified by its icon name (e.g. `new-command-alarm`).
	ByName(String),
	/// An icon specified by associated [custom image data](ImageData).
	Custom(ImageData),
}

/// Information related to an action.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionInfo {
	/// The title of the action.
	#[doc(alias = "text")]
	pub title: String,
	/// The name of the icon of the action.
	pub icon: String,
}

/// The image data that KRunner accepts for icons.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ImageData {
	/// The width of the image.
	pub width: i32,
	/// The height of the image.
	pub height: i32,
	/// The row stride (aka the size of one row in bytes) of the image.
	pub row_stride: i32,
	/// Whether the image contains an alpha channel (i.e. transparency
	/// information)
	pub has_alpha: bool,
	/// The format of the image.
	pub format: ImageFormat,
	/// The image data.
	pub data: Vec<u8>,
}

/// The image formats supported by KRunner for icons.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ImageFormat {
	/// 32-bit ARGB.
	Argb32,
	/// 32-bit RGB.
	Rgb32,
}

/// The type of the match.
///
/// The numeric values assigned to each type do have meaning:
/// a higher value corresponds to higher confidence that a
/// match would be relevant for the user.
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
	#[deprecated(since = "KDE Frameworks version 5.99.")]
	InformationalMatch = 50,

	/// A match that represents an action not directly related to activating
	/// the given search term, such as a search in an external tool or command
	/// learning trigger.
	///
	/// Helper matches tend to be generic to the query and should not be
	/// autoactivated just because the user hits "Enter" while typing.
	/// They must be explicitly selected to be activated, but unlike
	/// [`InformationalMatch`](Self::InformationalMatch), they cause
	/// an action to be triggered.
	HelperMatch = 70,

	/// An exact match to the query.
	ExactMatch = 100,
}

//================ IMPL ================//

pub(crate) fn action_as_arg<A: Action>(action: &A) -> (String, String, String) {
	let ActionInfo { title, icon } = action.info();
	(action.to_id(), title, icon)
}

impl<A> Default for Config<A> {
	fn default() -> Self {
		Self {
			match_filter: None,
			min_letter_count: None,
			_phan: PhantomData,
		}
	}
}

impl MatchIcon {
	fn new() -> Self {
		Self::default()
	}
}
impl Default for MatchIcon {
	fn default() -> Self {
		Self::ByName("".to_owned())
	}
}
impl From<String> for MatchIcon {
	fn from(s: String) -> Self {
		Self::ByName(s)
	}
}
impl From<ImageData> for MatchIcon {
	fn from(i: ImageData) -> Self {
		Self::Custom(i)
	}
}

type AnyVariant = Variant<Box<dyn RefArg + 'static>>;

fn assert_sig<T: Arg>(expected: &'static str) -> Signature<'static> {
	let sig = <T as Arg>::signature();
	debug_assert_eq!(&*sig, expected);
	sig
}

impl<A: Action> Arg for Config<A> {
	const ARG_TYPE: ArgType = ArgType::Array;

	fn signature() -> Signature<'static> {
		assert_sig::<PropMap>("a{sv}")
	}
}
impl<A: Action + 'static> Append for Config<A> {
	fn append_by_ref(&self, i: &mut IterAppend<'_>) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();

		match &self.match_filter {
			Some(MatchFilter::Keywords(kws)) => {
				fields.insert("TriggerWords", Variant(kws.box_clone()));
			}
			Some(MatchFilter::Regex(r)) => {
				fields.insert("MatchRegex", Variant(r.box_clone()));
			}
			_ => {}
		}
		if let Some(min_letter_count) = self.min_letter_count {
			fields.insert("MinLetterCount", Variant(min_letter_count.box_clone()));
		}

		let actions: Vec<_> = A::all().iter().map(action_as_arg).collect();
		fields.insert("Actions", Variant(actions.box_clone()));

		Dict::new(fields.iter()).append_by_ref(i)
	}
}

impl<A: Action> Default for Match<A> {
	fn default() -> Self {
		Self {
			id: "".to_owned(),
			title: "".to_owned(),
			subtitle: None,
			icon: MatchIcon::new(),
			ty: MatchType::PossibleMatch,
			relevance: 1.0,
			urls: vec![],
			category: None,
			multiline: false,
			actions: vec![],
		}
	}
}
impl<A: Action> Arg for Match<A> {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(String, String, String, MatchType, f64, PropMap)>("(sssida{sv})")
	}
}
impl<A: Action> Append for Match<A> {
	fn append_by_ref(&self, i: &mut IterAppend<'_>) {
		let mut fields = HashMap::<&'static str, AnyVariant>::new();

		let icon = match &self.icon {
			MatchIcon::ByName(n) => n,
			MatchIcon::Custom(_) => "",
		};

		if !self.urls.is_empty() {
			fields.insert("urls", Variant(self.urls.box_clone()));
		}
		if let Some(category) = &self.category {
			fields.insert("category", Variant(category.box_clone()));
		}
		if let Some(subtext) = &self.subtitle {
			fields.insert("subtext", Variant(subtext.box_clone()));
		}
		if self.multiline {
			fields.insert("multiline", Variant(self.multiline.box_clone()));
		}
		if !self.actions.is_empty() {
			let actions: Vec<_> = self.actions.iter().map(A::to_id).collect();
			fields.insert("actions", Variant(actions.box_clone()));
		}
		if let MatchIcon::Custom(icon) = &self.icon {
			fields.insert("icon-data", Variant(icon.box_clone()));
		}

		let fields = Dict::new(fields.iter());

		i.append((
			&self.id,
			&self.title,
			&icon,
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
	fn append_by_ref(&self, i: &mut IterAppend<'_>) {
		(*self as i32).append_by_ref(i)
	}
}

impl Arg for ImageData {
	const ARG_TYPE: ArgType = ArgType::Struct;

	fn signature() -> Signature<'static> {
		assert_sig::<(i32, i32, i32, bool, i32, i32, Vec<u8>)>("(iiibiiay)")
	}
}
impl RefArg for ImageData {
	fn arg_type(&self) -> ArgType {
		Self::ARG_TYPE
	}

	fn signature(&self) -> Signature<'static> {
		<Self as Arg>::signature()
	}

	fn append(&self, i: &mut IterAppend<'_>) {
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
impl Append for ImageData {
	fn append_by_ref(&self, i: &mut IterAppend<'_>) {
		i.append((
			&self.width,
			&self.height,
			&self.row_stride,
			&self.has_alpha,
			&self.format.bits_per_sample(),
			&self.format.channels(),
			&self.data,
		))
	}
}

impl ImageFormat {
	fn bits_per_sample(&self) -> i32 {
		match self {
			Self::Argb32 | Self::Rgb32 => 8,
		}
	}

	fn channels(&self) -> i32 {
		match self {
			Self::Argb32 => 4,
			Self::Rgb32 => 3,
		}
	}
}

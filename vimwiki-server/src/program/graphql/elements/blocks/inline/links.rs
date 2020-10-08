use super::Region;
use vimwiki::{
    elements,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LE,
};

#[derive(async_graphql::Union, Debug)]
pub enum Link {
    Wiki(WikiLink),
    IndexedInterWiki(IndexedInterWikiLink),
    NamedInterWiki(NamedInterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    ExternalFile(ExternalFileLink),
    Transclusion(TransclusionLink),
}

impl From<LE<elements::Link>> for Link {
    fn from(le: LE<elements::Link>) -> Self {
        match le.element {
            elements::Link::Wiki(x) => {
                Self::from(WikiLink::from(LE::new(x, le.region)))
            }
            elements::Link::InterWiki(elements::InterWikiLink::Indexed(x)) => {
                Self::from(IndexedInterWikiLink::from(LE::new(x, le.region)))
            }
            elements::Link::InterWiki(elements::InterWikiLink::Named(x)) => {
                Self::from(NamedInterWikiLink::from(LE::new(x, le.region)))
            }
            elements::Link::Diary(x) => {
                Self::from(DiaryLink::from(LE::new(x, le.region)))
            }
            elements::Link::Raw(x) => {
                Self::from(RawLink::from(LE::new(x, le.region)))
            }
            elements::Link::ExternalFile(x) => {
                Self::from(ExternalFileLink::from(LE::new(x, le.region)))
            }
            elements::Link::Transclusion(x) => {
                Self::from(TransclusionLink::from(LE::new(x, le.region)))
            }
        }
    }
}

/// Represents a single document wiki link
#[derive(async_graphql::SimpleObject, Debug)]
pub struct WikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LE<elements::WikiLink>> for WikiLink {
    fn from(le: LE<elements::WikiLink>) -> Self {
        Self {
            region: Region::from(le.region),
            is_dir: le.element.is_path_dir(),
            is_local_anchor: le.element.is_local_anchor(),
            path: le.element.path.to_string_lossy().to_string(),
            description: le.element.description.map(Description::from),
            anchor: le.element.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by index
#[derive(async_graphql::SimpleObject, Debug)]
pub struct IndexedInterWikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// The index of the wiki this link is associated with
    index: i32,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LE<elements::IndexedInterWikiLink>> for IndexedInterWikiLink {
    fn from(le: LE<elements::IndexedInterWikiLink>) -> Self {
        Self {
            region: Region::from(le.region),
            index: le.element.index as i32,
            is_dir: le.element.link.is_path_dir(),
            is_local_anchor: le.element.link.is_local_anchor(),
            path: le.element.link.path.to_string_lossy().to_string(),
            description: le.element.link.description.map(Description::from),
            anchor: le.element.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by name
#[derive(async_graphql::SimpleObject, Debug)]
pub struct NamedInterWikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// The name of the wiki this link is associated with
    name: String,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LE<elements::NamedInterWikiLink>> for NamedInterWikiLink {
    fn from(le: LE<elements::NamedInterWikiLink>) -> Self {
        Self {
            region: Region::from(le.region),
            name: le.element.name,
            is_dir: le.element.link.is_path_dir(),
            is_local_anchor: le.element.link.is_local_anchor(),
            path: le.element.link.path.to_string_lossy().to_string(),
            description: le.element.link.description.map(Description::from),
            anchor: le.element.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to a diary entry
#[derive(async_graphql::SimpleObject, Debug)]
pub struct DiaryLink {
    /// The segment of the document this link covers
    region: Region,

    /// Date of diary entry
    date: NaiveDate,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LE<elements::DiaryLink>> for DiaryLink {
    fn from(le: LE<elements::DiaryLink>) -> Self {
        Self {
            region: Region::from(le.region),
            date: le.element.date,
            description: le.element.description.map(Description::from),
            anchor: le.element.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to an external file
#[derive(async_graphql::SimpleObject, Debug)]
pub struct ExternalFileLink {
    /// The segment of the document this link covers
    region: Region,

    /// Scheme associated with the link
    scheme: ExternalFileLinkScheme,

    /// Path to the local file
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,
}

impl From<LE<elements::ExternalFileLink>> for ExternalFileLink {
    fn from(le: LE<elements::ExternalFileLink>) -> Self {
        Self {
            region: Region::from(le.region),
            scheme: ExternalFileLinkScheme::from(le.element.scheme),
            path: le.element.path.to_string_lossy().to_string(),
            description: le.element.description.map(Description::from),
        }
    }
}

/// Represents the scheme associated with an external file link
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

impl From<elements::ExternalFileLinkScheme> for ExternalFileLinkScheme {
    fn from(s: elements::ExternalFileLinkScheme) -> Self {
        match s {
            elements::ExternalFileLinkScheme::Local => Self::Local,
            elements::ExternalFileLinkScheme::File => Self::File,
            elements::ExternalFileLinkScheme::Absolute => Self::Absolute,
        }
    }
}

/// Represents a single document link formed from a raw URI
#[derive(async_graphql::SimpleObject, Debug)]
pub struct RawLink {
    /// The segment of the document this link covers
    region: Region,

    /// The URI representing the link
    uri: Uri,
}

impl From<LE<elements::RawLink>> for RawLink {
    fn from(le: LE<elements::RawLink>) -> Self {
        Self {
            region: Region::from(le.region),
            uri: Uri(le.element.uri),
        }
    }
}

/// Represents a single document transclusion link
#[derive(async_graphql::SimpleObject, Debug)]
pub struct TransclusionLink {
    /// The segment of the document this link covers
    region: Region,

    /// The URI representing the link's content to pull in
    uri: Uri,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Additional properties associated with the link
    properties: Vec<Property>,
}

impl From<LE<elements::TransclusionLink>> for TransclusionLink {
    fn from(mut le: LE<elements::TransclusionLink>) -> Self {
        Self {
            region: Region::from(le.region),
            uri: Uri(le.element.uri),
            description: le.element.description.map(Description::from),
            properties: le
                .element
                .properties
                .drain()
                .map(|(key, value)| Property { key, value })
                .collect(),
        }
    }
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Property {
    key: String,
    value: String,
}

#[derive(Debug)]
pub enum Description {
    Text(String),
    URI(Uri),
}

impl From<elements::Description> for Description {
    fn from(d: elements::Description) -> Self {
        match d {
            elements::Description::Text(x) => Self::Text(x),
            elements::Description::URI(x) => Self::URI(Uri(x)),
        }
    }
}

/// Represents the description of a link
#[async_graphql::Object]
impl Description {
    /// Represents the content of the description if it is text
    async fn text(&self) -> Option<&String> {
        match self {
            Self::Text(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description if it is a URI
    async fn uri(&self) -> Option<&Uri> {
        match self {
            Self::URI(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description
    async fn content(&self) -> String {
        match self {
            Self::Text(ref x) => x.to_string(),
            Self::URI(ref x) => x.0.to_string(),
        }
    }
}

/// Represents anchor for a link
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Anchor {
    /// The pieces of an anchor #one#two#three -> ["one", "two", "three"]
    elements: Vec<String>,
}

impl From<elements::Anchor> for Anchor {
    fn from(a: elements::Anchor) -> Self {
        Self {
            elements: a.elements,
        }
    }
}

#[derive(Debug)]
pub struct Uri(URI<'static>);

/// Represents a traditional URI
#[async_graphql::Object]
impl Uri {
    /// The authority portion of the URI, if it exists
    async fn authority(&self) -> Option<String> {
        self.0.authority().map(|x| x.to_string())
    }

    /// The fragment portion of the URI, if it exists
    async fn fragment(&self) -> Option<String> {
        self.0.fragment().map(|x| x.to_string())
    }

    /// The host portion of the URI, if it exists
    async fn host(&self) -> Option<String> {
        self.0.host().map(|x| x.to_string())
    }

    /// The password portion of the URI, if it exists
    async fn password(&self) -> Option<String> {
        self.0.password().map(|x| x.to_string())
    }

    /// The path of the URI
    async fn path(&self) -> String {
        self.0.path().to_string()
    }

    /// The port portion of the URI, if it exists
    async fn port(&self) -> Option<i32> {
        self.0.port().map(|x| x as i32)
    }

    /// The query portion of the URI, if it exists
    async fn query(&self) -> Option<String> {
        self.0.query().map(|x| x.to_string())
    }

    /// The scheme of the URI
    async fn scheme(&self) -> String {
        self.0.scheme().to_string()
    }

    /// The username portion of the URI, if it exists
    async fn username(&self) -> Option<String> {
        self.0.username().map(|x| x.to_string())
    }

    /// The entire URI
    async fn text(&self) -> String {
        self.0.to_string()
    }
}

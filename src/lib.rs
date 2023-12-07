use convert_case::{Case, Casing};
use quick_xml::{events::*, reader::Reader, writer::Writer};
use semver::{BuildMetadata, Prerelease, Version};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt, fs,
    io::Cursor,
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

#[cfg(test)]
// mod tests;
mod tests;

// Include Modules
mod impls;
pub use impls::*;

mod version_tools;
pub use version_tools::*;

/// Errors that can occur while parsing a ModInfo.xml file
#[derive(Debug, Error)]
pub enum ModinfoError {
    #[error("I/O error occurred: {0}")]
    IoError(std::io::Error),
    #[error("Invalid version: {0}")]
    InvalidVersion(lenient_semver_parser::Error<'static>),
    #[error("File not found")]
    FsNotFound,
    #[error("No modinfo.xml found")]
    NoModinfo,
    #[error("No Author found in modinfo.xml")]
    NoModinfoAuthor,
    #[error("No Description found in modinfo.xml")]
    NoModinfoDescription,
    #[error("No Name found in modinfo.xml")]
    NoModinfoName,
    #[error("No Version found in modinfo.xml")]
    NoModinfoVersion,
    #[error("Unable to determine the version for modinfo.xml")]
    NoModinfoValueVersion,
    #[error("Unknown tag: {0}")]
    UnknownTag(String),
    #[error("Could not write modinfo.xml")]
    WriteError,
    #[error("Could not parse XML: {0}")]
    XMLError(quick_xml::Error),
}

impl From<std::io::Error> for ModinfoError {
    fn from(err: std::io::Error) -> Self {
        ModinfoError::IoError(err)
    }
}
impl From<quick_xml::Error> for ModinfoError {
    fn from(err: quick_xml::Error) -> Self {
        ModinfoError::XMLError(err)
    }
}

impl From<lenient_semver_parser::Error<'static>> for ModinfoError {
    fn from(err: lenient_semver_parser::Error<'static>) -> Self {
        ModinfoError::InvalidVersion(err)
    }
}

/// The version of the modinfo.xml file
///
/// For reference, here are the two formats:
///
/// V1:
/// ```xml
/// <ModInfo>
///   <Name value="SomeMod" />
///   <Description value="Mod to show format of ModInfo v1" />
///   <Author value="Name" />
///   <Version value="0.1.0" />
/// </ModInfo>
/// ```
///
/// V2:
/// ```xml
/// <?xml version="1.0" encoding="utf-8"?>
/// <xml>
///   <Name value="SomeMod" />
///   <DisplayName value="Official Mod Name" />
///   <Version value="0.1.0" />
///   <Description value="Mod to show format of ModInfo v2" />
///   <Author value="Name" />
///   <Website value="https://example.org" />
/// </xml>
/// ```
#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ModinfoVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ModinfoValueMeta {
    version: ModinfoVersion,
    path: PathBuf,
}

impl Default for ModinfoValueMeta {
    fn default() -> Self {
        ModinfoValueMeta {
            version: ModinfoVersion::V2,
            path: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ModinfoValue {
    value: Option<Cow<'static, str>>,
}

impl fmt::Display for ModinfoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(ref value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ModinfoValueVersion {
    value: Version,
    compat: Option<Cow<'static, str>>,
}

impl fmt::Display for ModinfoValueVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = &self.value.to_string();
        let compat = match &self.compat {
            Some(ref value) => value.to_string(),
            None => String::new(),
        };

        if compat.is_empty() {
            write!(f, "{}", version)
        } else {
            write!(f, "{} ({})", version, compat)
        }
    }
}

impl Default for ModinfoValueVersion {
    fn default() -> Self {
        ModinfoValueVersion {
            value: Version::new(0, 1, 0),
            compat: None,
        }
    }
}

/// The main struct for the library
///
/// # Fields
///
/// * `name` - the name of the modlet
/// * `display_name` - the display name of the modlet (v2 only)
/// * `version` - the version of the modlet
/// * `description` - the description of the modlet
/// * `author` - the author of the modlet
/// * `website` - the website of the modlet (v2 only)
///
/// Additionally, version supports an optional `compat` field which can be used to indicate the game's version for the compatibility string
///
/// # Example
///
/// ```rust
/// use modinfo::Modinfo;
/// use std::borrow::Cow;
///
/// let mut modinfo = Modinfo::new();
///
/// modinfo.set_version("0.1.0".to_owned());
/// modinfo.set_value_for("name", "SomeMod");
/// modinfo.set_value_for("display_name", "Some Mod");
/// modinfo.set_value_for("author", "Some Author");
/// modinfo.set_value_for("description", "Some Description");
/// modinfo.set_value_for("website", "https://example.org");
///
/// assert_eq!(modinfo.get_value_for("name"), Some(&Cow::from("SomeMod")));
/// assert_eq!(modinfo.get_value_for("display_name"), Some(&Cow::from("Some Mod")));
/// assert_eq!(modinfo.get_value_for("author"), Some(&Cow::from("Some Author")));
/// assert_eq!(modinfo.get_value_for("description"), Some(&Cow::from("Some Description")));
/// assert_eq!(modinfo.get_value_for("website"), Some(&Cow::from("https://example.org")));
/// assert_eq!(modinfo.get_version(), &semver::Version::new(0, 1, 0));
/// ```
///
#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Modinfo {
    author: ModinfoValue,
    description: ModinfoValue,
    display_name: ModinfoValue,
    name: ModinfoValue,
    version: ModinfoValueVersion,
    website: ModinfoValue,
    meta: ModinfoValueMeta,
}

impl ToString for Modinfo {
    fn to_string(&self) -> String {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
        let is_v2 = ModinfoVersion::V2 == self.meta.version;

        let root_str = match is_v2 {
            true => String::from("xml"),
            false => String::from("ModInfo"),
        };

        if is_v2 {
            writer
                .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
                .unwrap();
        }
        writer.write_event(Event::Start(BytesStart::new(&root_str))).unwrap();

        // inject the attributes here
        for field in ["name", "display_name", "version", "description", "author", "website"] {
            if !is_v2 && (field == "website" || field == "display_name") {
                continue;
            }

            let field_name = field.to_owned().to_case(Case::Pascal);
            let mut elem = BytesStart::new(field_name);
            let value = match field {
                "version" => self.get_version().to_string(),
                _ => match self.get_value_for(field) {
                    Some(value) => value.to_string(),
                    None => String::new(),
                },
            };

            elem.push_attribute(attributes::Attribute {
                key: quick_xml::name::QName(b"value"),
                value: Cow::from(value.clone().into_bytes()),
            });

            if field == "version" && self.version.compat.is_some() {
                elem.push_attribute(attributes::Attribute {
                    key: quick_xml::name::QName(b"compat"),
                    value: Cow::from(self.version.compat.as_ref().unwrap().as_bytes()),
                });
            };

            writer.write_event(Event::Empty(elem)).unwrap();
        }

        writer.write_event(Event::End(BytesEnd::new(&root_str))).unwrap();

        String::from_utf8(writer.into_inner().into_inner()).unwrap()
    }
}

impl FromStr for Modinfo {
    type Err = ModinfoError;

    fn from_str(xml: &str) -> Result<Self, Self::Err> {
        let mut modinfo = Modinfo::default();
        let mut buf: Vec<u8> = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                // Root Element
                Ok(Event::Start(e)) => {
                    modinfo.meta.version = match e.name().as_ref() {
                        b"xml" => ModinfoVersion::V2,
                        _ => ModinfoVersion::V1,
                    }
                }
                // Child Elements (because they have no children)
                Ok(Event::Empty(e)) => {
                    let attributes = parse_attributes(e.attributes());
                    let value = attributes["value"].clone();

                    match e.name().as_ref() {
                        b"Author" => {
                            modinfo.author = ModinfoValue {
                                value: Some(value.into()),
                            }
                        }
                        b"Description" => {
                            modinfo.description = ModinfoValue {
                                value: Some(value.into()),
                            }
                        }
                        b"DisplayName" => {
                            modinfo.display_name = ModinfoValue {
                                value: Some(value.into()),
                            }
                        }
                        b"Name" => {
                            if modinfo.display_name.value.is_none() {
                                modinfo.display_name = ModinfoValue {
                                    value: Some(value.clone().to_case(Case::Title).into()),
                                }
                            }

                            modinfo.name = ModinfoValue {
                                value: Some(value.into()),
                            }
                        }
                        b"Version" => {
                            let mut compat = None;

                            if attributes.contains_key("compat") {
                                compat = Some(attributes["compat"].clone().into());
                            }
                            modinfo.version = ModinfoValueVersion {
                                value: match lenient_semver::parse_into::<Version>(&value) {
                                    Ok(result) => result.clone(),
                                    Err(err) => {
                                        lenient_semver::parse_into::<Version>(format!("0.0.0+{}", err).as_ref())
                                            .unwrap()
                                    }
                                },
                                compat,
                            }
                        }
                        b"Website" => {
                            modinfo.website = ModinfoValue {
                                value: Some(value.into()),
                            }
                        }
                        _ => (),
                    }
                }
                Ok(_) => (),
            }

            buf.clear();
        }

        Ok(modinfo)
    }
}

fn parse_attributes(input: attributes::Attributes) -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    input.map(|a| a.unwrap()).for_each(|a| {
        let key: String = String::from_utf8_lossy(a.key.as_ref()).to_lowercase();
        let value = String::from_utf8(a.value.into_owned()).unwrap();

        attributes.insert(key, value);
    });

    attributes
}

/// Parses a Modinfo.xml file and produces a Modinfo struct
///
/// It will auto-detect the version of the Modinfo.xml file (either V1 or V2)
///
/// # Arguments
///
/// * `file` - a Path-like object pointing to a ModInfo.xml file
///
/// # Returns
///
/// A `Result` containing either a `Modinfo` struct or a `ModinfoError`
///
/// ## Possible ModinfoError
///
/// * `ModinfoError::FsNotFound` - the file does not exist
/// * `ModinfoError::IoError` - an I/O error occurred
/// * `ModinfoError::NoModinfoAuthor` - no Author tag found (required)
/// * `ModinfoError::NoModinfoDescription` - no Description tag found (required)
/// * `ModinfoError::NoModinfoName` - no Name tag found (required)
/// * `ModinfoError::NoModinfoVersion` - no Version value found (required)
/// * `ModinfoError::XMLError` - an error occurred while trying to parse the XML (possibly invalid XML structure?)
///
pub fn parse(file: impl AsRef<Path>) -> Result<Modinfo, ModinfoError> {
    let modinfo = match Path::try_exists(file.as_ref()) {
        Ok(true) => Modinfo::from_str(fs::read_to_string(&file)?.as_ref()),
        Ok(false) => return Err(ModinfoError::FsNotFound),
        Err(err) => return Err(ModinfoError::IoError(err)),
    };

    match modinfo {
        Ok(mut modinfo) => {
            if modinfo.author.value.is_none() {
                return Err(ModinfoError::NoModinfoAuthor);
            }
            if modinfo.description.value.is_none() {
                return Err(ModinfoError::NoModinfoDescription);
            }
            if modinfo.name.value.is_none() {
                return Err(ModinfoError::NoModinfoName);
            }
            if modinfo.version.value.to_string().is_empty() {
                return Err(ModinfoError::NoModinfoVersion);
            }

            // store the original file path in the metadata
            modinfo.meta.path = file.as_ref().to_path_buf();

            Ok(modinfo)
        }
        Err(err) => Err(err),
    }
}

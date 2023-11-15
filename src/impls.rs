use super::*;

impl Modinfo {
    /// Create a new Modinfo instance with default values
    pub fn new() -> Self {
        Modinfo::default()
    }

    /// Write the Modinfo to a file
    /// uses `modinfo_version` to determine which format to use
    pub fn write(&self, file: Option<&Path>) -> Result<(), ModinfoError> {
        match file {
            Some(path) => {
                fs::write(path, self.to_string())?;
            }
            None => {
                fs::write(self.meta.path.clone(), self.to_string())?;
            }
        }

        Ok(())
    }

    /// Retrieve the value for a given field.
    ///
    /// Note: This is not case-sensitive so you can use `get_value_for("Author")` or `get_value_for("author")`
    ///
    /// Please note that `version` is excluded from this function, use `get_version` instead
    ///
    /// ```rust
    /// use modinfo::Modinfo;
    /// use std::borrow::Cow;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_value_for("author", "Joe");
    /// assert_eq!(modinfo.get_value_for("author"), Some(&Cow::from("Joe")));
    /// ```
    pub fn get_value_for<F>(&self, field: F) -> Option<&Cow<'_, str>>
    where
        F: AsRef<str>,
    {
        match field.as_ref().to_lowercase().as_ref() {
            "author" => self.author.value.as_ref(),
            "description" => self.description.value.as_ref(),
            "display_name" => self.display_name.value.as_ref(),
            "name" => self.name.value.as_ref(),
            "website" => self.website.value.as_ref(),
            "compat" => self.version.compat.as_ref(),
            _ => None,
        }
    }

    /// Set the value for a given `field` to `value`
    ///
    /// Note: `field` is not case-sensitive, so you can use `set_value_for("Author", "Joe")` or `get_value_for("author", "Joe")`
    ///
    /// ```rust
    /// use modinfo::Modinfo;
    /// use std::borrow::Cow;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_value_for("name", "MyMod");
    /// assert_eq!(modinfo.get_value_for("name"), Some(&Cow::from("MyMod")));
    /// ```
    pub fn set_value_for(&mut self, field: &str, value: &str) {
        match field.to_lowercase().as_ref() {
            "author" => self.author.value = Some(value.to_owned().into()),
            "description" => self.description.value = Some(value.to_owned().into()),
            "display_name" => self.display_name.value = Some(value.to_owned().into()),
            "name" => self.name.value = Some(value.to_owned().into()),
            "website" => self.website.value = Some(value.to_owned().into()),
            "version" => self.version.value.set_version(value),
            "compat" => self.version.compat = Some(value.to_owned().into()),
            _ => (),
        }
    }

    /// Retrieve the value for the version field included the ModInfo
    ///
    /// Note: This is the version of the modlet, not the version of the ModInfo file format
    /// Please see `get_modinfo_version` for that.
    ///
    /// ```rust
    /// use modinfo::Modinfo;
    ///
    /// let modinfo = Modinfo::default();
    /// assert_eq!(modinfo.get_version(), &semver::Version::new(0, 1, 0));
    /// ```
    pub fn get_version(&self) -> &Version {
        &self.version.value
    }

    /// Sets the version field inside the modinfo.xml file (modlet version)
    ///
    /// ```rust
    /// use modinfo::Modinfo;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3".to_owned());
    /// assert_eq!(modinfo.get_version(), &semver::Version::new(1, 2, 3));
    /// ```
    pub fn set_version(&mut self, version: String) {
        self.set_value_for("version", &version)
    }

    /// Retrieves the current version of the ModInfo.xml file (V1 or V2)
    ///
    /// returns a `ModinfoVersion` enum:
    ///     ModinfoVersion::V1
    ///     ModinfoVersion::V2
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    ///
    /// let mut modinfo = Modinfo::default();
    /// assert_eq!(modinfo.get_modinfo_version(), ModinfoVersion::V2);
    /// ```
    pub fn get_modinfo_version(&self) -> ModinfoVersion {
        self.meta.version
    }

    /// Sets the version of the ModInfo.xml file itesle (V1 or V2)
    ///
    /// Accepts a `ModinfoVersion` enum:
    ///     ModinfoVersion::V1
    ///     ModinfoVersion::V2
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_modinfo_version(ModinfoVersion::V1);
    /// assert_eq!(modinfo.get_modinfo_version(), ModinfoVersion::V1);
    /// ```
    pub fn set_modinfo_version(&mut self, version: ModinfoVersion) {
        self.meta.version = version
    }

    /// Retrieves the current modinfo.xml file path
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_file_path(PathBuf::from("modinfo.xml"));
    /// assert_eq!(modinfo.get_file_path(), &PathBuf::from("modinfo.xml"));
    /// ```
    pub fn get_file_path(&self) -> &PathBuf {
        &self.meta.path
    }

    /// Sets the modinfo.xml file path
    ///
    /// This is normally set automatically when a file is parsed, but can be set manually
    /// such as when creating a new modinfo.xml file.
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_file_path(std::path::PathBuf::from("modinfo.xml"));
    /// assert_eq!(modinfo.get_file_path(), &PathBuf::from("modinfo.xml"));
    /// ```
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.meta.path = path.clone();
    }

    /// Increases the Major version number by 1,
    /// sets Minor and Patch to 0, and removes any pre or build data.
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3-foo+bar".to_owned());
    /// modinfo.bump_version_major();
    /// assert_eq!(modinfo.get_version(), &semver::Version::new(2, 0, 0));
    /// ```
    pub fn bump_version_major(&mut self) {
        self.version.value.bump_major()
    }

    /// Increases the Minor version number by 1,
    /// sets Patch to 0, and removes any pre or build data.
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3-foo+bar".to_owned());
    /// modinfo.bump_version_minor();
    /// assert_eq!(modinfo.get_version(), &semver::Version::new(1, 3, 0));
    /// ```
    pub fn bump_version_minor(&mut self) {
        self.version.value.bump_minor()
    }

    /// Increases the Patch version number by 1,
    /// and removes any pre or build data.
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3-foo+bar".to_owned());
    /// modinfo.bump_version_patch();
    /// assert_eq!(modinfo.get_version(), &semver::Version::new(1, 2, 4));
    /// ```
    pub fn bump_version_patch(&mut self) {
        self.version.value.bump_patch()
    }

    /// Adds a pre-release version to the version field
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3".to_owned());
    /// modinfo.add_version_pre("foo");
    /// assert_eq!(modinfo.get_version(), &semver::Version::parse("1.2.3-foo").unwrap());
    /// ```
    pub fn add_version_pre(&mut self, pre: &str) {
        self.version.value.add_pre(pre)
    }

    /// Adds build data to the version field
    ///
    /// ```rust
    /// use modinfo::{Modinfo, ModinfoVersion};
    /// use std::path::PathBuf;
    ///
    /// let mut modinfo = Modinfo::default();
    /// modinfo.set_version("1.2.3".to_owned());
    /// modinfo.add_version_build("bar");
    /// assert_eq!(modinfo.get_version(), &semver::Version::parse("1.2.3+bar").unwrap());
    /// ```
    pub fn add_version_build(&mut self, build: &str) {
        self.version.value.add_build(build)
    }
}

//! Path-based asset identifiers.

use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;
use std::path::Path;

// =============================================================================
// AssetPath
// =============================================================================

/// A path identifier for assets.
///
/// `AssetPath` represents the path used to load an asset, supporting both
/// owned and borrowed string data. It provides utility methods for working
/// with asset paths.
///
/// # Path Format
///
/// Asset paths use forward slashes as separators, regardless of platform:
/// - `textures/player.png`
/// - `audio/music/theme.ogg`
/// - `shaders/basic.vert`
///
/// # FFI Considerations
///
/// For FFI, convert to a C string using `as_str()` and standard FFI string
/// handling. The path does not include a null terminator by default.
///
/// # Example
///
/// ```
/// use goud_engine::assets::AssetPath;
///
/// let path = AssetPath::new("textures/player.png");
///
/// assert_eq!(path.as_str(), "textures/player.png");
/// assert_eq!(path.file_name(), Some("player.png"));
/// assert_eq!(path.extension(), Some("png"));
/// assert_eq!(path.directory(), Some("textures"));
///
/// // From owned string
/// let owned = AssetPath::from_string("audio/sfx/jump.wav".to_string());
/// assert_eq!(owned.extension(), Some("wav"));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AssetPath<'a> {
    /// The path string, either borrowed or owned.
    path: Cow<'a, str>,
}

impl<'a> AssetPath<'a> {
    /// Creates a new asset path from a string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::new("textures/player.png");
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    #[inline]
    pub fn new(path: &'a str) -> Self {
        Self {
            path: Cow::Borrowed(path),
        }
    }

    /// Creates a new asset path from an owned string.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::from_string("textures/player.png".to_string());
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    #[inline]
    pub fn from_string(path: String) -> AssetPath<'static> {
        AssetPath {
            path: Cow::Owned(path),
        }
    }

    /// Returns the path as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Returns `true` if the path is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Returns the length of the path in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// Returns the file name component of the path.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("textures/player.png").file_name(), Some("player.png"));
    /// assert_eq!(AssetPath::new("player.png").file_name(), Some("player.png"));
    /// assert_eq!(AssetPath::new("textures/").file_name(), None);
    /// ```
    pub fn file_name(&self) -> Option<&str> {
        let path = self.path.as_ref();
        if path.ends_with('/') {
            return None;
        }
        path.rsplit('/').next().filter(|s| !s.is_empty())
    }

    /// Returns the file extension, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("player.png").extension(), Some("png"));
    /// assert_eq!(AssetPath::new("textures/player.png").extension(), Some("png"));
    /// assert_eq!(AssetPath::new("Makefile").extension(), None);
    /// assert_eq!(AssetPath::new(".gitignore").extension(), None);
    /// ```
    pub fn extension(&self) -> Option<&str> {
        let file_name = self.file_name()?;
        let dot_pos = file_name.rfind('.')?;

        // Handle hidden files like ".gitignore" (no extension)
        if dot_pos == 0 {
            return None;
        }

        Some(&file_name[dot_pos + 1..])
    }

    /// Returns the directory component of the path.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("textures/player.png").directory(), Some("textures"));
    /// assert_eq!(AssetPath::new("a/b/c/file.txt").directory(), Some("a/b/c"));
    /// assert_eq!(AssetPath::new("file.txt").directory(), None);
    /// ```
    pub fn directory(&self) -> Option<&str> {
        let path = self.path.as_ref();
        let pos = path.rfind('/')?;
        if pos == 0 {
            return None;
        }
        Some(&path[..pos])
    }

    /// Returns the file stem (file name without extension).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("player.png").stem(), Some("player"));
    /// assert_eq!(AssetPath::new("textures/player.png").stem(), Some("player"));
    /// assert_eq!(AssetPath::new("archive.tar.gz").stem(), Some("archive.tar"));
    /// assert_eq!(AssetPath::new(".gitignore").stem(), Some(".gitignore"));
    /// ```
    pub fn stem(&self) -> Option<&str> {
        let file_name = self.file_name()?;
        if let Some(dot_pos) = file_name.rfind('.') {
            if dot_pos == 0 {
                // Hidden file with no extension
                Some(file_name)
            } else {
                Some(&file_name[..dot_pos])
            }
        } else {
            // No extension
            Some(file_name)
        }
    }

    /// Converts this path to an owned `AssetPath<'static>`.
    ///
    /// If the path is already owned, this is a no-op. If borrowed,
    /// the string is cloned.
    pub fn into_owned(self) -> AssetPath<'static> {
        AssetPath {
            path: Cow::Owned(self.path.into_owned()),
        }
    }

    /// Creates an `AssetPath` from a `std::path::Path`.
    ///
    /// Converts backslashes to forward slashes for platform consistency.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    /// use std::path::Path;
    ///
    /// let path = AssetPath::from_path(Path::new("textures/player.png"));
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    pub fn from_path(path: &Path) -> AssetPath<'static> {
        let path_str = path.to_string_lossy();
        // Normalize to forward slashes
        let normalized = path_str.replace('\\', "/");
        AssetPath::from_string(normalized)
    }

    /// Joins this path with another path component.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let base = AssetPath::new("textures");
    /// let full = base.join("player.png");
    /// assert_eq!(full.as_str(), "textures/player.png");
    ///
    /// // Handles trailing slashes
    /// let base = AssetPath::new("textures/");
    /// let full = base.join("player.png");
    /// assert_eq!(full.as_str(), "textures/player.png");
    /// ```
    pub fn join(&self, other: &str) -> AssetPath<'static> {
        let base = self.path.trim_end_matches('/');
        let other = other.trim_start_matches('/');

        if base.is_empty() {
            AssetPath::from_string(other.to_string())
        } else if other.is_empty() {
            AssetPath::from_string(base.to_string())
        } else {
            AssetPath::from_string(format!("{}/{}", base, other))
        }
    }

    /// Returns the path with a different extension.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::new("textures/player.png");
    /// let new_path = path.with_extension("jpg");
    /// assert_eq!(new_path.as_str(), "textures/player.jpg");
    ///
    /// // Add extension to file without one
    /// let path = AssetPath::new("Makefile");
    /// let new_path = path.with_extension("bak");
    /// assert_eq!(new_path.as_str(), "Makefile.bak");
    /// ```
    pub fn with_extension(&self, ext: &str) -> AssetPath<'static> {
        if let Some(stem) = self.stem() {
            if let Some(dir) = self.directory() {
                AssetPath::from_string(format!("{}/{}.{}", dir, stem, ext))
            } else {
                AssetPath::from_string(format!("{}.{}", stem, ext))
            }
        } else {
            // No file name, just append
            AssetPath::from_string(format!("{}.{}", self.path, ext))
        }
    }
}

impl<'a> fmt::Debug for AssetPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetPath({:?})", self.path)
    }
}

impl<'a> fmt::Display for AssetPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl<'a> AsRef<str> for AssetPath<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.path
    }
}

impl<'a> From<&'a str> for AssetPath<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl From<String> for AssetPath<'static> {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<'a> PartialEq<str> for AssetPath<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.path.as_ref() == other
    }
}

impl<'a> PartialEq<&str> for AssetPath<'a> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.path.as_ref() == *other
    }
}

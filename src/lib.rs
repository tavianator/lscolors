//! A library for colorizing paths according to the `LS_COLORS` environment variable.
//!
//! # Example
//! ```
//! use lscolors::{LsColors, Style};
//!
//! let lscolors = LsColors::from_env().unwrap_or_default();
//!
//! let path = "some/folder/archive.zip";
//! let style = lscolors.style_for_path(path);
//!
//! // If you want to use `ansi_term`:
//! # #[cfg(features = "ansi_term")]
//! # {
//! let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
//! println!("{}", ansi_style.paint(path));
//! # }
//! ```

mod fs;
pub mod style;

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

pub use crate::style::{Color, FontStyle, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Indicator {
    /// `no`: Normal (non-filename) text
    Normal,

    /// `fi`: Regular file
    RegularFile,

    /// `di`: Directory
    Directory,

    /// `ln`: Symbolic link
    SymbolicLink,

    /// `pi`: Named pipe or FIFO
    FIFO,

    /// `so`: Socket
    Socket,

    /// `do`: Door (IPC connection to another program)
    Door,

    /// `bd`: Block-oriented device
    BlockDevice,

    /// `cd`: Character-oriented device
    CharacterDevice,

    /// `or`: A broken symbolic link
    OrphanedSymbolicLink,

    /// `su`: A file that is setuid (`u+s`)
    Setuid,

    /// `sg`: A file that is setgid (`g+s`)
    Setgid,

    /// `st`: A directory that is sticky and other-writable (`+t`, `o+w`)
    Sticky,

    /// `ow`: A directory that is not sticky and other-writeable (`o+w`)
    OtherWritable,

    /// `tw`: A directory that is sticky and other-writable (`+t`, `o+w`)
    StickyAndOtherWritable,

    /// `ex`: Executable file
    ExecutableFile,

    /// `mi`: Missing file
    MissingFile,

    /// `ca`: File with capabilities set
    Capabilities,

    /// `mh`: File with multiple hard links
    MultipleHardLinks,

    /// `lc`: Code that is printed before the color sequence
    LeftCode,

    /// `rc`: Code that is printed after the color sequence
    RightCode,

    /// `ec`: End code
    EndCode,

    /// `rs`: Code to reset to ordinary colors
    Reset,

    /// `cl`: Code to clear to the end of the line
    ClearLine,
}

impl Indicator {
    pub fn from(indicator: &str) -> Option<Indicator> {
        match indicator {
            "no" => Some(Indicator::Normal),
            "fi" => Some(Indicator::RegularFile),
            "di" => Some(Indicator::Directory),
            "ln" => Some(Indicator::SymbolicLink),
            "pi" => Some(Indicator::FIFO),
            "so" => Some(Indicator::Socket),
            "do" => Some(Indicator::Door),
            "bd" => Some(Indicator::BlockDevice),
            "cd" => Some(Indicator::CharacterDevice),
            "or" => Some(Indicator::OrphanedSymbolicLink),
            "su" => Some(Indicator::Setuid),
            "sg" => Some(Indicator::Setgid),
            "st" => Some(Indicator::Sticky),
            "ow" => Some(Indicator::OtherWritable),
            "tw" => Some(Indicator::StickyAndOtherWritable),
            "ex" => Some(Indicator::ExecutableFile),
            "mi" => Some(Indicator::MissingFile),
            "ca" => Some(Indicator::Capabilities),
            "mh" => Some(Indicator::MultipleHardLinks),
            "lc" => Some(Indicator::LeftCode),
            "rc" => Some(Indicator::RightCode),
            "ec" => Some(Indicator::EndCode),
            "rs" => Some(Indicator::Reset),
            "cl" => Some(Indicator::ClearLine),
            _ => None,
        }
    }
}

type FileNameSuffix = String;

/// Iterator over the path components with their respective style.
pub struct StyledComponents<'a> {
    /// Reference to the underlying LsColors object
    lscolors: &'a LsColors,

    /// Full path to the current component
    component_path: PathBuf,

    /// Underlying iterator over the path components
    components: std::iter::Peekable<std::path::Components<'a>>,
}

impl<'a> Iterator for StyledComponents<'a> {
    type Item = (OsString, Option<&'a Style>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(component) = self.components.next() {
            let mut component_str = component.as_os_str().to_os_string();

            self.component_path.push(&component_str);
            let style = self.lscolors.style_for_path(&self.component_path);

            if self.components.peek().is_some() {
                match component {
                    // Prefix needs no separator, as it is always followed by RootDir.
                    // RootDir is already a separator.
                    Component::Prefix(_) | Component::RootDir => {}
                    // Everything else uses a separator that is painted the same way as the component.
                    Component::CurDir | Component::ParentDir | Component::Normal(_) => {
                        component_str.push(MAIN_SEPARATOR.to_string());
                    }
                }
            }

            Some((component_str, style))
        } else {
            None
        }
    }
}

const LS_COLORS_DEFAULT: &str = "rs=0:di=01;34:ln=01;36:mh=00:pi=40;33:so=01;35:do=01;35:bd=40;33;01:cd=40;33;01:or=40;31;01:mi=00:su=37;41:sg=30;43:ca=30;41:tw=30;42:ow=34;42:st=37;44:ex=01;32:*.tar=01;31:*.tgz=01;31:*.arc=01;31:*.arj=01;31:*.taz=01;31:*.lha=01;31:*.lz4=01;31:*.lzh=01;31:*.lzma=01;31:*.tlz=01;31:*.txz=01;31:*.tzo=01;31:*.t7z=01;31:*.zip=01;31:*.z=01;31:*.dz=01;31:*.gz=01;31:*.lrz=01;31:*.lz=01;31:*.lzo=01;31:*.xz=01;31:*.zst=01;31:*.tzst=01;31:*.bz2=01;31:*.bz=01;31:*.tbz=01;31:*.tbz2=01;31:*.tz=01;31:*.deb=01;31:*.rpm=01;31:*.jar=01;31:*.war=01;31:*.ear=01;31:*.sar=01;31:*.rar=01;31:*.alz=01;31:*.ace=01;31:*.zoo=01;31:*.cpio=01;31:*.7z=01;31:*.rz=01;31:*.cab=01;31:*.wim=01;31:*.swm=01;31:*.dwm=01;31:*.esd=01;31:*.jpg=01;35:*.jpeg=01;35:*.mjpg=01;35:*.mjpeg=01;35:*.gif=01;35:*.bmp=01;35:*.pbm=01;35:*.pgm=01;35:*.ppm=01;35:*.tga=01;35:*.xbm=01;35:*.xpm=01;35:*.tif=01;35:*.tiff=01;35:*.png=01;35:*.svg=01;35:*.svgz=01;35:*.mng=01;35:*.pcx=01;35:*.mov=01;35:*.mpg=01;35:*.mpeg=01;35:*.m2v=01;35:*.mkv=01;35:*.webm=01;35:*.ogm=01;35:*.mp4=01;35:*.m4v=01;35:*.mp4v=01;35:*.vob=01;35:*.qt=01;35:*.nuv=01;35:*.wmv=01;35:*.asf=01;35:*.rm=01;35:*.rmvb=01;35:*.flc=01;35:*.avi=01;35:*.fli=01;35:*.flv=01;35:*.gl=01;35:*.dl=01;35:*.xcf=01;35:*.xwd=01;35:*.yuv=01;35:*.cgm=01;35:*.emf=01;35:*.ogv=01;35:*.ogx=01;35:*.aac=00;36:*.au=00;36:*.flac=00;36:*.m4a=00;36:*.mid=00;36:*.midi=00;36:*.mka=00;36:*.mp3=00;36:*.mpc=00;36:*.ogg=00;36:*.ra=00;36:*.wav=00;36:*.oga=00;36:*.opus=00;36:*.spx=00;36:*.xspf=00;36:";

/// Holds information about how different file system entries should be colorized / styled.
#[derive(Debug, Clone)]
pub struct LsColors {
    indicator_mapping: HashMap<Indicator, Style>,

    // Note: you might expect to see a `HashMap` for `suffix_mapping` as well, but we need to
    // preserve the exact order of the mapping in order to be consistent with `ls`.
    suffix_mapping: Vec<(FileNameSuffix, Style)>,
}

impl Default for LsColors {
    /// Constructs a default `LsColors` instance with some default styles. See `man dircolors` for
    /// information about the default styles and colors.
    fn default() -> Self {
        let mut lscolors = LsColors::empty();
        lscolors.add_from_string(LS_COLORS_DEFAULT);
        lscolors
    }
}

impl LsColors {
    /// Construct an empty [`LsColors`](struct.LsColors.html) instance with no pre-defined styles.
    pub fn empty() -> Self {
        LsColors {
            indicator_mapping: HashMap::new(),
            suffix_mapping: vec![],
        }
    }

    /// Creates a new [`LsColors`](struct.LsColors.html) instance from the `LS_COLORS` environment
    /// variable. The basis for this is a default style as constructed via the `Default`
    /// implementation.
    pub fn from_env() -> Option<Self> {
        env::var("LS_COLORS")
            .ok()
            .as_ref()
            .map(|s| Self::from_string(s))
    }

    /// Creates a new [`LsColors`](struct.LsColors.html) instance from the given string.
    pub fn from_string(input: &str) -> Self {
        let mut lscolors = LsColors::default();
        lscolors.add_from_string(input);
        lscolors
    }

    fn add_from_string(&mut self, input: &str) {
        for entry in input.split(':') {
            let parts: Vec<_> = entry.split('=').collect();

            if let Some([entry, ansi_style]) = parts.get(0..2) {
                let style = Style::from_ansi_sequence(ansi_style);
                if let Some(suffix) = entry.strip_prefix('*') {
                    if let Some(style) = style {
                        self.suffix_mapping
                            .push((suffix.to_string().to_ascii_lowercase(), style));
                    }
                } else if let Some(indicator) = Indicator::from(entry) {
                    if let Some(style) = style {
                        self.indicator_mapping.insert(indicator, style);
                    } else {
                        self.indicator_mapping.remove(&indicator);
                    }
                }
            }
        }
    }

    /// Get the ANSI style for a given path.
    ///
    /// *Note:* this function calls `Path::symlink_metadata` internally. If you already happen to
    /// have the `Metadata` available, use [`style_for_path_with_metadata`](#method.style_for_path_with_metadata).
    pub fn style_for_path<P: AsRef<Path>>(&self, path: P) -> Option<&Style> {
        let metadata = path.as_ref().symlink_metadata().ok();
        self.style_for_path_with_metadata(path, metadata.as_ref())
    }

    /// Check if an indicator has an associated color.
    fn has_color_for(&self, indicator: Indicator) -> bool {
        self.indicator_mapping.contains_key(&indicator)
    }

    /// Get the indicator type for a path with corresponding metadata.
    fn indicator_for(&self, path: &Path, metadata: Option<&std::fs::Metadata>) -> Indicator {
        if let Some(metadata) = metadata {
            let file_type = metadata.file_type();

            if file_type.is_file() {
                let mode = crate::fs::mode(metadata);
                let nlink = crate::fs::nlink(metadata);

                if self.has_color_for(Indicator::Setuid) && mode & 0o4000 != 0 {
                    Indicator::Setuid
                } else if self.has_color_for(Indicator::Setgid) && mode & 0o2000 != 0 {
                    Indicator::Setgid
                } else if self.has_color_for(Indicator::ExecutableFile) && mode & 0o0111 != 0 {
                    Indicator::ExecutableFile
                } else if self.has_color_for(Indicator::MultipleHardLinks) && nlink > 1 {
                    Indicator::MultipleHardLinks
                } else {
                    Indicator::RegularFile
                }
            } else if file_type.is_dir() {
                let mode = crate::fs::mode(metadata);

                if self.has_color_for(Indicator::StickyAndOtherWritable) && mode & 0o1002 == 0o1002
                {
                    Indicator::StickyAndOtherWritable
                } else if self.has_color_for(Indicator::OtherWritable) && mode & 0o0002 != 0 {
                    Indicator::OtherWritable
                } else if self.has_color_for(Indicator::Sticky) && mode & 0o1000 != 0 {
                    Indicator::Sticky
                } else {
                    Indicator::Directory
                }
            } else if file_type.is_symlink() {
                // This works because `Path::exists` traverses symlinks.
                if self.has_color_for(Indicator::OrphanedSymbolicLink) && !path.exists() {
                    return Indicator::OrphanedSymbolicLink;
                }

                Indicator::SymbolicLink
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;

                    if file_type.is_fifo() {
                        return Indicator::FIFO;
                    }
                    if file_type.is_socket() {
                        return Indicator::Socket;
                    }
                    if file_type.is_block_device() {
                        return Indicator::BlockDevice;
                    }
                    if file_type.is_char_device() {
                        return Indicator::CharacterDevice;
                    }
                }

                // Treat files of unknown type as errors
                Indicator::MissingFile
            }
        } else {
            // Default to a regular file, so we still try the suffix map when no metadata is available
            Indicator::RegularFile
        }
    }

    /// Get the ANSI style for a path, given the corresponding `Metadata` struct.
    ///
    /// *Note:* The `Metadata` struct must have been acquired via `Path::symlink_metadata` in
    /// order to colorize symbolic links correctly.
    pub fn style_for_path_with_metadata<P: AsRef<Path>>(
        &self,
        path: P,
        metadata: Option<&std::fs::Metadata>,
    ) -> Option<&Style> {
        let indicator = self.indicator_for(path.as_ref(), metadata);

        if indicator == Indicator::RegularFile {
            // Note: using '.to_str()' here means that filename
            // matching will not work with invalid-UTF-8 paths.
            let filename = path.as_ref().file_name()?.to_str()?.to_ascii_lowercase();

            // We need to traverse LS_COLORS from back to front
            // to be consistent with `ls`:
            for (suffix, style) in self.suffix_mapping.iter().rev() {
                // Note: For some reason, 'ends_with' is much
                // slower if we omit `.as_str()` here:
                if filename.ends_with(suffix.as_str()) {
                    return Some(style);
                }
            }
        }

        self.style_for_indicator(indicator)
    }

    /// Get ANSI styles for each component of a given path. Components already include the path
    /// separator symbol, if required. For a path like `foo/bar/test.md`, this would return an
    /// iterator over three pairs for the three path components `foo/`, `bar/` and `test.md`
    /// together with their respective styles.
    pub fn style_for_path_components<'a>(&'a self, path: &'a Path) -> StyledComponents<'a> {
        StyledComponents {
            lscolors: self,
            component_path: PathBuf::new(),
            components: path.components().peekable(),
        }
    }

    /// Get the ANSI style for a certain `Indicator` (regular file, directory, symlink, ...). Note
    /// that this function implements a fallback logic for some of the indicators (just like `ls`).
    /// For example, the style for `mi` (missing file) falls back to `or` (orphaned symbolic link)
    /// if it has not been specified explicitly.
    pub fn style_for_indicator(&self, indicator: Indicator) -> Option<&Style> {
        self.indicator_mapping
            .get(&indicator)
            .or_else(|| {
                self.indicator_mapping.get(&match indicator {
                    Indicator::Setuid
                    | Indicator::Setgid
                    | Indicator::ExecutableFile
                    | Indicator::MultipleHardLinks => Indicator::RegularFile,

                    Indicator::StickyAndOtherWritable
                    | Indicator::OtherWritable
                    | Indicator::Sticky => Indicator::Directory,

                    Indicator::OrphanedSymbolicLink => Indicator::SymbolicLink,

                    Indicator::MissingFile => Indicator::OrphanedSymbolicLink,

                    _ => indicator,
                })
            })
            .or_else(|| self.indicator_mapping.get(&Indicator::Normal))
    }
}

#[cfg(test)]
mod tests {
    use crate::style::{Color, FontStyle, Style};
    use crate::{Indicator, LsColors};

    use std::fs::{self, File};
    use std::path::{Path, PathBuf};

    #[test]
    fn basic_usage() {
        let lscolors = LsColors::default();

        let style_dir = lscolors.style_for_indicator(Indicator::Directory).unwrap();
        assert_eq!(FontStyle::bold(), style_dir.font_style);
        assert_eq!(Some(Color::Blue), style_dir.foreground);
        assert_eq!(None, style_dir.background);

        let style_symlink = lscolors
            .style_for_indicator(Indicator::SymbolicLink)
            .unwrap();
        assert_eq!(FontStyle::bold(), style_symlink.font_style);
        assert_eq!(Some(Color::Cyan), style_symlink.foreground);
        assert_eq!(None, style_symlink.background);

        let style_rs = lscolors.style_for_path("test.wav").unwrap();
        assert_eq!(FontStyle::default(), style_rs.font_style);
        assert_eq!(Some(Color::Cyan), style_rs.foreground);
        assert_eq!(None, style_rs.background);
    }

    #[test]
    fn style_for_path_uses_correct_ordering() {
        let lscolors = LsColors::from_string("*.foo=01;35:*README.foo=33;44");

        let style_foo = lscolors.style_for_path("some/folder/dummy.foo").unwrap();
        assert_eq!(FontStyle::bold(), style_foo.font_style);
        assert_eq!(Some(Color::Magenta), style_foo.foreground);
        assert_eq!(None, style_foo.background);

        let style_readme = lscolors
            .style_for_path("some/other/folder/README.foo")
            .unwrap();
        assert_eq!(FontStyle::default(), style_readme.font_style);
        assert_eq!(Some(Color::Yellow), style_readme.foreground);
        assert_eq!(Some(Color::Blue), style_readme.background);
    }

    #[test]
    fn style_for_path_uses_lowercase_matching() {
        let lscolors = LsColors::from_string("*.O=01;35");

        let style_artifact = lscolors.style_for_path("artifact.o").unwrap();
        assert_eq!(FontStyle::bold(), style_artifact.font_style);
        assert_eq!(Some(Color::Magenta), style_artifact.foreground);
        assert_eq!(None, style_artifact.background);
    }

    #[test]
    fn default_styles_should_be_preserved() {
        // Setting an unrelated style should not influence the default
        // style for "directory" (below)
        let lscolors = LsColors::from_string("ex=01:");

        let style_dir = lscolors.style_for_indicator(Indicator::Directory).unwrap();
        assert_eq!(FontStyle::bold(), style_dir.font_style);
        assert_eq!(Some(Color::Blue), style_dir.foreground);
        assert_eq!(None, style_dir.background);
    }

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("temporary directory")
    }

    fn create_file<P: AsRef<Path>>(path: P) -> PathBuf {
        File::create(&path).expect("temporary file");
        path.as_ref().to_path_buf()
    }

    fn create_dir<P: AsRef<Path>>(path: P) -> PathBuf {
        fs::create_dir(&path).expect("temporary directory");
        path.as_ref().to_path_buf()
    }

    fn get_default_style<P: AsRef<Path>>(path: P) -> Option<Style> {
        let lscolors = LsColors::default();
        lscolors.style_for_path(path).cloned()
    }

    #[cfg(unix)]
    fn create_symlink<P: AsRef<Path>>(from: P, to: P) {
        std::os::unix::fs::symlink(from, to).expect("temporary symlink");
    }

    #[cfg(windows)]
    fn create_symlink<P: AsRef<Path>>(src: P, dst: P) {
        if src.as_ref().is_dir() {
            std::os::windows::fs::symlink_dir(src, dst).expect("temporary symlink");
        } else {
            std::os::windows::fs::symlink_file(src, dst).expect("temporary symlink");
        }
    }

    #[test]
    fn style_for_directory() {
        let tmp_dir = temp_dir();
        let style = get_default_style(tmp_dir.path()).unwrap();
        assert_eq!(Some(Color::Blue), style.foreground);
    }

    #[test]
    fn style_for_file() {
        let tmp_dir = temp_dir();
        let tmp_file_path = create_file(tmp_dir.path().join("test-file"));
        let style = get_default_style(tmp_file_path);
        assert_eq!(None, style);
    }

    #[test]
    fn style_for_symlink() {
        let tmp_dir = temp_dir();
        let tmp_file_path = create_file(tmp_dir.path().join("test-file"));
        let tmp_symlink_path = tmp_dir.path().join("test-symlink");

        create_symlink(&tmp_file_path, &tmp_symlink_path);

        let style = get_default_style(tmp_symlink_path).unwrap();
        assert_eq!(Some(Color::Cyan), style.foreground);
    }

    #[test]
    fn style_for_broken_symlink() {
        let tmp_dir = temp_dir();
        let tmp_file_path = tmp_dir.path().join("non-existing-file");
        let tmp_symlink_path = tmp_dir.path().join("broken-symlink");

        create_symlink(&tmp_file_path, &tmp_symlink_path);

        let style = get_default_style(tmp_symlink_path).unwrap();
        assert_eq!(Some(Color::Red), style.foreground);
    }

    #[test]
    fn style_for_missing_file() {
        let lscolors1 = LsColors::from_string("mi=01:or=33;44");

        let style_missing = lscolors1
            .style_for_indicator(Indicator::MissingFile)
            .unwrap();
        assert_eq!(FontStyle::bold(), style_missing.font_style);

        let lscolors2 = LsColors::from_string("or=33;44");

        let style_missing = lscolors2
            .style_for_indicator(Indicator::MissingFile)
            .unwrap();
        assert_eq!(Some(Color::Yellow), style_missing.foreground);

        let lscolors3 = LsColors::from_string("or=33;44:mi=00");

        let style_missing = lscolors3
            .style_for_indicator(Indicator::MissingFile)
            .unwrap();
        assert_eq!(Some(Color::Yellow), style_missing.foreground);
    }

    #[cfg(unix)]
    #[test]
    fn style_for_setid() {
        use std::fs::{set_permissions, Permissions};
        use std::os::unix::fs::PermissionsExt;

        let tmp_dir = temp_dir();
        let tmp_file = create_file(tmp_dir.path().join("setid"));
        let perms = Permissions::from_mode(0o6750);
        set_permissions(&tmp_file, perms).unwrap();

        let suid_style = get_default_style(&tmp_file).unwrap();
        assert_eq!(Some(Color::Red), suid_style.background);

        let lscolors = LsColors::from_string("su=0");
        let sgid_style = lscolors.style_for_path(&tmp_file).unwrap();
        assert_eq!(Some(Color::Yellow), sgid_style.background);
    }

    #[cfg(unix)]
    #[test]
    fn style_for_multi_hard_links() {
        let tmp_dir = temp_dir();
        let tmp_file = create_file(tmp_dir.path().join("file1"));
        std::fs::hard_link(&tmp_file, tmp_dir.path().join("file2")).unwrap();

        let lscolors = LsColors::from_string("mh=35");
        let style = lscolors.style_for_path(&tmp_file).unwrap();
        assert_eq!(Some(Color::Magenta), style.foreground);
    }

    #[cfg(unix)]
    #[test]
    fn style_for_sticky_other_writable() {
        use std::fs::{set_permissions, Permissions};
        use std::os::unix::fs::PermissionsExt;

        let tmp_root = temp_dir();
        let tmp_dir = create_dir(tmp_root.path().join("test-dir"));
        let perms = Permissions::from_mode(0o1777);
        set_permissions(&tmp_dir, perms).unwrap();

        let so_style = get_default_style(&tmp_dir).unwrap();
        assert_eq!(Some(Color::Black), so_style.foreground);
        assert_eq!(Some(Color::Green), so_style.background);

        let lscolors1 = LsColors::from_string("tw=0");
        let ow_style = lscolors1.style_for_path(&tmp_dir).unwrap();
        assert_eq!(Some(Color::Blue), ow_style.foreground);
        assert_eq!(Some(Color::Green), ow_style.background);

        let lscolors2 = LsColors::from_string("tw=0:ow=0");
        let st_style = lscolors2.style_for_path(&tmp_dir).unwrap();
        assert_eq!(Some(Color::White), st_style.foreground);
        assert_eq!(Some(Color::Blue), st_style.background);
    }

    #[test]
    fn style_for_path_components() {
        use std::ffi::OsString;

        let tmp_root = temp_dir();
        let tmp_dir = create_dir(tmp_root.path().join("test-dir"));
        create_file(tmp_root.path().join("test-file.png"));

        let tmp_symlink = tmp_root.path().join("test-symlink");
        create_symlink(&tmp_dir, &tmp_symlink);

        let path_via_symlink = tmp_symlink.join("test-file.png");

        let lscolors = LsColors::from_string("di=34:ln=35:*.png=36");

        let mut components: Vec<_> = lscolors
            .style_for_path_components(&path_via_symlink)
            .collect();

        let (c_file, style_file) = components.pop().unwrap();
        assert_eq!("test-file.png", c_file);
        assert_eq!(Some(Color::Cyan), style_file.unwrap().foreground);

        let (c_symlink, style_symlink) = components.pop().unwrap();
        let mut expected_symlink_name = OsString::from("test-symlink");
        expected_symlink_name.push(std::path::MAIN_SEPARATOR.to_string());
        assert_eq!(expected_symlink_name, c_symlink);
        assert_eq!(
            Some(Color::Magenta),
            style_symlink.cloned().and_then(|style| style.foreground)
        );

        let (_, style_dir) = components.pop().unwrap();
        assert_eq!(Some(Color::Blue), style_dir.unwrap().foreground);
    }
}

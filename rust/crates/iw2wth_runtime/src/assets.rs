//! Filesystem/assets adapter boundary for legacy mappack runtime lookups.
//!
//! `game.lua` and `main.lua` build asset paths by string concatenation and then
//! ask LÖVE whether those files exist before loading renderer/audio resources.
//! This module preserves those deterministic path and fallback decisions without
//! moving image, sound, or filesystem APIs into `iw2wth_core`.

use std::{collections::BTreeMap, fs, io, path::PathBuf};

pub const LEGACY_PORTAL_BACKGROUND_PATH: &str = "graphics/SMB/portalbackground.png";

pub trait LegacyAssetSource {
    fn is_file(&self, path: &str) -> bool;
    fn read_bytes(&self, path: &str) -> io::Result<Vec<u8>>;
    fn read_to_string(&self, path: &str) -> io::Result<String>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyAssetPath(String);

impl LegacyAssetPath {
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LegacyAssetPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug)]
pub struct FsLegacyAssetSource {
    root: PathBuf,
}

impl FsLegacyAssetSource {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
}

impl LegacyAssetSource for FsLegacyAssetSource {
    fn is_file(&self, path: &str) -> bool {
        self.resolve(path).is_file()
    }

    fn read_bytes(&self, path: &str) -> io::Result<Vec<u8>> {
        fs::read(self.resolve(path))
    }

    fn read_to_string(&self, path: &str) -> io::Result<String> {
        fs::read_to_string(self.resolve(path))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BufferedLegacyAssetSource {
    files: BTreeMap<String, Vec<u8>>,
}

impl BufferedLegacyAssetSource {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.files.insert(path.into(), Vec::new());
        self
    }

    #[must_use]
    pub fn with_file_contents(
        mut self,
        path: impl Into<String>,
        contents: impl Into<String>,
    ) -> Self {
        self.files.insert(path.into(), contents.into().into_bytes());
        self
    }

    #[must_use]
    pub fn with_file_bytes(mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        self.files.insert(path.into(), bytes.into());
        self
    }
}

impl LegacyAssetSource for BufferedLegacyAssetSource {
    fn is_file(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn read_bytes(&self, path: &str) -> io::Result<Vec<u8>> {
        self.files.get(path).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("legacy asset not found: {path}"),
            )
        })
    }

    fn read_to_string(&self, path: &str) -> io::Result<String> {
        let bytes = self.read_bytes(path)?;
        String::from_utf8(bytes).map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("legacy asset is not valid UTF-8: {path}: {source}"),
            )
        })
    }
}

#[must_use]
pub fn legacy_mappack_settings_path(mappack: &str) -> LegacyAssetPath {
    LegacyAssetPath::new(format!("mappacks/{mappack}/settings.txt"))
}

#[must_use]
pub fn legacy_mappack_level_path(mappack: &str, filename: &str) -> LegacyAssetPath {
    LegacyAssetPath::new(format!("mappacks/{mappack}/{filename}.txt"))
}

#[must_use]
pub fn legacy_mappack_tiles_path(
    source: &impl LegacyAssetSource,
    mappack: &str,
) -> Option<LegacyAssetPath> {
    let path = LegacyAssetPath::new(format!("mappacks/{mappack}/tiles.png"));
    source.is_file(path.as_str()).then_some(path)
}

#[must_use]
pub fn legacy_mappack_music_path(
    source: &impl LegacyAssetSource,
    mappack: &str,
) -> Option<LegacyAssetPath> {
    ["music.ogg", "music.mp3"]
        .into_iter()
        .map(|filename| LegacyAssetPath::new(format!("mappacks/{mappack}/{filename}")))
        .find(|path| source.is_file(path.as_str()))
}

#[must_use]
pub fn legacy_mappack_background_paths(
    source: &impl LegacyAssetSource,
    mappack: &str,
    world: u32,
    level: u32,
    sublevel: u32,
) -> Vec<LegacyAssetPath> {
    let mut level_prefix = format!("{world}-{level}");
    if sublevel != 0 {
        level_prefix = format!("{level_prefix}_{sublevel}");
    }

    let level_backgrounds =
        collect_numbered_backgrounds(source, mappack, &format!("{level_prefix}background"));
    if !level_backgrounds.is_empty() {
        return level_backgrounds;
    }

    let global_backgrounds = collect_numbered_backgrounds(source, mappack, "background");
    if !global_backgrounds.is_empty() {
        return global_backgrounds;
    }

    vec![LegacyAssetPath::new(LEGACY_PORTAL_BACKGROUND_PATH)]
}

fn collect_numbered_backgrounds(
    source: &impl LegacyAssetSource,
    mappack: &str,
    prefix: &str,
) -> Vec<LegacyAssetPath> {
    let mut paths = Vec::new();
    let mut index = 1;

    loop {
        let path = LegacyAssetPath::new(format!("mappacks/{mappack}/{prefix}{index}.png"));
        if !source.is_file(path.as_str()) {
            break;
        }

        paths.push(path);
        index += 1;
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::{
        BufferedLegacyAssetSource, LEGACY_PORTAL_BACKGROUND_PATH, legacy_mappack_background_paths,
        legacy_mappack_level_path, legacy_mappack_music_path, legacy_mappack_settings_path,
        legacy_mappack_tiles_path,
    };

    fn path_strings(paths: Vec<super::LegacyAssetPath>) -> Vec<String> {
        paths
            .into_iter()
            .map(|path| path.as_str().to_owned())
            .collect()
    }

    #[test]
    fn mappack_settings_and_level_paths_preserve_lua_concatenation() {
        assert_eq!(
            legacy_mappack_settings_path("smb").as_str(),
            "mappacks/smb/settings.txt",
        );
        assert_eq!(
            legacy_mappack_level_path("portal", "2-1_3").as_str(),
            "mappacks/portal/2-1_3.txt",
        );
    }

    #[test]
    fn custom_tiles_path_is_available_only_when_the_mappack_file_exists() {
        let source = BufferedLegacyAssetSource::new().with_file("mappacks/smb/tiles.png");

        assert_eq!(
            legacy_mappack_tiles_path(&source, "smb")
                .expect("tiles should exist")
                .as_str(),
            "mappacks/smb/tiles.png",
        );
        assert_eq!(legacy_mappack_tiles_path(&source, "portal"), None);
    }

    #[test]
    fn custom_music_prefers_ogg_before_mp3_like_game_load() {
        let source = BufferedLegacyAssetSource::new()
            .with_file("mappacks/smb/music.mp3")
            .with_file("mappacks/smb/music.ogg");

        assert_eq!(
            legacy_mappack_music_path(&source, "smb")
                .expect("music should exist")
                .as_str(),
            "mappacks/smb/music.ogg",
        );

        let source = BufferedLegacyAssetSource::new().with_file("mappacks/smb/music.mp3");
        assert_eq!(
            legacy_mappack_music_path(&source, "smb")
                .expect("mp3 should be fallback")
                .as_str(),
            "mappacks/smb/music.mp3",
        );
    }

    #[test]
    fn level_specific_backgrounds_win_and_stop_at_the_first_missing_index() {
        let source = BufferedLegacyAssetSource::new()
            .with_file("mappacks/portal/2-3_1background1.png")
            .with_file("mappacks/portal/2-3_1background2.png")
            .with_file("mappacks/portal/2-3_1background4.png")
            .with_file("mappacks/portal/background1.png");

        assert_eq!(
            path_strings(legacy_mappack_background_paths(&source, "portal", 2, 3, 1)),
            vec![
                "mappacks/portal/2-3_1background1.png",
                "mappacks/portal/2-3_1background2.png",
            ],
        );
    }

    #[test]
    fn global_backgrounds_fallback_before_the_default_portal_background() {
        let source = BufferedLegacyAssetSource::new()
            .with_file("mappacks/portal/background1.png")
            .with_file("mappacks/portal/background2.png");

        assert_eq!(
            path_strings(legacy_mappack_background_paths(&source, "portal", 2, 3, 0)),
            vec![
                "mappacks/portal/background1.png",
                "mappacks/portal/background2.png",
            ],
        );
    }

    #[test]
    fn missing_custom_backgrounds_use_the_smb_portal_background_fallback() {
        let source = BufferedLegacyAssetSource::new();

        assert_eq!(
            path_strings(legacy_mappack_background_paths(&source, "smb", 1, 1, 0)),
            vec![LEGACY_PORTAL_BACKGROUND_PATH],
        );
    }
}

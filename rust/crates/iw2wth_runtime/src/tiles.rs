//! Adapter-side legacy tile metadata decoded from Mari0 tile atlases.
//!
//! `quad.lua` derives collision, invisibility, breakability, coin-block, coin,
//! and portalability flags from six alpha pixels beside each 16x16 tile. This
//! module preserves that metadata extraction without moving image or atlas
//! policy into `iw2wth_core`.

use std::{error::Error, fmt, io};

use image::{DynamicImage, ImageError, RgbaImage};
use iw2wth_core::TileId;

use crate::{
    assets::{LegacyAssetPath, LegacyAssetSource, legacy_mappack_tiles_path},
    render::{LEGACY_TILE_ATLAS_STRIDE_PX, LegacyTileAtlas, legacy_tile_quad_count},
};

const LEGACY_TILE_METADATA_COLLISION_ROW: u32 = 0;
const LEGACY_TILE_METADATA_INVISIBLE_ROW: u32 = 1;
const LEGACY_TILE_METADATA_BREAKABLE_ROW: u32 = 2;
const LEGACY_TILE_METADATA_COIN_BLOCK_ROW: u32 = 3;
const LEGACY_TILE_METADATA_COIN_ROW: u32 = 4;
const LEGACY_TILE_METADATA_NOT_PORTALABLE_ROW: u32 = 5;
const LEGACY_ALPHA_HALF_THRESHOLD: u8 = 127;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyTileMetadata {
    pub collision: bool,
    pub invisible: bool,
    pub breakable: bool,
    pub coin_block: bool,
    pub coin: bool,
    pub portalable: bool,
}

impl LegacyTileMetadata {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            collision: false,
            invisible: false,
            breakable: false,
            coin_block: false,
            coin: false,
            portalable: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyTileMetadataTable {
    tiles: Vec<LegacyTileMetadata>,
    smb_tile_count: u32,
    portal_tile_count: u32,
    custom_tile_count: u32,
}

impl LegacyTileMetadataTable {
    pub fn load(
        source: &impl LegacyAssetSource,
        graphicspack: &str,
        mappack: &str,
    ) -> Result<Self, LegacyTileMetadataLoadError> {
        let smb_path = LegacyTileAtlas::Smb.image_path(graphicspack, mappack);
        let portal_path = LegacyTileAtlas::Portal.image_path(graphicspack, mappack);
        let mut smb_tiles = load_legacy_tile_metadata_atlas(source, smb_path)?;
        let mut portal_tiles = load_legacy_tile_metadata_atlas(source, portal_path)?;
        let mut custom_tiles = match legacy_mappack_tiles_path(source, mappack) {
            Some(path) => load_legacy_tile_metadata_atlas(source, path)?,
            None => Vec::new(),
        };

        let smb_tile_count = len_to_u32_saturating(smb_tiles.len());
        let portal_tile_count = len_to_u32_saturating(portal_tiles.len());
        let custom_tile_count = len_to_u32_saturating(custom_tiles.len());
        let mut tiles =
            Vec::with_capacity(smb_tiles.len() + portal_tiles.len() + custom_tiles.len());
        tiles.append(&mut smb_tiles);
        tiles.append(&mut portal_tiles);
        tiles.append(&mut custom_tiles);

        Ok(Self {
            tiles,
            smb_tile_count,
            portal_tile_count,
            custom_tile_count,
        })
    }

    #[must_use]
    pub const fn smb_tile_count(&self) -> u32 {
        self.smb_tile_count
    }

    #[must_use]
    pub const fn portal_tile_count(&self) -> u32 {
        self.portal_tile_count
    }

    #[must_use]
    pub const fn custom_tile_count(&self) -> u32 {
        self.custom_tile_count
    }

    #[must_use]
    pub fn metadata_for_tile(&self, tile_id: TileId) -> Option<LegacyTileMetadata> {
        let index = usize::from(tile_id.0.checked_sub(1)?);
        self.tiles.get(index).copied()
    }

    #[must_use]
    pub fn tile_collides(&self, tile_id: TileId) -> bool {
        self.metadata_for_tile(tile_id)
            .is_some_and(|metadata| metadata.collision)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_metadata_for_tests(tiles: Vec<LegacyTileMetadata>) -> Self {
        let tile_count = len_to_u32_saturating(tiles.len());
        Self {
            tiles,
            smb_tile_count: tile_count,
            portal_tile_count: 0,
            custom_tile_count: 0,
        }
    }
}

pub fn load_legacy_tile_metadata_atlas(
    source: &impl LegacyAssetSource,
    path: LegacyAssetPath,
) -> Result<Vec<LegacyTileMetadata>, LegacyTileMetadataLoadError> {
    let bytes = source.read_bytes(path.as_str()).map_err(|source| {
        LegacyTileMetadataLoadError::ReadAtlas {
            path: path.clone(),
            source,
        }
    })?;
    let image = image::load_from_memory(&bytes)
        .map(DynamicImage::into_rgba8)
        .map_err(|source| LegacyTileMetadataLoadError::DecodeAtlas {
            path: path.clone(),
            source,
        })?;

    Ok(extract_legacy_tile_metadata(&image))
}

#[must_use]
pub fn extract_legacy_tile_metadata(image: &RgbaImage) -> Vec<LegacyTileMetadata> {
    let width = image.width();
    let height = image.height();
    let tile_count = legacy_tile_quad_count(width, height);
    let columns = width / LEGACY_TILE_ATLAS_STRIDE_PX;
    let rows = height / LEGACY_TILE_ATLAS_STRIDE_PX;
    let mut metadata = Vec::with_capacity(usize::try_from(tile_count).unwrap_or(0));

    for row in 0..rows {
        for column in 0..columns {
            let metadata_x = column * LEGACY_TILE_ATLAS_STRIDE_PX + 16;
            let metadata_y = row * LEGACY_TILE_ATLAS_STRIDE_PX;
            metadata.push(LegacyTileMetadata {
                collision: alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_COLLISION_ROW,
                ),
                invisible: alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_INVISIBLE_ROW,
                ),
                breakable: alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_BREAKABLE_ROW,
                ),
                coin_block: alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_COIN_BLOCK_ROW,
                ),
                coin: alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_COIN_ROW,
                ),
                portalable: !alpha_is_set(
                    image,
                    metadata_x,
                    metadata_y + LEGACY_TILE_METADATA_NOT_PORTALABLE_ROW,
                ),
            });
        }
    }

    metadata
}

fn alpha_is_set(image: &RgbaImage, x: u32, y: u32) -> bool {
    image.get_pixel(x, y).0[3] > LEGACY_ALPHA_HALF_THRESHOLD
}

fn len_to_u32_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[derive(Debug)]
pub enum LegacyTileMetadataLoadError {
    ReadAtlas {
        path: LegacyAssetPath,
        source: io::Error,
    },
    DecodeAtlas {
        path: LegacyAssetPath,
        source: ImageError,
    },
}

impl fmt::Display for LegacyTileMetadataLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadAtlas { path, .. } => {
                write!(
                    formatter,
                    "failed to read legacy tile atlas at {}",
                    path.as_str()
                )
            }
            Self::DecodeAtlas { path, .. } => {
                write!(
                    formatter,
                    "failed to decode legacy tile atlas at {}",
                    path.as_str()
                )
            }
        }
    }
}

impl Error for LegacyTileMetadataLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadAtlas { source, .. } => Some(source),
            Self::DecodeAtlas { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
    use iw2wth_core::TileId;

    use super::{
        LegacyTileMetadata, LegacyTileMetadataTable, extract_legacy_tile_metadata,
        load_legacy_tile_metadata_atlas,
    };
    use crate::assets::{BufferedLegacyAssetSource, LegacyAssetPath};

    fn atlas_png(tile_flags: &[LegacyTileMetadata]) -> Vec<u8> {
        let mut image = RgbaImage::new((tile_flags.len() as u32) * 17, 17);
        for (index, metadata) in tile_flags.iter().enumerate() {
            let metadata_x = index as u32 * 17 + 16;
            set_flag(&mut image, metadata_x, 0, metadata.collision);
            set_flag(&mut image, metadata_x, 1, metadata.invisible);
            set_flag(&mut image, metadata_x, 2, metadata.breakable);
            set_flag(&mut image, metadata_x, 3, metadata.coin_block);
            set_flag(&mut image, metadata_x, 4, metadata.coin);
            set_flag(&mut image, metadata_x, 5, !metadata.portalable);
        }

        let mut bytes = Cursor::new(Vec::new());
        match ImageBuffer::write_to(&image, &mut bytes, ImageFormat::Png) {
            Ok(()) => bytes.into_inner(),
            Err(error) => panic!("failed to encode test PNG: {error}"),
        }
    }

    fn set_flag(image: &mut RgbaImage, x: u32, y: u32, enabled: bool) {
        if enabled {
            image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }

    #[test]
    fn tile_metadata_extracts_the_same_alpha_flags_as_quad_lua() {
        let flags = [
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                invisible: true,
                breakable: true,
                coin_block: true,
                coin: true,
                portalable: false,
            },
        ];
        let bytes = atlas_png(&flags);
        let image = match image::load_from_memory(&bytes) {
            Ok(image) => image.into_rgba8(),
            Err(error) => panic!("failed to decode test PNG: {error}"),
        };

        assert_eq!(extract_legacy_tile_metadata(&image), flags);
    }

    #[test]
    fn tile_metadata_table_keeps_smb_portal_custom_tile_id_order() {
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png(&[LegacyTileMetadata {
                    invisible: true,
                    ..LegacyTileMetadata::empty()
                }]),
            )
            .with_file_bytes(
                "mappacks/test/tiles.png",
                atlas_png(&[LegacyTileMetadata {
                    collision: true,
                    coin: true,
                    ..LegacyTileMetadata::empty()
                }]),
            );

        let table = match LegacyTileMetadataTable::load(&source, "SMB", "test") {
            Ok(table) => table,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(table.smb_tile_count(), 2);
        assert_eq!(table.portal_tile_count(), 1);
        assert_eq!(table.custom_tile_count(), 1);
        assert!(!table.tile_collides(TileId(1)));
        assert!(table.tile_collides(TileId(2)));
        assert_eq!(
            table.metadata_for_tile(TileId(3)),
            Some(LegacyTileMetadata {
                invisible: true,
                ..LegacyTileMetadata::empty()
            }),
        );
        assert_eq!(
            table.metadata_for_tile(TileId(4)),
            Some(LegacyTileMetadata {
                collision: true,
                coin: true,
                ..LegacyTileMetadata::empty()
            }),
        );
        assert_eq!(table.metadata_for_tile(TileId(5)), None);
    }

    #[test]
    fn tile_atlas_loading_decodes_png_bytes_from_asset_source() {
        let source = BufferedLegacyAssetSource::new().with_file_bytes(
            "atlas.png",
            atlas_png(&[LegacyTileMetadata {
                collision: true,
                breakable: true,
                ..LegacyTileMetadata::empty()
            }]),
        );

        let metadata =
            match load_legacy_tile_metadata_atlas(&source, LegacyAssetPath::new("atlas.png")) {
                Ok(metadata) => metadata,
                Err(error) => panic!("{error}"),
            };

        assert_eq!(
            metadata,
            vec![LegacyTileMetadata {
                collision: true,
                breakable: true,
                ..LegacyTileMetadata::empty()
            }],
        );
    }
}

//! Rendering intent adapter boundary for the legacy Mari0 draw path.
//!
//! Lua code loads image assets through LÖVE, keeps nearest-neighbor filtering
//! as the global default, draws tile sprite batches in SMB/Portal/custom order,
//! and tiles custom backgrounds back-to-front with simple parallax math. This
//! module captures those deterministic renderer-facing decisions without
//! exposing renderer objects to `iw2wth_core`.

use crate::assets::LegacyAssetPath;

pub const LEGACY_TILE_SIZE_PX: f32 = 16.0;
pub const LEGACY_TILE_ATLAS_STRIDE_PX: u32 = 17;
pub const LEGACY_VIEW_HEIGHT_TILES: f32 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl LegacyColor {
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[must_use]
pub fn legacy_background_color(background: u8) -> Option<LegacyColor> {
    match background {
        1 => Some(LegacyColor::rgb(92.0 / 255.0, 148.0 / 255.0, 252.0 / 255.0)),
        2 => Some(LegacyColor::rgb(0.0, 0.0, 0.0)),
        3 => Some(LegacyColor::rgb(32.0 / 255.0, 56.0 / 255.0, 236.0 / 255.0)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTextureFilter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTextureWrap {
    Clamp,
    Repeat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRenderImageSpec {
    pub path: LegacyAssetPath,
    pub min_filter: LegacyTextureFilter,
    pub mag_filter: LegacyTextureFilter,
    pub wrap_x: LegacyTextureWrap,
    pub wrap_y: LegacyTextureWrap,
}

impl LegacyRenderImageSpec {
    #[must_use]
    pub fn nearest_clamped(path: LegacyAssetPath) -> Self {
        Self {
            path,
            min_filter: LegacyTextureFilter::Nearest,
            mag_filter: LegacyTextureFilter::Nearest,
            wrap_x: LegacyTextureWrap::Clamp,
            wrap_y: LegacyTextureWrap::Clamp,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyTileAtlas {
    Smb,
    Portal,
    Custom,
}

impl LegacyTileAtlas {
    #[must_use]
    pub fn image_path(self, graphicspack: &str, mappack: &str) -> LegacyAssetPath {
        match self {
            Self::Smb => LegacyAssetPath::new(format!("graphics/{graphicspack}/smbtiles.png")),
            Self::Portal => {
                LegacyAssetPath::new(format!("graphics/{graphicspack}/portaltiles.png"))
            }
            Self::Custom => LegacyAssetPath::new(format!("mappacks/{mappack}/tiles.png")),
        }
    }

    #[must_use]
    pub fn image_spec(self, graphicspack: &str, mappack: &str) -> LegacyRenderImageSpec {
        LegacyRenderImageSpec::nearest_clamped(self.image_path(graphicspack, mappack))
    }
}

#[must_use]
pub fn legacy_tile_quad_count(image_width_px: u32, image_height_px: u32) -> u32 {
    (image_width_px / LEGACY_TILE_ATLAS_STRIDE_PX) * (image_height_px / LEGACY_TILE_ATLAS_STRIDE_PX)
}

#[must_use]
pub fn legacy_tile_atlas_for_tile(
    tile_number: u32,
    smb_tile_count: u32,
    portal_tile_count: u32,
    custom_tile_count: u32,
) -> Option<LegacyTileAtlas> {
    if tile_number == 0 {
        return None;
    }

    if tile_number <= smb_tile_count {
        Some(LegacyTileAtlas::Smb)
    } else if tile_number <= smb_tile_count + portal_tile_count {
        Some(LegacyTileAtlas::Portal)
    } else if tile_number <= smb_tile_count + portal_tile_count + custom_tile_count {
        Some(LegacyTileAtlas::Custom)
    } else {
        None
    }
}

#[must_use]
pub fn legacy_tile_batch_draw_order(custom_tiles: bool) -> Vec<LegacyTileAtlas> {
    let mut order = vec![LegacyTileAtlas::Smb, LegacyTileAtlas::Portal];
    if custom_tiles {
        order.push(LegacyTileAtlas::Custom);
    }
    order
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyTileBatchDrawIntent {
    pub atlas: LegacyTileAtlas,
    pub x_px: f32,
    pub y_px: f32,
}

#[must_use]
pub fn legacy_tile_batch_draw_intents(
    xscroll: f32,
    scale: f32,
    custom_tiles: bool,
) -> Vec<LegacyTileBatchDrawIntent> {
    let x_px = (-(xscroll % 1.0) * LEGACY_TILE_SIZE_PX * scale).floor();

    legacy_tile_batch_draw_order(custom_tiles)
        .into_iter()
        .map(|atlas| LegacyTileBatchDrawIntent {
            atlas,
            x_px,
            y_px: 0.0,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyBackgroundLayer {
    pub path: LegacyAssetPath,
    pub width_tiles: f32,
    pub height_tiles: f32,
}

impl LegacyBackgroundLayer {
    #[must_use]
    pub fn from_image_pixels(path: LegacyAssetPath, width_px: u32, height_px: u32) -> Self {
        Self {
            path,
            width_tiles: width_px as f32 / LEGACY_TILE_SIZE_PX,
            height_tiles: height_px as f32 / LEGACY_TILE_SIZE_PX,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyImageDrawIntent {
    pub path: LegacyAssetPath,
    pub x_px: f32,
    pub y_px: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

#[must_use]
pub fn legacy_custom_background_draw_intents(
    layers: &[LegacyBackgroundLayer],
    screen_width_tiles: f32,
    xscroll: f32,
    scale: f32,
    scroll_factor: f32,
    reverse_scroll_factor: bool,
) -> Vec<LegacyImageDrawIntent> {
    let mut intents = Vec::new();

    for layer_index in (0..layers.len()).rev() {
        let layer = &layers[layer_index];
        let lua_index = layer_index as f32 + 1.0;
        let mut layer_scroll = xscroll / (lua_index * scroll_factor + 1.0);
        if reverse_scroll_factor {
            layer_scroll = 0.0;
        }

        let y_count = (LEGACY_VIEW_HEIGHT_TILES / layer.height_tiles).ceil() as u32;
        let x_count = (screen_width_tiles / layer.width_tiles).ceil() as u32 + 1;
        let scroll_offset_px =
            ((layer_scroll % layer.width_tiles) * LEGACY_TILE_SIZE_PX * scale).floor();

        for y in 1..=y_count {
            for x in 1..=x_count {
                intents.push(LegacyImageDrawIntent {
                    path: layer.path.clone(),
                    x_px: (((x - 1) as f32 * layer.width_tiles) * LEGACY_TILE_SIZE_PX * scale)
                        .floor()
                        - scroll_offset_px,
                    y_px: (y - 1) as f32 * layer.height_tiles * LEGACY_TILE_SIZE_PX * scale,
                    rotation: 0.0,
                    scale_x: scale,
                    scale_y: scale,
                });
            }
        }
    }

    intents
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyBackgroundLayer, LegacyColor, LegacyRenderImageSpec, LegacyTextureFilter,
        LegacyTextureWrap, LegacyTileAtlas, LegacyTileBatchDrawIntent, legacy_background_color,
        legacy_custom_background_draw_intents, legacy_tile_atlas_for_tile,
        legacy_tile_batch_draw_intents, legacy_tile_batch_draw_order, legacy_tile_quad_count,
    };
    use crate::assets::LegacyAssetPath;

    #[test]
    fn background_colors_match_legacy_lua_tables() {
        assert_eq!(
            legacy_background_color(1),
            Some(LegacyColor::rgb(92.0 / 255.0, 148.0 / 255.0, 252.0 / 255.0)),
        );
        assert_eq!(
            legacy_background_color(2),
            Some(LegacyColor::rgb(0.0, 0.0, 0.0))
        );
        assert_eq!(
            legacy_background_color(3),
            Some(LegacyColor::rgb(32.0 / 255.0, 56.0 / 255.0, 236.0 / 255.0)),
        );
        assert_eq!(legacy_background_color(4), None);
    }

    #[test]
    fn tile_atlas_image_specs_preserve_legacy_paths_and_nearest_filtering() {
        assert_eq!(
            LegacyTileAtlas::Smb.image_spec("SMB", "portal"),
            LegacyRenderImageSpec {
                path: LegacyAssetPath::new("graphics/SMB/smbtiles.png"),
                min_filter: LegacyTextureFilter::Nearest,
                mag_filter: LegacyTextureFilter::Nearest,
                wrap_x: LegacyTextureWrap::Clamp,
                wrap_y: LegacyTextureWrap::Clamp,
            },
        );
        assert_eq!(
            LegacyTileAtlas::Portal
                .image_path("ALLSTARS", "portal")
                .as_str(),
            "graphics/ALLSTARS/portaltiles.png",
        );
        assert_eq!(
            LegacyTileAtlas::Custom
                .image_path("SMB", "custompack")
                .as_str(),
            "mappacks/custompack/tiles.png",
        );
    }

    #[test]
    fn tile_quad_count_uses_seventeen_pixel_legacy_atlas_stride() {
        assert_eq!(legacy_tile_quad_count(170, 34), 20);
        assert_eq!(legacy_tile_quad_count(16, 16), 0);
    }

    #[test]
    fn tile_atlas_lookup_preserves_lua_threshold_order() {
        assert_eq!(legacy_tile_atlas_for_tile(0, 3, 2, 1), None);
        assert_eq!(
            legacy_tile_atlas_for_tile(3, 3, 2, 1),
            Some(LegacyTileAtlas::Smb)
        );
        assert_eq!(
            legacy_tile_atlas_for_tile(4, 3, 2, 1),
            Some(LegacyTileAtlas::Portal),
        );
        assert_eq!(
            legacy_tile_atlas_for_tile(6, 3, 2, 1),
            Some(LegacyTileAtlas::Custom),
        );
        assert_eq!(legacy_tile_atlas_for_tile(7, 3, 2, 1), None);
    }

    #[test]
    fn tile_batch_draw_order_matches_game_draw() {
        assert_eq!(
            legacy_tile_batch_draw_order(false),
            vec![LegacyTileAtlas::Smb, LegacyTileAtlas::Portal],
        );
        assert_eq!(
            legacy_tile_batch_draw_order(true),
            vec![
                LegacyTileAtlas::Smb,
                LegacyTileAtlas::Portal,
                LegacyTileAtlas::Custom
            ],
        );
    }

    #[test]
    fn tile_batch_draw_intents_share_the_legacy_fractional_scroll_offset() {
        assert_eq!(
            legacy_tile_batch_draw_intents(3.25, 2.0, true),
            vec![
                LegacyTileBatchDrawIntent {
                    atlas: LegacyTileAtlas::Smb,
                    x_px: -8.0,
                    y_px: 0.0,
                },
                LegacyTileBatchDrawIntent {
                    atlas: LegacyTileAtlas::Portal,
                    x_px: -8.0,
                    y_px: 0.0,
                },
                LegacyTileBatchDrawIntent {
                    atlas: LegacyTileAtlas::Custom,
                    x_px: -8.0,
                    y_px: 0.0,
                },
            ],
        );
    }

    #[test]
    fn custom_backgrounds_draw_back_to_front_with_parallax_tiling() {
        let layers = vec![
            LegacyBackgroundLayer::from_image_pixels(LegacyAssetPath::new("front.png"), 32, 240),
            LegacyBackgroundLayer::from_image_pixels(LegacyAssetPath::new("back.png"), 64, 240),
        ];

        let intents = legacy_custom_background_draw_intents(&layers, 5.0, 3.0, 2.0, 1.0, false);

        assert_eq!(intents.len(), 7);
        assert_eq!(intents[0].path.as_str(), "back.png");
        assert_eq!(intents[0].x_px, -32.0);
        assert_eq!(intents[1].x_px, 96.0);
        assert_eq!(intents[2].x_px, 224.0);
        assert_eq!(intents[3].path.as_str(), "front.png");
        assert_eq!(intents[3].x_px, -48.0);
        assert_eq!(intents[4].x_px, 16.0);
        assert_eq!(intents[5].x_px, 80.0);
        assert_eq!(intents[6].x_px, 144.0);
    }

    #[test]
    fn reverse_scroll_factor_pins_custom_background_scroll_to_zero() {
        let layers = vec![LegacyBackgroundLayer::from_image_pixels(
            LegacyAssetPath::new("background.png"),
            80,
            240,
        )];

        let intents = legacy_custom_background_draw_intents(&layers, 5.0, 99.0, 1.0, 1.0, true);

        assert_eq!(intents.len(), 2);
        assert_eq!(intents[0].x_px, 0.0);
        assert_eq!(intents[1].x_px, 80.0);
    }
}

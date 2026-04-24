//! Map-query adapter boundary for parsed Mari0 level data.
//!
//! Legacy gameplay code often asks the Lua `map` table questions using
//! 1-based tile coordinates. This module keeps that lookup convention at the
//! runtime edge while exposing typed data that core callbacks can consume.

use iw2wth_core::{
    CellValue, LegacyEntityKind, LegacyEntitySurface, LegacyGelKind, LegacyMapBounds,
    LegacyMapTileCoord, LevelCell, Mari0Level, TileId, content::MARI0_LEVEL_HEIGHT,
};

use crate::tiles::{LegacyTileMetadata, LegacyTileMetadataTable};

pub trait LegacyMapQuery {
    fn bounds(&self) -> LegacyMapBounds;
    fn tile_id_at(&self, coord: LegacyMapTileCoord) -> Option<TileId>;
    fn top_gel_at(&self, coord: LegacyMapTileCoord) -> Option<LegacyGelKind>;

    #[must_use]
    fn contains(&self, coord: LegacyMapTileCoord) -> bool {
        self.bounds().contains(coord)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyMapTileMetadata {
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub metadata: LegacyTileMetadata,
}

impl LegacyMapTileMetadata {
    #[must_use]
    pub const fn collides(self) -> bool {
        self.metadata.collision
    }

    #[must_use]
    pub const fn invisible(self) -> bool {
        self.metadata.invisible
    }

    #[must_use]
    pub const fn breakable(self) -> bool {
        self.metadata.breakable
    }

    #[must_use]
    pub const fn coin_block(self) -> bool {
        self.metadata.coin_block
    }

    #[must_use]
    pub const fn coin(self) -> bool {
        self.metadata.coin
    }

    #[must_use]
    pub const fn portalable(self) -> bool {
        self.metadata.portalable
    }

    #[must_use]
    pub const fn solid_portalable(self) -> bool {
        self.metadata.collision && self.metadata.portalable
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LegacyTileMetadataMapQuery<'level, 'tiles> {
    level: LegacyLevelMapQuery<'level>,
    tiles: &'tiles LegacyTileMetadataTable,
}

impl<'level, 'tiles> LegacyTileMetadataMapQuery<'level, 'tiles> {
    #[must_use]
    pub const fn new(level: &'level Mari0Level, tiles: &'tiles LegacyTileMetadataTable) -> Self {
        Self {
            level: LegacyLevelMapQuery::new(level),
            tiles,
        }
    }

    #[must_use]
    pub fn tile_metadata_at(self, coord: LegacyMapTileCoord) -> Option<LegacyMapTileMetadata> {
        let tile_id = self.level.tile_id_at(coord)?;
        let metadata = self.tiles.metadata_for_tile(tile_id)?;

        Some(LegacyMapTileMetadata {
            coord,
            tile_id,
            metadata,
        })
    }

    #[must_use]
    pub fn tile_collides_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::collides)
    }

    #[must_use]
    pub fn tile_is_invisible_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::invisible)
    }

    #[must_use]
    pub fn tile_is_breakable_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::breakable)
    }

    #[must_use]
    pub fn tile_is_coin_block_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::coin_block)
    }

    #[must_use]
    pub fn tile_is_coin_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::coin)
    }

    #[must_use]
    pub fn tile_is_portalable_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::portalable)
    }

    #[must_use]
    pub fn tile_is_solid_portalable_at(self, coord: LegacyMapTileCoord) -> bool {
        self.tile_metadata_at(coord)
            .is_some_and(LegacyMapTileMetadata::solid_portalable)
    }

    #[must_use]
    pub fn legacy_entity_kind_at(self, coord: LegacyMapTileCoord) -> Option<LegacyEntityKind> {
        self.level
            .cell_at(coord)?
            .legacy_entity()
            .map(|entity| entity.kind)
    }
}

impl LegacyMapQuery for LegacyTileMetadataMapQuery<'_, '_> {
    fn bounds(&self) -> LegacyMapBounds {
        self.level.bounds()
    }

    fn tile_id_at(&self, coord: LegacyMapTileCoord) -> Option<TileId> {
        self.level.tile_id_at(coord)
    }

    fn top_gel_at(&self, coord: LegacyMapTileCoord) -> Option<LegacyGelKind> {
        self.level.top_gel_at(coord)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LegacyLevelMapQuery<'level> {
    level: &'level Mari0Level,
}

impl<'level> LegacyLevelMapQuery<'level> {
    #[must_use]
    pub const fn new(level: &'level Mari0Level) -> Self {
        Self { level }
    }

    #[must_use]
    fn cell_at(self, coord: LegacyMapTileCoord) -> Option<&'level LevelCell> {
        if coord.x < 1 || coord.y < 1 {
            return None;
        }

        let x = usize::try_from(coord.x - 1).ok()?;
        let y = usize::try_from(coord.y - 1).ok()?;
        self.level.cell(x, y)
    }
}

impl LegacyMapQuery for LegacyLevelMapQuery<'_> {
    fn bounds(&self) -> LegacyMapBounds {
        LegacyMapBounds::new(
            usize_to_i32_saturating(self.level.width()),
            usize_to_i32_saturating(MARI0_LEVEL_HEIGHT),
        )
    }

    fn tile_id_at(&self, coord: LegacyMapTileCoord) -> Option<TileId> {
        self.cell_at(coord).and_then(LevelCell::tile_id).map(TileId)
    }

    fn top_gel_at(&self, coord: LegacyMapTileCoord) -> Option<LegacyGelKind> {
        let cell = self.cell_at(coord)?;
        let entity = cell.legacy_entity()?;

        if entity.kind != LegacyEntityKind::GelTile(LegacyEntitySurface::Top) {
            return None;
        }

        entity.parameters.first().and_then(gel_kind_from_cell_value)
    }
}

fn usize_to_i32_saturating(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn gel_kind_from_cell_value(value: &CellValue) -> Option<LegacyGelKind> {
    match value.as_u16()? {
        1 => Some(LegacyGelKind::Blue),
        2 => Some(LegacyGelKind::Orange),
        3 => Some(LegacyGelKind::White),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{LegacyLevelMapQuery, LegacyMapQuery, LegacyTileMetadataMapQuery};
    use crate::tiles::{LegacyTileMetadata, LegacyTileMetadataTable};
    use iw2wth_core::{
        LegacyGelKind, LegacyMapTileCoord, Mari0Level, TileId, content::MARI0_LEVEL_HEIGHT,
    };

    fn parse_level_with_cells(width: usize, overrides: &[(usize, &str)]) -> Mari0Level {
        let mut cells = vec!["1"; width * MARI0_LEVEL_HEIGHT];
        for (index, value) in overrides {
            cells[*index] = value;
        }

        Mari0Level::parse(&cells.join(",")).expect("test level should parse")
    }

    #[test]
    fn level_query_uses_lua_one_based_coordinates() {
        let level = parse_level_with_cells(2, &[(0, "7"), (1, "8"), (2, "9")]);
        let query = LegacyLevelMapQuery::new(&level);

        assert_eq!(query.bounds().width, 2);
        assert_eq!(query.bounds().height, 15);
        assert!(query.contains(LegacyMapTileCoord::new(1, 1)));
        assert_eq!(
            query.tile_id_at(LegacyMapTileCoord::new(1, 1)),
            Some(TileId(7)),
        );
        assert_eq!(
            query.tile_id_at(LegacyMapTileCoord::new(2, 1)),
            Some(TileId(8)),
        );
        assert_eq!(
            query.tile_id_at(LegacyMapTileCoord::new(1, 2)),
            Some(TileId(9)),
        );
        assert_eq!(query.tile_id_at(LegacyMapTileCoord::new(0, 1)), None);
        assert_eq!(query.tile_id_at(LegacyMapTileCoord::new(3, 1)), None);
    }

    #[test]
    fn top_gel_query_preserves_legacy_surface_and_gel_id_payloads() {
        let level = parse_level_with_cells(
            2,
            &[
                (0, "1-85-2"),
                (1, "1-85-1"),
                (2, "1-85-3"),
                (3, "1-88-2"),
                (4, "1-85-9"),
            ],
        );
        let query = LegacyLevelMapQuery::new(&level);

        assert_eq!(
            query.top_gel_at(LegacyMapTileCoord::new(1, 1)),
            Some(LegacyGelKind::Orange),
        );
        assert_eq!(
            query.top_gel_at(LegacyMapTileCoord::new(2, 1)),
            Some(LegacyGelKind::Blue),
        );
        assert_eq!(
            query.top_gel_at(LegacyMapTileCoord::new(1, 2)),
            Some(LegacyGelKind::White),
        );
        assert_eq!(query.top_gel_at(LegacyMapTileCoord::new(2, 2)), None);
        assert_eq!(query.top_gel_at(LegacyMapTileCoord::new(1, 3)), None);
    }

    #[test]
    fn tile_metadata_query_joins_level_tile_ids_to_quad_lua_flags() {
        let level = parse_level_with_cells(3, &[(0, "1"), (1, "2"), (2, "3")]);
        let tiles = LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                invisible: true,
                breakable: true,
                coin_block: true,
                coin: false,
                portalable: false,
            },
            LegacyTileMetadata {
                collision: false,
                invisible: false,
                breakable: false,
                coin_block: false,
                coin: true,
                portalable: true,
            },
        ]);
        let query = LegacyTileMetadataMapQuery::new(&level, &tiles);

        let solid = query
            .tile_metadata_at(LegacyMapTileCoord::new(2, 1))
            .expect("tile 2 metadata should exist");
        assert_eq!(solid.coord, LegacyMapTileCoord::new(2, 1));
        assert_eq!(solid.tile_id, TileId(2));
        assert!(solid.collides());
        assert!(solid.invisible());
        assert!(solid.breakable());
        assert!(solid.coin_block());
        assert!(!solid.coin());
        assert!(!solid.portalable());
        assert!(!solid.solid_portalable());

        assert!(query.tile_is_coin_at(LegacyMapTileCoord::new(3, 1)));
        assert!(query.tile_is_portalable_at(LegacyMapTileCoord::new(3, 1)));
        assert!(!query.tile_collides_at(LegacyMapTileCoord::new(3, 1)));
        assert!(!query.tile_is_coin_at(LegacyMapTileCoord::new(4, 1)));
        assert_eq!(query.tile_metadata_at(LegacyMapTileCoord::new(4, 1)), None);
    }

    #[test]
    fn tile_metadata_query_keeps_portalability_tied_to_solid_tiles_for_get_tile_style_checks() {
        let level = parse_level_with_cells(3, &[(0, "1"), (1, "2"), (2, "3")]);
        let tiles = LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata {
                collision: false,
                portalable: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                portalable: false,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                portalable: true,
                ..LegacyTileMetadata::empty()
            },
        ]);
        let query = LegacyTileMetadataMapQuery::new(&level, &tiles);

        assert!(!query.tile_is_solid_portalable_at(LegacyMapTileCoord::new(1, 1)));
        assert!(!query.tile_is_solid_portalable_at(LegacyMapTileCoord::new(2, 1)));
        assert!(query.tile_is_solid_portalable_at(LegacyMapTileCoord::new(3, 1)));
    }
}

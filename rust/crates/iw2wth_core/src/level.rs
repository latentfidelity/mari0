#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TileId(pub u16);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LevelGrid {
    width: u32,
    height: u32,
    tiles: Vec<TileId>,
}

impl LevelGrid {
    #[must_use]
    pub fn new(width: u32, height: u32, fill: TileId) -> Self {
        let len = width
            .checked_mul(height)
            .expect("level grid dimensions should fit in u32") as usize;

        Self {
            width,
            height,
            tiles: vec![fill; len],
        }
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn get(&self, coord: TileCoord) -> Option<TileId> {
        self.index(coord).map(|index| self.tiles[index])
    }

    pub fn set(&mut self, coord: TileCoord, tile: TileId) -> bool {
        let Some(index) = self.index(coord) else {
            return false;
        };

        self.tiles[index] = tile;
        true
    }

    #[must_use]
    fn index(&self, coord: TileCoord) -> Option<usize> {
        let x = u32::try_from(coord.x).ok()?;
        let y = u32::try_from(coord.y).ok()?;

        if x >= self.width || y >= self.height {
            return None;
        }

        Some((y * self.width + x) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::{LevelGrid, TileCoord, TileId};

    #[test]
    fn level_grid_bounds_are_checked() {
        let mut grid = LevelGrid::new(2, 2, TileId(0));

        assert!(grid.set(TileCoord::new(1, 1), TileId(7)));
        assert_eq!(grid.get(TileCoord::new(1, 1)), Some(TileId(7)));
        assert_eq!(grid.get(TileCoord::new(-1, 0)), None);
        assert!(!grid.set(TileCoord::new(2, 0), TileId(3)));
    }
}

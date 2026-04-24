//! Parsers for legacy Mari0 mappack content files.

use std::{collections::BTreeMap, error::Error, fmt};

pub const MARI0_LEVEL_HEIGHT: usize = 15;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappackSettings {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub lives: Option<u32>,
    pub raw: BTreeMap<String, String>,
}

impl MappackSettings {
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let mut settings = Self {
            name: None,
            author: None,
            description: None,
            lives: None,
            raw: BTreeMap::new(),
        };

        for line in input.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            let key = key.trim();
            let value = value.trim();
            settings.raw.insert(key.to_owned(), value.to_owned());

            match key {
                "name" => settings.name = Some(value.to_owned()),
                "author" => settings.author = Some(value.to_owned()),
                "description" => settings.description = Some(value.to_owned()),
                "lives" => settings.lives = value.parse().ok(),
                _ => {}
            }
        }

        settings
    }

    #[must_use]
    pub fn effective_lives(&self) -> Lives {
        match self.lives.unwrap_or(3) {
            0 => Lives::Infinite,
            count => Lives::Finite(count),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lives {
    Finite(u32),
    Infinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mari0Level {
    width: usize,
    cells: Vec<LevelCell>,
    pub properties: LevelProperties,
}

impl Mari0Level {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let sections = split_preserve(input, ';');
        let map_section = sections
            .first()
            .ok_or(ParseError::MissingMapSection)?
            .trim();
        let tokens = split_preserve(map_section, ',');

        if tokens.is_empty() || tokens.len() % MARI0_LEVEL_HEIGHT != 0 {
            return Err(ParseError::InvalidCellCount {
                count: tokens.len(),
                height: MARI0_LEVEL_HEIGHT,
            });
        }

        let width = tokens.len() / MARI0_LEVEL_HEIGHT;
        let mut cells = Vec::with_capacity(tokens.len());

        for token in tokens {
            cells.push(LevelCell::parse(token.trim())?);
        }

        Ok(Self {
            width,
            cells,
            properties: LevelProperties::parse_sections(&sections[1..]),
        })
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        MARI0_LEVEL_HEIGHT
    }

    #[must_use]
    pub fn cells(&self) -> &[LevelCell] {
        &self.cells
    }

    #[must_use]
    pub fn cell(&self, x: usize, y: usize) -> Option<&LevelCell> {
        if x >= self.width || y >= MARI0_LEVEL_HEIGHT {
            return None;
        }

        self.cells.get(y * self.width + x)
    }

    pub fn set_cell_tile_id(&mut self, x: usize, y: usize, tile_id: u16) -> Option<u16> {
        if x >= self.width || y >= MARI0_LEVEL_HEIGHT {
            return None;
        }

        let cell = self.cells.get_mut(y * self.width + x)?;
        let previous = cell.tile_id();
        if let Some(value) = cell.values.first_mut() {
            *value = CellValue::Number(f64::from(tile_id));
        }

        previous
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LevelCell {
    pub values: Vec<CellValue>,
}

impl LevelCell {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::EmptyCell);
        }

        let mut values = Vec::new();

        for value in split_preserve(input, '-') {
            if value == "link" {
                values.push(CellValue::Link);
            } else {
                values.push(CellValue::Number(
                    value
                        .parse()
                        .map_err(|_| ParseError::InvalidCellValue(value.to_owned()))?,
                ));
            }
        }

        Ok(Self { values })
    }

    #[must_use]
    pub fn tile_id(&self) -> Option<u16> {
        self.values.first().and_then(CellValue::as_u16)
    }

    #[must_use]
    pub fn entity_id(&self) -> Option<u16> {
        self.values.get(1).and_then(CellValue::as_u16)
    }

    #[must_use]
    pub fn legacy_entity(&self) -> Option<LegacyEntityPlacement> {
        let id = self.entity_id()?;

        Some(LegacyEntityPlacement {
            id,
            kind: LegacyEntityKind::from_id(id),
            parameters: self.entity_parameters(),
            links: self.link_targets(),
        })
    }

    #[must_use]
    pub fn entity_parameters(&self) -> Vec<CellValue> {
        self.values
            .iter()
            .skip(2)
            .take_while(|value| **value != CellValue::Link)
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn link_targets(&self) -> Vec<LinkTarget> {
        let mut targets = Vec::new();

        for index in 0..self.values.len() {
            if self.values[index] != CellValue::Link {
                continue;
            }

            let Some(x) = self.values.get(index + 1).and_then(CellValue::as_i32) else {
                continue;
            };
            let Some(y) = self.values.get(index + 2).and_then(CellValue::as_i32) else {
                continue;
            };

            targets.push(LinkTarget { x, y });
        }

        targets
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinkTarget {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyEntityPlacement {
    pub id: u16,
    pub kind: LegacyEntityKind,
    pub parameters: Vec<CellValue>,
    pub links: Vec<LinkTarget>,
}

impl LegacyEntityPlacement {
    #[must_use]
    pub fn parameter_u16(&self, index: usize) -> Option<u16> {
        self.parameters.get(index).and_then(CellValue::as_u16)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEntityKind {
    Box(LegacyBoxEntity),
    Button,
    Drain,
    Enemy(LegacyEnemyEntity),
    FaithPlate(LegacyFaithPlateEntity),
    GelDispenser(LegacyGelDispenserEntity),
    GelTile(LegacyEntitySurface),
    Goal(LegacyGoalEntity),
    GroundLight(LegacyGroundLightEntity),
    Hazard(LegacyHazardEntity),
    Laser(LegacyLaserEntity),
    LevelControl(LegacyLevelControlEntity),
    LightBridge(LegacyLightBridgeEntity),
    Linked(LegacyLinkedEntity),
    MazeGate,
    Platform(LegacyPlatformEntity),
    Plant,
    PlayerSpawn,
    PowerUp(LegacyPowerUpEntity),
    Spring,
    Warp(LegacyWarpEntity),
    Unknown(u16),
}

impl LegacyEntityKind {
    #[must_use]
    pub const fn from_id(id: u16) -> Self {
        match id {
            2 => Self::PowerUp(LegacyPowerUpEntity::Mushroom),
            3 => Self::PowerUp(LegacyPowerUpEntity::OneUp),
            4 => Self::PowerUp(LegacyPowerUpEntity::Star),
            5 => Self::PowerUp(LegacyPowerUpEntity::ManyCoins),
            6 => Self::Enemy(LegacyEnemyEntity::Goomba),
            7 => Self::Enemy(LegacyEnemyEntity::Koopa),
            8 => Self::PlayerSpawn,
            9 => Self::Enemy(LegacyEnemyEntity::GoombaHalf),
            10 => Self::Enemy(LegacyEnemyEntity::KoopaHalf),
            11 => Self::Goal(LegacyGoalEntity::Flag),
            12 => Self::Enemy(LegacyEnemyEntity::KoopaRed),
            13 => Self::Enemy(LegacyEnemyEntity::KoopaRedHalf),
            14 => Self::Warp(LegacyWarpEntity::Vine),
            15 => Self::Enemy(LegacyEnemyEntity::HammerBro),
            16 => Self::Enemy(LegacyEnemyEntity::CheepRed),
            17 => Self::Enemy(LegacyEnemyEntity::CheepWhite),
            18 => Self::Platform(LegacyPlatformEntity::OscillatingUp),
            19 => Self::Platform(LegacyPlatformEntity::OscillatingRight),
            20 => Self::Box(LegacyBoxEntity::CompanionCube),
            21 => Self::Warp(LegacyWarpEntity::Pipe),
            22 => Self::Enemy(LegacyEnemyEntity::Lakito),
            23 => Self::LevelControl(LegacyLevelControlEntity::MazeStart),
            24 => Self::LevelControl(LegacyLevelControlEntity::MazeEnd),
            25 => Self::MazeGate,
            26 => Self::Linked(LegacyLinkedEntity::EmancipationGrillHorizontal),
            27 => Self::Linked(LegacyLinkedEntity::EmancipationGrillVertical),
            28 => Self::Linked(LegacyLinkedEntity::DoorVertical),
            29 => Self::Linked(LegacyLinkedEntity::DoorHorizontal),
            30 => Self::Linked(LegacyLinkedEntity::WallIndicator),
            31 => Self::Warp(LegacyWarpEntity::PipeSpawn),
            32 => Self::Platform(LegacyPlatformEntity::Falling),
            33 => Self::LevelControl(LegacyLevelControlEntity::BulletBillStart),
            34 => Self::LevelControl(LegacyLevelControlEntity::BulletBillEnd),
            35 => Self::Drain,
            36 => Self::LightBridge(LegacyLightBridgeEntity::Right),
            37 => Self::LightBridge(LegacyLightBridgeEntity::Left),
            38 => Self::LightBridge(LegacyLightBridgeEntity::Down),
            39 => Self::LightBridge(LegacyLightBridgeEntity::Up),
            40 => Self::Button,
            41 => Self::Platform(LegacyPlatformEntity::SpawnerDown),
            42 => Self::Platform(LegacyPlatformEntity::SpawnerUp),
            43 => Self::GroundLight(LegacyGroundLightEntity::Vertical),
            44 => Self::GroundLight(LegacyGroundLightEntity::Horizontal),
            45 => Self::GroundLight(LegacyGroundLightEntity::UpRight),
            46 => Self::GroundLight(LegacyGroundLightEntity::RightDown),
            47 => Self::GroundLight(LegacyGroundLightEntity::DownLeft),
            48 => Self::GroundLight(LegacyGroundLightEntity::LeftUp),
            49 => Self::FaithPlate(LegacyFaithPlateEntity::Up),
            50 => Self::FaithPlate(LegacyFaithPlateEntity::Right),
            51 => Self::FaithPlate(LegacyFaithPlateEntity::Left),
            52 => Self::Laser(LegacyLaserEntity::EmitterRight),
            53 => Self::Laser(LegacyLaserEntity::EmitterDown),
            54 => Self::Laser(LegacyLaserEntity::EmitterLeft),
            55 => Self::Laser(LegacyLaserEntity::EmitterUp),
            56 => Self::Laser(LegacyLaserEntity::DetectorRight),
            57 => Self::Laser(LegacyLaserEntity::DetectorDown),
            58 => Self::Laser(LegacyLaserEntity::DetectorLeft),
            59 => Self::Laser(LegacyLaserEntity::DetectorUp),
            60 => Self::Enemy(LegacyEnemyEntity::BulletBillLauncher),
            61 => Self::GelDispenser(LegacyGelDispenserEntity::BlueDown),
            62 => Self::GelDispenser(LegacyGelDispenserEntity::BlueRight),
            63 => Self::GelDispenser(LegacyGelDispenserEntity::BlueLeft),
            64 => Self::GelDispenser(LegacyGelDispenserEntity::OrangeDown),
            65 => Self::GelDispenser(LegacyGelDispenserEntity::OrangeRight),
            66 => Self::GelDispenser(LegacyGelDispenserEntity::OrangeLeft),
            67 => Self::Box(LegacyBoxEntity::Tube),
            68 => Self::Box(LegacyBoxEntity::PushButtonLeft),
            69 => Self::Box(LegacyBoxEntity::PushButtonRight),
            70 => Self::Plant,
            71 => Self::GelDispenser(LegacyGelDispenserEntity::WhiteDown),
            72 => Self::GelDispenser(LegacyGelDispenserEntity::WhiteRight),
            73 => Self::GelDispenser(LegacyGelDispenserEntity::WhiteLeft),
            74 => Self::Linked(LegacyLinkedEntity::Timer),
            75 => Self::Enemy(LegacyEnemyEntity::Beetle),
            76 => Self::Enemy(LegacyEnemyEntity::BeetleHalf),
            77 => Self::Enemy(LegacyEnemyEntity::KoopaRedFlying),
            78 => Self::Enemy(LegacyEnemyEntity::KoopaFlying),
            79 => Self::Hazard(LegacyHazardEntity::CastleFireCounterClockwise),
            80 => Self::Platform(LegacyPlatformEntity::Seesaw),
            81 => Self::Warp(LegacyWarpEntity::WarpPipe),
            82 => Self::Hazard(LegacyHazardEntity::CastleFireClockwise),
            83 => Self::LevelControl(LegacyLevelControlEntity::LakitoEnd),
            84 => Self::Linked(LegacyLinkedEntity::NotGate),
            85 => Self::GelTile(LegacyEntitySurface::Top),
            86 => Self::GelTile(LegacyEntitySurface::Left),
            87 => Self::GelTile(LegacyEntitySurface::Bottom),
            88 => Self::GelTile(LegacyEntitySurface::Right),
            89 => Self::Hazard(LegacyHazardEntity::FireStart),
            90 => Self::Enemy(LegacyEnemyEntity::Bowser),
            91 => Self::Goal(LegacyGoalEntity::Axe),
            92 => Self::Platform(LegacyPlatformEntity::Bonus),
            93 => Self::Spring,
            94 => Self::Enemy(LegacyEnemyEntity::Squid),
            95 => Self::LevelControl(LegacyLevelControlEntity::FlyingFishStart),
            96 => Self::LevelControl(LegacyLevelControlEntity::FlyingFishEnd),
            97 => Self::Hazard(LegacyHazardEntity::UpFire),
            98 => Self::Enemy(LegacyEnemyEntity::Spikey),
            99 => Self::Enemy(LegacyEnemyEntity::SpikeyHalf),
            100 => Self::Goal(LegacyGoalEntity::Checkpoint),
            _ => Self::Unknown(id),
        }
    }

    #[must_use]
    pub const fn is_typed(self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEnemyEntity {
    Beetle,
    BeetleHalf,
    Bowser,
    BulletBillLauncher,
    CheepRed,
    CheepWhite,
    Goomba,
    GoombaHalf,
    HammerBro,
    Koopa,
    KoopaFlying,
    Lakito,
    Spikey,
    SpikeyHalf,
    KoopaHalf,
    KoopaRed,
    KoopaRedFlying,
    KoopaRedHalf,
    Squid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPowerUpEntity {
    Mushroom,
    OneUp,
    Star,
    ManyCoins,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBoxEntity {
    CompanionCube,
    PushButtonLeft,
    PushButtonRight,
    Tube,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFaithPlateEntity {
    Left,
    Right,
    Up,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHazardEntity {
    CastleFireClockwise,
    CastleFireCounterClockwise,
    FireStart,
    UpFire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGoalEntity {
    Axe,
    Checkpoint,
    Flag,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLevelControlEntity {
    BulletBillEnd,
    BulletBillStart,
    FlyingFishEnd,
    FlyingFishStart,
    LakitoEnd,
    MazeEnd,
    MazeStart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLinkedEntity {
    DoorHorizontal,
    DoorVertical,
    EmancipationGrillHorizontal,
    EmancipationGrillVertical,
    NotGate,
    Timer,
    WallIndicator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlatformEntity {
    Bonus,
    Falling,
    OscillatingRight,
    OscillatingUp,
    Seesaw,
    SpawnerDown,
    SpawnerUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyWarpEntity {
    Pipe,
    PipeSpawn,
    Vine,
    WarpPipe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLightBridgeEntity {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLaserEntity {
    DetectorDown,
    DetectorLeft,
    DetectorRight,
    DetectorUp,
    EmitterDown,
    EmitterLeft,
    EmitterRight,
    EmitterUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGelDispenserEntity {
    BlueDown,
    BlueLeft,
    BlueRight,
    OrangeDown,
    OrangeLeft,
    OrangeRight,
    WhiteDown,
    WhiteLeft,
    WhiteRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEntitySurface {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGroundLightEntity {
    Horizontal,
    Vertical,
    UpRight,
    RightDown,
    DownLeft,
    LeftUp,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Number(f64),
    Link,
}

impl CellValue {
    #[must_use]
    pub fn as_u16(&self) -> Option<u16> {
        let value = self.as_integral_f64()?;

        if value < 0.0 || value > f64::from(u16::MAX) {
            return None;
        }

        Some(value as u16)
    }

    #[must_use]
    pub fn as_i32(&self) -> Option<i32> {
        let value = self.as_integral_f64()?;

        if value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
            return None;
        }

        Some(value as i32)
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Link => None,
        }
    }

    #[must_use]
    fn as_integral_f64(&self) -> Option<f64> {
        let Self::Number(value) = self else {
            return None;
        };

        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }

        Some(*value)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelProperties {
    pub background: Option<u32>,
    pub spriteset: Option<u32>,
    pub music: Option<u32>,
    pub timelimit: Option<u32>,
    pub scrollfactor: Option<f32>,
    pub intermission: bool,
    pub has_warp_zone: bool,
    pub underwater: bool,
    pub bonus_stage: bool,
    pub custom_background: bool,
    pub raw: BTreeMap<String, Option<String>>,
}

impl LevelProperties {
    fn parse_sections(sections: &[&str]) -> Self {
        let mut properties = Self::default();

        for section in sections {
            let section = section.trim();
            if section.is_empty() {
                continue;
            }

            let (key, value) = match section.split_once('=') {
                Some((key, value)) => (key.trim(), Some(value.trim())),
                None => (section, None),
            };

            properties
                .raw
                .insert(key.to_owned(), value.map(std::string::ToString::to_string));

            match (key, value) {
                ("background", Some(value)) => properties.background = value.parse().ok(),
                ("spriteset", Some(value)) => properties.spriteset = value.parse().ok(),
                ("music", Some(value)) => properties.music = value.parse().ok(),
                ("timelimit", Some(value)) => properties.timelimit = value.parse().ok(),
                ("scrollfactor", Some(value)) => properties.scrollfactor = value.parse().ok(),
                ("intermission", _) => properties.intermission = true,
                ("haswarpzone", _) => properties.has_warp_zone = true,
                ("underwater", _) => properties.underwater = true,
                ("bonusstage", _) => properties.bonus_stage = true,
                ("custombackground" | "portalbackground", _) => properties.custom_background = true,
                _ => {}
            }
        }

        properties
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    MissingMapSection,
    InvalidCellCount { count: usize, height: usize },
    EmptyCell,
    InvalidCellValue(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMapSection => write!(formatter, "level is missing a map section"),
            Self::InvalidCellCount { count, height } => {
                write!(
                    formatter,
                    "level has {count} cells, which is not divisible by height {height}"
                )
            }
            Self::EmptyCell => write!(formatter, "level contains an empty cell"),
            Self::InvalidCellValue(value) => write!(formatter, "invalid cell value: {value}"),
        }
    }
}

impl Error for ParseError {}

fn split_preserve(input: &str, delimiter: char) -> Vec<&str> {
    input.split(delimiter).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CellValue, LegacyBoxEntity, LegacyEnemyEntity, LegacyEntityKind, LegacyEntitySurface,
        LegacyFaithPlateEntity, LegacyGelDispenserEntity, LegacyGoalEntity,
        LegacyGroundLightEntity, LegacyHazardEntity, LegacyLaserEntity, LegacyLevelControlEntity,
        LegacyLightBridgeEntity, LegacyLinkedEntity, LegacyPlatformEntity, LegacyPowerUpEntity,
        LegacyWarpEntity, LevelCell, LinkTarget, Lives, MARI0_LEVEL_HEIGHT, MappackSettings,
        Mari0Level,
    };

    const SMB_SETTINGS: &str = include_str!("../../../../mappacks/smb/settings.txt");
    const UNTITLED_SETTINGS: &str =
        include_str!("../../../../mappacks/dlc_the_untitled_game/settings.txt");
    const SMB_1_1: &str = include_str!("../../../../mappacks/smb/1-1.txt");

    #[test]
    fn parses_mappack_settings_with_default_lives() {
        let settings = MappackSettings::parse(SMB_SETTINGS);

        assert_eq!(settings.name.as_deref(), Some("super mario bros."));
        assert_eq!(settings.author.as_deref(), Some("nintendo"));
        assert_eq!(settings.effective_lives(), Lives::Finite(3));
    }

    #[test]
    fn parses_zero_lives_as_infinite() {
        let settings = MappackSettings::parse(UNTITLED_SETTINGS);

        assert_eq!(settings.name.as_deref(), Some("the untitled game"));
        assert_eq!(settings.lives, Some(0));
        assert_eq!(settings.effective_lives(), Lives::Infinite);
    }

    #[test]
    fn parses_level_cell_link_tokens() {
        let cell = LevelCell::parse("134-46-link-33-13").expect("cell should parse");

        assert_eq!(
            cell.values,
            vec![
                CellValue::Number(134.0),
                CellValue::Number(46.0),
                CellValue::Link,
                CellValue::Number(33.0),
                CellValue::Number(13.0),
            ]
        );
        assert_eq!(cell.tile_id(), Some(134));
        assert_eq!(cell.entity_id(), Some(46));
        assert_eq!(cell.link_targets(), vec![LinkTarget { x: 33, y: 13 }]);
    }

    #[test]
    fn preserves_fractional_cell_values_but_not_as_ids() {
        let cell = LevelCell::parse("1-18-1.5").expect("cell should parse");

        assert_eq!(cell.values.get(2).and_then(CellValue::as_f64), Some(1.5));
        assert_eq!(cell.values.get(2).and_then(CellValue::as_u16), None);
    }

    #[test]
    fn interprets_high_use_legacy_entity_ids() {
        assert_eq!(
            LegacyEntityKind::from_id(44),
            LegacyEntityKind::GroundLight(LegacyGroundLightEntity::Horizontal)
        );
        assert_eq!(
            LegacyEntityKind::from_id(85),
            LegacyEntityKind::GelTile(LegacyEntitySurface::Top)
        );
        assert_eq!(LegacyEntityKind::from_id(35), LegacyEntityKind::Drain);
        assert_eq!(LegacyEntityKind::from_id(70), LegacyEntityKind::Plant);
        assert_eq!(LegacyEntityKind::from_id(25), LegacyEntityKind::MazeGate);
        assert_eq!(
            LegacyEntityKind::from_id(6),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Goomba)
        );
        assert_eq!(
            LegacyEntityKind::from_id(7),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Koopa)
        );
        assert_eq!(
            LegacyEntityKind::from_id(9),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::GoombaHalf)
        );
        assert_eq!(
            LegacyEntityKind::from_id(10),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::KoopaHalf)
        );
        assert_eq!(
            LegacyEntityKind::from_id(11),
            LegacyEntityKind::Goal(LegacyGoalEntity::Flag)
        );
        assert_eq!(
            LegacyEntityKind::from_id(12),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::KoopaRed)
        );
        assert_eq!(
            LegacyEntityKind::from_id(13),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::KoopaRedHalf)
        );
        assert_eq!(
            LegacyEntityKind::from_id(14),
            LegacyEntityKind::Warp(LegacyWarpEntity::Vine)
        );
        assert_eq!(
            LegacyEntityKind::from_id(15),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::HammerBro)
        );
        assert_eq!(
            LegacyEntityKind::from_id(16),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::CheepRed)
        );
        assert_eq!(
            LegacyEntityKind::from_id(17),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::CheepWhite)
        );
        assert_eq!(
            LegacyEntityKind::from_id(18),
            LegacyEntityKind::Platform(LegacyPlatformEntity::OscillatingUp)
        );
        assert_eq!(
            LegacyEntityKind::from_id(19),
            LegacyEntityKind::Platform(LegacyPlatformEntity::OscillatingRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(20),
            LegacyEntityKind::Box(LegacyBoxEntity::CompanionCube)
        );
        assert_eq!(
            LegacyEntityKind::from_id(22),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Lakito)
        );
        assert_eq!(
            LegacyEntityKind::from_id(23),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::MazeStart)
        );
        assert_eq!(
            LegacyEntityKind::from_id(24),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::MazeEnd)
        );
        assert_eq!(
            LegacyEntityKind::from_id(26),
            LegacyEntityKind::Linked(LegacyLinkedEntity::EmancipationGrillHorizontal)
        );
        assert_eq!(
            LegacyEntityKind::from_id(27),
            LegacyEntityKind::Linked(LegacyLinkedEntity::EmancipationGrillVertical)
        );
        assert_eq!(
            LegacyEntityKind::from_id(28),
            LegacyEntityKind::Linked(LegacyLinkedEntity::DoorVertical)
        );
        assert_eq!(
            LegacyEntityKind::from_id(29),
            LegacyEntityKind::Linked(LegacyLinkedEntity::DoorHorizontal)
        );
        assert_eq!(
            LegacyEntityKind::from_id(30),
            LegacyEntityKind::Linked(LegacyLinkedEntity::WallIndicator)
        );
        assert_eq!(
            LegacyEntityKind::from_id(31),
            LegacyEntityKind::Warp(LegacyWarpEntity::PipeSpawn)
        );
        assert_eq!(
            LegacyEntityKind::from_id(32),
            LegacyEntityKind::Platform(LegacyPlatformEntity::Falling)
        );
        assert_eq!(
            LegacyEntityKind::from_id(33),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::BulletBillStart)
        );
        assert_eq!(
            LegacyEntityKind::from_id(34),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::BulletBillEnd)
        );
        assert_eq!(
            LegacyEntityKind::from_id(36),
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Right)
        );
        assert_eq!(
            LegacyEntityKind::from_id(37),
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Left)
        );
        assert_eq!(
            LegacyEntityKind::from_id(38),
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Down)
        );
        assert_eq!(
            LegacyEntityKind::from_id(39),
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Up)
        );
        assert_eq!(
            LegacyEntityKind::from_id(49),
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Up)
        );
        assert_eq!(
            LegacyEntityKind::from_id(50),
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Right)
        );
        assert_eq!(
            LegacyEntityKind::from_id(51),
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Left)
        );
        assert_eq!(
            LegacyEntityKind::from_id(52),
            LegacyEntityKind::Laser(LegacyLaserEntity::EmitterRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(53),
            LegacyEntityKind::Laser(LegacyLaserEntity::EmitterDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(54),
            LegacyEntityKind::Laser(LegacyLaserEntity::EmitterLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(55),
            LegacyEntityKind::Laser(LegacyLaserEntity::EmitterUp)
        );
        assert_eq!(
            LegacyEntityKind::from_id(56),
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(57),
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(58),
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(59),
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorUp)
        );
        assert_eq!(
            LegacyEntityKind::from_id(60),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::BulletBillLauncher)
        );
        assert_eq!(
            LegacyEntityKind::from_id(61),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::BlueDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(62),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::BlueRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(63),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::BlueLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(64),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::OrangeDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(65),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::OrangeRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(66),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::OrangeLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(67),
            LegacyEntityKind::Box(LegacyBoxEntity::Tube)
        );
        assert_eq!(
            LegacyEntityKind::from_id(68),
            LegacyEntityKind::Box(LegacyBoxEntity::PushButtonLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(69),
            LegacyEntityKind::Box(LegacyBoxEntity::PushButtonRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(41),
            LegacyEntityKind::Platform(LegacyPlatformEntity::SpawnerDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(42),
            LegacyEntityKind::Platform(LegacyPlatformEntity::SpawnerUp)
        );
        assert_eq!(
            LegacyEntityKind::from_id(21),
            LegacyEntityKind::Warp(LegacyWarpEntity::Pipe)
        );
        assert_eq!(LegacyEntityKind::from_id(8), LegacyEntityKind::PlayerSpawn);
        assert_eq!(
            LegacyEntityKind::from_id(2),
            LegacyEntityKind::PowerUp(LegacyPowerUpEntity::Mushroom)
        );
        assert_eq!(
            LegacyEntityKind::from_id(74),
            LegacyEntityKind::Linked(LegacyLinkedEntity::Timer)
        );
        assert_eq!(
            LegacyEntityKind::from_id(75),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Beetle)
        );
        assert_eq!(
            LegacyEntityKind::from_id(76),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::BeetleHalf)
        );
        assert_eq!(
            LegacyEntityKind::from_id(77),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::KoopaRedFlying)
        );
        assert_eq!(
            LegacyEntityKind::from_id(78),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::KoopaFlying)
        );
        assert_eq!(
            LegacyEntityKind::from_id(71),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::WhiteDown)
        );
        assert_eq!(
            LegacyEntityKind::from_id(72),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::WhiteRight)
        );
        assert_eq!(
            LegacyEntityKind::from_id(73),
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::WhiteLeft)
        );
        assert_eq!(
            LegacyEntityKind::from_id(79),
            LegacyEntityKind::Hazard(LegacyHazardEntity::CastleFireCounterClockwise)
        );
        assert_eq!(
            LegacyEntityKind::from_id(84),
            LegacyEntityKind::Linked(LegacyLinkedEntity::NotGate)
        );
        assert_eq!(
            LegacyEntityKind::from_id(80),
            LegacyEntityKind::Platform(LegacyPlatformEntity::Seesaw)
        );
        assert_eq!(
            LegacyEntityKind::from_id(81),
            LegacyEntityKind::Warp(LegacyWarpEntity::WarpPipe)
        );
        assert_eq!(
            LegacyEntityKind::from_id(82),
            LegacyEntityKind::Hazard(LegacyHazardEntity::CastleFireClockwise)
        );
        assert_eq!(
            LegacyEntityKind::from_id(83),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::LakitoEnd)
        );
        assert_eq!(
            LegacyEntityKind::from_id(89),
            LegacyEntityKind::Hazard(LegacyHazardEntity::FireStart)
        );
        assert_eq!(
            LegacyEntityKind::from_id(91),
            LegacyEntityKind::Goal(LegacyGoalEntity::Axe)
        );
        assert_eq!(
            LegacyEntityKind::from_id(92),
            LegacyEntityKind::Platform(LegacyPlatformEntity::Bonus)
        );
        assert_eq!(LegacyEntityKind::from_id(93), LegacyEntityKind::Spring);
        assert_eq!(
            LegacyEntityKind::from_id(94),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Squid)
        );
        assert_eq!(
            LegacyEntityKind::from_id(95),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::FlyingFishStart)
        );
        assert_eq!(
            LegacyEntityKind::from_id(96),
            LegacyEntityKind::LevelControl(LegacyLevelControlEntity::FlyingFishEnd)
        );
        assert_eq!(
            LegacyEntityKind::from_id(97),
            LegacyEntityKind::Hazard(LegacyHazardEntity::UpFire)
        );
        assert_eq!(
            LegacyEntityKind::from_id(98),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Spikey)
        );
        assert_eq!(
            LegacyEntityKind::from_id(99),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::SpikeyHalf)
        );
        assert_eq!(
            LegacyEntityKind::from_id(100),
            LegacyEntityKind::Goal(LegacyGoalEntity::Checkpoint)
        );
        assert_eq!(LegacyEntityKind::from_id(40), LegacyEntityKind::Button);
        assert_eq!(
            LegacyEntityKind::from_id(90),
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Bowser)
        );
    }

    #[test]
    fn legacy_entity_placement_preserves_parameters_and_links() {
        let pipe = LevelCell::parse("80-21-3").expect("pipe cell should parse");
        let pipe_entity = pipe.legacy_entity().expect("pipe should have entity");

        assert_eq!(pipe_entity.id, 21);
        assert_eq!(
            pipe_entity.kind,
            LegacyEntityKind::Warp(LegacyWarpEntity::Pipe)
        );
        assert_eq!(pipe_entity.parameter_u16(0), Some(3));
        assert_eq!(pipe_entity.links, Vec::new());

        let linked_light =
            LevelCell::parse("134-46-link-33-13").expect("ground light cell should parse");
        let linked_entity = linked_light
            .legacy_entity()
            .expect("ground light should have entity");

        assert_eq!(
            linked_entity.kind,
            LegacyEntityKind::GroundLight(LegacyGroundLightEntity::RightDown)
        );
        assert!(linked_entity.parameters.is_empty());
        assert_eq!(linked_entity.links, vec![LinkTarget { x: 33, y: 13 }]);
    }

    #[test]
    fn power_up_legacy_entities_map_block_rewards() {
        let cases = [
            (2, LegacyPowerUpEntity::Mushroom),
            (3, LegacyPowerUpEntity::OneUp),
            (4, LegacyPowerUpEntity::Star),
            (5, LegacyPowerUpEntity::ManyCoins),
        ];

        for (id, expected) in cases {
            let cell = LevelCell::parse(&format!("1-{id}")).expect("power-up cell should parse");
            let entity = cell.legacy_entity().expect("power-up should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::PowerUp(expected));
            assert!(entity.parameters.is_empty());
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn singleton_legacy_entities_preserve_marker_payloads() {
        let spawn = LevelCell::parse("1-8").expect("player spawn cell should parse");
        let spawn_entity = spawn
            .legacy_entity()
            .expect("player spawn should have entity");

        assert_eq!(spawn_entity.kind, LegacyEntityKind::PlayerSpawn);
        assert!(spawn_entity.parameters.is_empty());
        assert!(spawn_entity.links.is_empty());

        let maze_gate = LevelCell::parse("1-25-4").expect("maze gate cell should parse");
        let maze_gate_entity = maze_gate
            .legacy_entity()
            .expect("maze gate should have entity");

        assert_eq!(maze_gate_entity.kind, LegacyEntityKind::MazeGate);
        assert_eq!(maze_gate_entity.parameter_u16(0), Some(4));
        assert!(maze_gate_entity.links.is_empty());

        let drain = LevelCell::parse("1-35").expect("drain cell should parse");
        let drain_entity = drain.legacy_entity().expect("drain should have entity");

        assert_eq!(drain_entity.kind, LegacyEntityKind::Drain);
        assert!(drain_entity.parameters.is_empty());
        assert!(drain_entity.links.is_empty());

        let button = LevelCell::parse("1-40-link-7-3").expect("button cell should parse");
        let button_entity = button.legacy_entity().expect("button should have entity");

        assert_eq!(button_entity.kind, LegacyEntityKind::Button);
        assert!(button_entity.parameters.is_empty());
        assert_eq!(button_entity.links, vec![LinkTarget { x: 7, y: 3 }]);

        let plant = LevelCell::parse("1-70").expect("plant cell should parse");
        let plant_entity = plant.legacy_entity().expect("plant should have entity");

        assert_eq!(plant_entity.kind, LegacyEntityKind::Plant);
        assert!(plant_entity.parameters.is_empty());
        assert!(plant_entity.links.is_empty());
    }

    #[test]
    fn ground_light_legacy_entities_map_all_orientations_and_preserve_links() {
        let cases = [
            (43, LegacyGroundLightEntity::Vertical),
            (44, LegacyGroundLightEntity::Horizontal),
            (45, LegacyGroundLightEntity::UpRight),
            (46, LegacyGroundLightEntity::RightDown),
            (47, LegacyGroundLightEntity::DownLeft),
            (48, LegacyGroundLightEntity::LeftUp),
        ];

        for (id, expected) in cases {
            let cell = LevelCell::parse(&format!("1-{id}-link-2-11"))
                .expect("ground light cell should parse");
            let entity = cell
                .legacy_entity()
                .expect("ground light should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::GroundLight(expected));
            assert!(entity.parameters.is_empty());
            assert_eq!(entity.links, vec![LinkTarget { x: 2, y: 11 }]);
        }
    }

    #[test]
    fn gel_tile_legacy_entities_preserve_surface_and_gel_id_parameter() {
        let cases = [
            (85, LegacyEntitySurface::Top),
            (86, LegacyEntitySurface::Left),
            (87, LegacyEntitySurface::Bottom),
            (88, LegacyEntitySurface::Right),
        ];

        for (id, expected) in cases {
            let cell = LevelCell::parse(&format!("1-{id}-3")).expect("gel tile cell should parse");
            let entity = cell.legacy_entity().expect("gel tile should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::GelTile(expected));
            assert_eq!(entity.parameter_u16(0), Some(3));
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn linked_legacy_entities_preserve_links_and_timer_parameters() {
        let door = LevelCell::parse("1-28-link-9-4").expect("door cell should parse");
        let door_entity = door.legacy_entity().expect("door should have entity");

        assert_eq!(
            door_entity.kind,
            LegacyEntityKind::Linked(LegacyLinkedEntity::DoorVertical)
        );
        assert!(door_entity.parameters.is_empty());
        assert_eq!(door_entity.links, vec![LinkTarget { x: 9, y: 4 }]);

        let not_gate =
            LevelCell::parse("1-84-link-12-8-link-13-8").expect("not gate cell should parse");
        let not_gate_entity = not_gate
            .legacy_entity()
            .expect("not gate should have entity");

        assert_eq!(
            not_gate_entity.kind,
            LegacyEntityKind::Linked(LegacyLinkedEntity::NotGate)
        );
        assert_eq!(
            not_gate_entity.links,
            vec![LinkTarget { x: 12, y: 8 }, LinkTarget { x: 13, y: 8 }]
        );

        let timer = LevelCell::parse("1-74-4-link-5-6").expect("timer cell should parse");
        let timer_entity = timer.legacy_entity().expect("timer should have entity");

        assert_eq!(
            timer_entity.kind,
            LegacyEntityKind::Linked(LegacyLinkedEntity::Timer)
        );
        assert_eq!(timer_entity.parameter_u16(0), Some(4));
        assert_eq!(timer_entity.links, vec![LinkTarget { x: 5, y: 6 }]);
    }

    #[test]
    fn platform_legacy_entities_preserve_width_and_type_parameters() {
        let oscillating_up =
            LevelCell::parse("1-18-1.5").expect("oscillating platform cell should parse");
        let oscillating_up_entity = oscillating_up
            .legacy_entity()
            .expect("oscillating platform should have entity");

        assert_eq!(
            oscillating_up_entity.kind,
            LegacyEntityKind::Platform(LegacyPlatformEntity::OscillatingUp)
        );
        assert_eq!(
            oscillating_up_entity
                .parameters
                .first()
                .and_then(CellValue::as_f64),
            Some(1.5)
        );
        assert_eq!(oscillating_up_entity.parameter_u16(0), None);

        let spawner_down = LevelCell::parse("1-41-3").expect("platform spawner cell should parse");
        let spawner_down_entity = spawner_down
            .legacy_entity()
            .expect("platform spawner should have entity");

        assert_eq!(
            spawner_down_entity.kind,
            LegacyEntityKind::Platform(LegacyPlatformEntity::SpawnerDown)
        );
        assert_eq!(spawner_down_entity.parameter_u16(0), Some(3));

        let seesaw = LevelCell::parse("1-80-9").expect("seesaw cell should parse");
        let seesaw_entity = seesaw.legacy_entity().expect("seesaw should have entity");

        assert_eq!(
            seesaw_entity.kind,
            LegacyEntityKind::Platform(LegacyPlatformEntity::Seesaw)
        );
        assert_eq!(seesaw_entity.parameter_u16(0), Some(9));

        let bonus_platform = LevelCell::parse("1-92").expect("bonus platform cell should parse");
        let bonus_platform_entity = bonus_platform
            .legacy_entity()
            .expect("bonus platform should have entity");

        assert_eq!(
            bonus_platform_entity.kind,
            LegacyEntityKind::Platform(LegacyPlatformEntity::Bonus)
        );
        assert!(bonus_platform_entity.parameters.is_empty());
    }

    #[test]
    fn warp_legacy_entities_preserve_destination_parameters() {
        let pipe_spawn = LevelCell::parse("1-31-0").expect("pipe spawn cell should parse");
        let pipe_spawn_entity = pipe_spawn
            .legacy_entity()
            .expect("pipe spawn should have entity");

        assert_eq!(
            pipe_spawn_entity.kind,
            LegacyEntityKind::Warp(LegacyWarpEntity::PipeSpawn)
        );
        assert_eq!(pipe_spawn_entity.parameter_u16(0), Some(0));

        let warp_pipe = LevelCell::parse("1-81-8").expect("warp pipe cell should parse");
        let warp_pipe_entity = warp_pipe
            .legacy_entity()
            .expect("warp pipe should have entity");

        assert_eq!(
            warp_pipe_entity.kind,
            LegacyEntityKind::Warp(LegacyWarpEntity::WarpPipe)
        );
        assert_eq!(warp_pipe_entity.parameter_u16(0), Some(8));

        let vine = LevelCell::parse("1-14-5").expect("vine cell should parse");
        let vine_entity = vine.legacy_entity().expect("vine should have entity");

        assert_eq!(
            vine_entity.kind,
            LegacyEntityKind::Warp(LegacyWarpEntity::Vine)
        );
        assert_eq!(vine_entity.parameter_u16(0), Some(5));
        assert!(vine_entity.links.is_empty());
    }

    #[test]
    fn lightbridge_legacy_entities_map_direction_only_payloads() {
        let right = LevelCell::parse("1-36").expect("right lightbridge cell should parse");
        let right_entity = right
            .legacy_entity()
            .expect("right lightbridge should have entity");

        assert_eq!(
            right_entity.kind,
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Right)
        );
        assert!(right_entity.parameters.is_empty());
        assert!(right_entity.links.is_empty());

        let up = LevelCell::parse("1-39").expect("up lightbridge cell should parse");
        let up_entity = up
            .legacy_entity()
            .expect("up lightbridge should have entity");

        assert_eq!(
            up_entity.kind,
            LegacyEntityKind::LightBridge(LegacyLightBridgeEntity::Up)
        );
        assert!(up_entity.parameters.is_empty());
        assert!(up_entity.links.is_empty());
    }

    #[test]
    fn laser_legacy_entities_map_emitters_and_preserve_detector_links() {
        let emitter = LevelCell::parse("1-52").expect("laser emitter cell should parse");
        let emitter_entity = emitter
            .legacy_entity()
            .expect("laser emitter should have entity");

        assert_eq!(
            emitter_entity.kind,
            LegacyEntityKind::Laser(LegacyLaserEntity::EmitterRight)
        );
        assert!(emitter_entity.parameters.is_empty());
        assert!(emitter_entity.links.is_empty());

        let detector =
            LevelCell::parse("1-56-link-12-8").expect("laser detector cell should parse");
        let detector_entity = detector
            .legacy_entity()
            .expect("laser detector should have entity");

        assert_eq!(
            detector_entity.kind,
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorRight)
        );
        assert!(detector_entity.parameters.is_empty());
        assert_eq!(detector_entity.links, vec![LinkTarget { x: 12, y: 8 }]);

        let upward_detector =
            LevelCell::parse("1-59-link-4-2").expect("up laser detector cell should parse");
        let upward_detector_entity = upward_detector
            .legacy_entity()
            .expect("up laser detector should have entity");

        assert_eq!(
            upward_detector_entity.kind,
            LegacyEntityKind::Laser(LegacyLaserEntity::DetectorUp)
        );
        assert_eq!(
            upward_detector_entity.links,
            vec![LinkTarget { x: 4, y: 2 }]
        );
    }

    #[test]
    fn gel_dispenser_legacy_entities_map_color_and_direction_payloads() {
        let blue = LevelCell::parse("1-61").expect("blue gel dispenser cell should parse");
        let blue_entity = blue
            .legacy_entity()
            .expect("blue gel dispenser should have entity");

        assert_eq!(
            blue_entity.kind,
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::BlueDown)
        );
        assert!(blue_entity.parameters.is_empty());
        assert!(blue_entity.links.is_empty());

        let orange = LevelCell::parse("1-65").expect("orange gel dispenser cell should parse");
        let orange_entity = orange
            .legacy_entity()
            .expect("orange gel dispenser should have entity");

        assert_eq!(
            orange_entity.kind,
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::OrangeRight)
        );
        assert!(orange_entity.parameters.is_empty());
        assert!(orange_entity.links.is_empty());

        let white = LevelCell::parse("1-73").expect("white gel dispenser cell should parse");
        let white_entity = white
            .legacy_entity()
            .expect("white gel dispenser should have entity");

        assert_eq!(
            white_entity.kind,
            LegacyEntityKind::GelDispenser(LegacyGelDispenserEntity::WhiteLeft)
        );
        assert!(white_entity.parameters.is_empty());
        assert!(white_entity.links.is_empty());
    }

    #[test]
    fn faithplate_legacy_entities_map_direction_only_payloads() {
        let up = LevelCell::parse("1-49").expect("up faithplate cell should parse");
        let up_entity = up
            .legacy_entity()
            .expect("up faithplate should have entity");

        assert_eq!(
            up_entity.kind,
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Up)
        );
        assert!(up_entity.parameters.is_empty());
        assert!(up_entity.links.is_empty());

        let right = LevelCell::parse("1-50").expect("right faithplate cell should parse");
        let right_entity = right
            .legacy_entity()
            .expect("right faithplate should have entity");

        assert_eq!(
            right_entity.kind,
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Right)
        );
        assert!(right_entity.parameters.is_empty());
        assert!(right_entity.links.is_empty());

        let left = LevelCell::parse("1-51").expect("left faithplate cell should parse");
        let left_entity = left
            .legacy_entity()
            .expect("left faithplate should have entity");

        assert_eq!(
            left_entity.kind,
            LegacyEntityKind::FaithPlate(LegacyFaithPlateEntity::Left)
        );
        assert!(left_entity.parameters.is_empty());
        assert!(left_entity.links.is_empty());
    }

    #[test]
    fn box_legacy_entities_preserve_button_and_tube_links() {
        let cube = LevelCell::parse("1-20").expect("companion cube cell should parse");
        let cube_entity = cube
            .legacy_entity()
            .expect("companion cube should have entity");

        assert_eq!(
            cube_entity.kind,
            LegacyEntityKind::Box(LegacyBoxEntity::CompanionCube)
        );
        assert!(cube_entity.parameters.is_empty());
        assert!(cube_entity.links.is_empty());

        let tube = LevelCell::parse("1-67-link-9-4").expect("box tube cell should parse");
        let tube_entity = tube.legacy_entity().expect("box tube should have entity");

        assert_eq!(
            tube_entity.kind,
            LegacyEntityKind::Box(LegacyBoxEntity::Tube)
        );
        assert!(tube_entity.parameters.is_empty());
        assert_eq!(tube_entity.links, vec![LinkTarget { x: 9, y: 4 }]);

        let pushbutton = LevelCell::parse("1-68-link-12-8").expect("pushbutton cell should parse");
        let pushbutton_entity = pushbutton
            .legacy_entity()
            .expect("pushbutton should have entity");

        assert_eq!(
            pushbutton_entity.kind,
            LegacyEntityKind::Box(LegacyBoxEntity::PushButtonLeft)
        );
        assert!(pushbutton_entity.parameters.is_empty());
        assert_eq!(pushbutton_entity.links, vec![LinkTarget { x: 12, y: 8 }]);

        let right_pushbutton =
            LevelCell::parse("1-69-link-13-8").expect("right pushbutton cell should parse");
        let right_pushbutton_entity = right_pushbutton
            .legacy_entity()
            .expect("right pushbutton should have entity");

        assert_eq!(
            right_pushbutton_entity.kind,
            LegacyEntityKind::Box(LegacyBoxEntity::PushButtonRight)
        );
        assert_eq!(
            right_pushbutton_entity.links,
            vec![LinkTarget { x: 13, y: 8 }]
        );
    }

    #[test]
    fn fire_hazard_legacy_entities_preserve_castle_fire_length_parameters() {
        let counter_clockwise =
            LevelCell::parse("1-79-12").expect("counterclockwise castle fire cell should parse");
        let counter_clockwise_entity = counter_clockwise
            .legacy_entity()
            .expect("counterclockwise castle fire should have entity");

        assert_eq!(
            counter_clockwise_entity.kind,
            LegacyEntityKind::Hazard(LegacyHazardEntity::CastleFireCounterClockwise)
        );
        assert_eq!(counter_clockwise_entity.parameter_u16(0), Some(12));
        assert!(counter_clockwise_entity.links.is_empty());

        let clockwise =
            LevelCell::parse("1-82-6").expect("clockwise castle fire cell should parse");
        let clockwise_entity = clockwise
            .legacy_entity()
            .expect("clockwise castle fire should have entity");

        assert_eq!(
            clockwise_entity.kind,
            LegacyEntityKind::Hazard(LegacyHazardEntity::CastleFireClockwise)
        );
        assert_eq!(clockwise_entity.parameter_u16(0), Some(6));
        assert!(clockwise_entity.links.is_empty());

        let fire_start = LevelCell::parse("1-89").expect("fire start cell should parse");
        let fire_start_entity = fire_start
            .legacy_entity()
            .expect("fire start should have entity");

        assert_eq!(
            fire_start_entity.kind,
            LegacyEntityKind::Hazard(LegacyHazardEntity::FireStart)
        );
        assert!(fire_start_entity.parameters.is_empty());
        assert!(fire_start_entity.links.is_empty());

        let upfire = LevelCell::parse("1-97").expect("upfire cell should parse");
        let upfire_entity = upfire.legacy_entity().expect("upfire should have entity");

        assert_eq!(
            upfire_entity.kind,
            LegacyEntityKind::Hazard(LegacyHazardEntity::UpFire)
        );
        assert!(upfire_entity.parameters.is_empty());
        assert!(upfire_entity.links.is_empty());
    }

    #[test]
    fn goal_legacy_entities_map_progression_markers() {
        let flag = LevelCell::parse("1-11").expect("flag cell should parse");
        let flag_entity = flag.legacy_entity().expect("flag should have entity");

        assert_eq!(
            flag_entity.kind,
            LegacyEntityKind::Goal(LegacyGoalEntity::Flag)
        );
        assert!(flag_entity.parameters.is_empty());
        assert!(flag_entity.links.is_empty());

        let axe = LevelCell::parse("1-91").expect("axe cell should parse");
        let axe_entity = axe.legacy_entity().expect("axe should have entity");

        assert_eq!(
            axe_entity.kind,
            LegacyEntityKind::Goal(LegacyGoalEntity::Axe)
        );
        assert!(axe_entity.parameters.is_empty());
        assert!(axe_entity.links.is_empty());

        let checkpoint = LevelCell::parse("1-100").expect("checkpoint cell should parse");
        let checkpoint_entity = checkpoint
            .legacy_entity()
            .expect("checkpoint should have entity");

        assert_eq!(
            checkpoint_entity.kind,
            LegacyEntityKind::Goal(LegacyGoalEntity::Checkpoint)
        );
        assert!(checkpoint_entity.parameters.is_empty());
        assert!(checkpoint_entity.links.is_empty());
    }

    #[test]
    fn boss_legacy_entity_maps_bowser_marker() {
        let bowser = LevelCell::parse("1-90").expect("bowser cell should parse");
        let bowser_entity = bowser.legacy_entity().expect("bowser should have entity");

        assert_eq!(
            bowser_entity.kind,
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Bowser)
        );
        assert!(bowser_entity.parameters.is_empty());
        assert!(bowser_entity.links.is_empty());
    }

    #[test]
    fn bullet_bill_legacy_entity_maps_launcher_marker() {
        let bullet_bill = LevelCell::parse("1-60").expect("bullet bill cell should parse");
        let bullet_bill_entity = bullet_bill
            .legacy_entity()
            .expect("bullet bill should have entity");

        assert_eq!(
            bullet_bill_entity.kind,
            LegacyEntityKind::Enemy(LegacyEnemyEntity::BulletBillLauncher)
        );
        assert!(bullet_bill_entity.parameters.is_empty());
        assert!(bullet_bill_entity.links.is_empty());
    }

    #[test]
    fn goomba_family_legacy_entities_map_half_tile_variants() {
        let cases = [
            (9, LegacyEnemyEntity::GoombaHalf),
            (98, LegacyEnemyEntity::Spikey),
            (99, LegacyEnemyEntity::SpikeyHalf),
        ];

        for (id, expected) in cases {
            let cell = LevelCell::parse(&format!("1-{id}"))
                .expect("goomba-family enemy cell should parse");
            let entity = cell
                .legacy_entity()
                .expect("goomba-family enemy should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::Enemy(expected));
            assert!(entity.parameters.is_empty());
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn koopa_family_legacy_entities_map_half_red_beetle_and_flying_variants() {
        let cases = [
            (10, LegacyEnemyEntity::KoopaHalf),
            (12, LegacyEnemyEntity::KoopaRed),
            (13, LegacyEnemyEntity::KoopaRedHalf),
            (75, LegacyEnemyEntity::Beetle),
            (76, LegacyEnemyEntity::BeetleHalf),
            (77, LegacyEnemyEntity::KoopaRedFlying),
            (78, LegacyEnemyEntity::KoopaFlying),
        ];

        for (id, expected) in cases {
            let cell =
                LevelCell::parse(&format!("1-{id}")).expect("koopa-family enemy cell should parse");
            let entity = cell
                .legacy_entity()
                .expect("koopa-family enemy should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::Enemy(expected));
            assert!(entity.parameters.is_empty());
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn cheep_family_legacy_entities_map_underwater_variants() {
        let cases = [
            (16, LegacyEnemyEntity::CheepRed),
            (17, LegacyEnemyEntity::CheepWhite),
        ];

        for (id, expected) in cases {
            let cell =
                LevelCell::parse(&format!("1-{id}")).expect("cheep-family enemy cell should parse");
            let entity = cell
                .legacy_entity()
                .expect("cheep-family enemy should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::Enemy(expected));
            assert!(entity.parameters.is_empty());
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn spring_legacy_entity_maps_player_interaction_marker() {
        let spring = LevelCell::parse("1-93").expect("spring cell should parse");
        let spring_entity = spring.legacy_entity().expect("spring should have entity");

        assert_eq!(spring_entity.kind, LegacyEntityKind::Spring);
        assert!(spring_entity.parameters.is_empty());
        assert!(spring_entity.links.is_empty());
    }

    #[test]
    fn squid_legacy_entity_maps_underwater_enemy_marker() {
        let squid = LevelCell::parse("1-94").expect("squid cell should parse");
        let squid_entity = squid.legacy_entity().expect("squid should have entity");

        assert_eq!(
            squid_entity.kind,
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Squid)
        );
        assert!(squid_entity.parameters.is_empty());
        assert!(squid_entity.links.is_empty());
    }

    #[test]
    fn hammer_bro_legacy_entity_maps_enemy_marker() {
        let hammer_bro = LevelCell::parse("1-15").expect("hammer bro cell should parse");
        let hammer_bro_entity = hammer_bro
            .legacy_entity()
            .expect("hammer bro should have entity");

        assert_eq!(
            hammer_bro_entity.kind,
            LegacyEntityKind::Enemy(LegacyEnemyEntity::HammerBro)
        );
        assert!(hammer_bro_entity.parameters.is_empty());
        assert!(hammer_bro_entity.links.is_empty());
    }

    #[test]
    fn lakito_legacy_entity_maps_enemy_marker() {
        let lakito = LevelCell::parse("1-22").expect("lakito cell should parse");
        let lakito_entity = lakito.legacy_entity().expect("lakito should have entity");

        assert_eq!(
            lakito_entity.kind,
            LegacyEntityKind::Enemy(LegacyEnemyEntity::Lakito)
        );
        assert!(lakito_entity.parameters.is_empty());
        assert!(lakito_entity.links.is_empty());
    }

    #[test]
    fn level_control_legacy_entities_map_boundary_markers() {
        let cases = [
            (23, LegacyLevelControlEntity::MazeStart),
            (24, LegacyLevelControlEntity::MazeEnd),
            (33, LegacyLevelControlEntity::BulletBillStart),
            (34, LegacyLevelControlEntity::BulletBillEnd),
            (83, LegacyLevelControlEntity::LakitoEnd),
            (95, LegacyLevelControlEntity::FlyingFishStart),
            (96, LegacyLevelControlEntity::FlyingFishEnd),
        ];

        for (id, expected) in cases {
            let cell = LevelCell::parse(&format!("1-{id}"))
                .expect("level-control marker cell should parse");
            let entity = cell
                .legacy_entity()
                .expect("level-control marker should have entity");

            assert_eq!(entity.kind, LegacyEntityKind::LevelControl(expected));
            assert!(entity.parameters.is_empty());
            assert!(entity.links.is_empty());
        }
    }

    #[test]
    fn parses_legacy_smb_level_grid_and_properties() {
        let level = Mari0Level::parse(SMB_1_1).expect("SMB 1-1 should parse");

        assert_eq!(level.width(), 224);
        assert_eq!(level.height(), MARI0_LEVEL_HEIGHT);
        assert_eq!(level.cells().len(), 224 * MARI0_LEVEL_HEIGHT);
        assert_eq!(level.cell(0, 0).and_then(LevelCell::tile_id), Some(1));
        assert_eq!(level.properties.background, Some(1));
        assert_eq!(level.properties.spriteset, Some(1));
        assert_eq!(level.properties.music, Some(2));
        assert_eq!(level.properties.timelimit, Some(400));
    }

    #[test]
    fn rejects_wrong_height_cell_count() {
        let error = Mari0Level::parse("1,2,3;background=1").expect_err("cell count should fail");

        assert_eq!(
            error.to_string(),
            "level has 3 cells, which is not divisible by height 15"
        );
    }
}

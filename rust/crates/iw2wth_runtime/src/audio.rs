//! Audio intent adapter boundary for legacy Mari0 sound effects.
//!
//! Lua gameplay code calls `playsound(sound)`, which checks the global
//! `soundenabled` flag and then restarts the LÖVE source by stopping it before
//! playing it again. This module preserves that command shape and the legacy
//! sound-effect asset metadata without exposing LÖVE audio objects to
//! `iw2wth_core`.

pub const LEGACY_DEFAULT_SOUND_VOLUME: f32 = 1.0;
pub const LEGACY_PORTAL_SOUND_VOLUME: f32 = 0.3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySoundEffect {
    Jump,
    JumpBig,
    Stomp,
    Shot,
    BlockHit,
    BlockBreak,
    Coin,
    Pipe,
    Boom,
    MushroomAppear,
    MushroomEat,
    Shrink,
    Death,
    GameOver,
    Fireball,
    OneUp,
    LevelEnd,
    CastleEnd,
    ScoreRing,
    Intermission,
    Fire,
    BridgeBreak,
    BowserFall,
    Vine,
    Swim,
    Rainboom,
    Konami,
    Pause,
    BulletBill,
    Stab,
    Portal1Open,
    Portal2Open,
    PortalEnter,
    PortalFizzle,
    LowTime,
}

impl LegacySoundEffect {
    #[must_use]
    pub const fn lua_global(self) -> &'static str {
        match self {
            Self::Jump => "jumpsound",
            Self::JumpBig => "jumpbigsound",
            Self::Stomp => "stompsound",
            Self::Shot => "shotsound",
            Self::BlockHit => "blockhitsound",
            Self::BlockBreak => "blockbreaksound",
            Self::Coin => "coinsound",
            Self::Pipe => "pipesound",
            Self::Boom => "boomsound",
            Self::MushroomAppear => "mushroomappearsound",
            Self::MushroomEat => "mushroomeatsound",
            Self::Shrink => "shrinksound",
            Self::Death => "deathsound",
            Self::GameOver => "gameoversound",
            Self::Fireball => "fireballsound",
            Self::OneUp => "oneupsound",
            Self::LevelEnd => "levelendsound",
            Self::CastleEnd => "castleendsound",
            Self::ScoreRing => "scoreringsound",
            Self::Intermission => "intermissionsound",
            Self::Fire => "firesound",
            Self::BridgeBreak => "bridgebreaksound",
            Self::BowserFall => "bowserfallsound",
            Self::Vine => "vinesound",
            Self::Swim => "swimsound",
            Self::Rainboom => "rainboomsound",
            Self::Konami => "konamisound",
            Self::Pause => "pausesound",
            Self::BulletBill => "bulletbillsound",
            Self::Stab => "stabsound",
            Self::Portal1Open => "portal1opensound",
            Self::Portal2Open => "portal2opensound",
            Self::PortalEnter => "portalentersound",
            Self::PortalFizzle => "portalfizzlesound",
            Self::LowTime => "lowtime",
        }
    }

    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Jump => "sounds/jump.ogg",
            Self::JumpBig => "sounds/jumpbig.ogg",
            Self::Stomp => "sounds/stomp.ogg",
            Self::Shot => "sounds/shot.ogg",
            Self::BlockHit => "sounds/blockhit.ogg",
            Self::BlockBreak => "sounds/blockbreak.ogg",
            Self::Coin => "sounds/coin.ogg",
            Self::Pipe => "sounds/pipe.ogg",
            Self::Boom => "sounds/boom.ogg",
            Self::MushroomAppear => "sounds/mushroomappear.ogg",
            Self::MushroomEat => "sounds/mushroomeat.ogg",
            Self::Shrink => "sounds/shrink.ogg",
            Self::Death => "sounds/death.ogg",
            Self::GameOver => "sounds/gameover.ogg",
            Self::Fireball => "sounds/fireball.ogg",
            Self::OneUp => "sounds/oneup.ogg",
            Self::LevelEnd => "sounds/levelend.ogg",
            Self::CastleEnd => "sounds/castleend.ogg",
            Self::ScoreRing => "sounds/scorering.ogg",
            Self::Intermission => "sounds/intermission.ogg",
            Self::Fire => "sounds/fire.ogg",
            Self::BridgeBreak => "sounds/bridgebreak.ogg",
            Self::BowserFall => "sounds/bowserfall.ogg",
            Self::Vine => "sounds/vine.ogg",
            Self::Swim => "sounds/swim.ogg",
            Self::Rainboom => "sounds/rainboom.ogg",
            Self::Konami => "sounds/konami.ogg",
            Self::Pause => "sounds/pause.ogg",
            Self::BulletBill => "sounds/bulletbill.ogg",
            Self::Stab => "sounds/stab.ogg",
            Self::Portal1Open => "sounds/portal1open.ogg",
            Self::Portal2Open => "sounds/portal2open.ogg",
            Self::PortalEnter => "sounds/portalenter.ogg",
            Self::PortalFizzle => "sounds/portalfizzle.ogg",
            Self::LowTime => "sounds/lowtime.ogg",
        }
    }

    #[must_use]
    pub const fn initial_volume(self) -> f32 {
        match self {
            Self::Portal1Open | Self::Portal2Open | Self::PortalEnter | Self::PortalFizzle => {
                LEGACY_PORTAL_SOUND_VOLUME
            }
            _ => LEGACY_DEFAULT_SOUND_VOLUME,
        }
    }

    #[must_use]
    pub const fn looping(self) -> bool {
        matches!(self, Self::ScoreRing)
    }

    #[must_use]
    pub const fn spec(self) -> LegacySoundEffectSpec {
        LegacySoundEffectSpec {
            effect: self,
            lua_global: self.lua_global(),
            path: self.path(),
            initial_volume: self.initial_volume(),
            looping: self.looping(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySoundEffectSpec {
    pub effect: LegacySoundEffect,
    pub lua_global: &'static str,
    pub path: &'static str,
    pub initial_volume: f32,
    pub looping: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyAudioCommand {
    StopSound(LegacySoundEffect),
    PlaySound(LegacySoundEffect),
    StopAll,
    PauseAll,
    SetMasterVolume(f32),
}

#[must_use]
pub fn legacy_play_sound_commands(
    sound_enabled: bool,
    sound: LegacySoundEffect,
) -> Vec<LegacyAudioCommand> {
    if sound_enabled {
        vec![
            LegacyAudioCommand::StopSound(sound),
            LegacyAudioCommand::PlaySound(sound),
        ]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LEGACY_DEFAULT_SOUND_VOLUME, LEGACY_PORTAL_SOUND_VOLUME, LegacyAudioCommand,
        LegacySoundEffect, legacy_play_sound_commands,
    };

    #[test]
    fn sound_effect_specs_preserve_legacy_lua_globals_and_paths() {
        assert_eq!(LegacySoundEffect::Jump.lua_global(), "jumpsound");
        assert_eq!(LegacySoundEffect::Jump.path(), "sounds/jump.ogg");
        assert_eq!(LegacySoundEffect::BlockHit.lua_global(), "blockhitsound");
        assert_eq!(LegacySoundEffect::BlockHit.path(), "sounds/blockhit.ogg");
        assert_eq!(LegacySoundEffect::Coin.lua_global(), "coinsound");
        assert_eq!(LegacySoundEffect::Coin.path(), "sounds/coin.ogg");
        assert_eq!(
            LegacySoundEffect::BulletBill.lua_global(),
            "bulletbillsound"
        );
        assert_eq!(
            LegacySoundEffect::BulletBill.path(),
            "sounds/bulletbill.ogg"
        );
        assert_eq!(LegacySoundEffect::LowTime.lua_global(), "lowtime");
        assert_eq!(LegacySoundEffect::LowTime.path(), "sounds/lowtime.ogg");
    }

    #[test]
    fn portal_sound_effects_preserve_lower_legacy_starting_volume() {
        assert_eq!(
            LegacySoundEffect::Portal1Open.initial_volume(),
            LEGACY_PORTAL_SOUND_VOLUME,
        );
        assert_eq!(
            LegacySoundEffect::Portal2Open.initial_volume(),
            LEGACY_PORTAL_SOUND_VOLUME,
        );
        assert_eq!(
            LegacySoundEffect::PortalEnter.initial_volume(),
            LEGACY_PORTAL_SOUND_VOLUME,
        );
        assert_eq!(
            LegacySoundEffect::PortalFizzle.initial_volume(),
            LEGACY_PORTAL_SOUND_VOLUME,
        );
        assert_eq!(
            LegacySoundEffect::Coin.initial_volume(),
            LEGACY_DEFAULT_SOUND_VOLUME,
        );
    }

    #[test]
    fn score_ring_is_the_looping_sound_effect_from_main_lua() {
        assert!(LegacySoundEffect::ScoreRing.looping());
        assert!(!LegacySoundEffect::Intermission.looping());
    }

    #[test]
    fn sound_spec_bundles_adapter_metadata_for_loading() {
        assert_eq!(
            LegacySoundEffect::ScoreRing.spec(),
            super::LegacySoundEffectSpec {
                effect: LegacySoundEffect::ScoreRing,
                lua_global: "scoreringsound",
                path: "sounds/scorering.ogg",
                initial_volume: LEGACY_DEFAULT_SOUND_VOLUME,
                looping: true,
            },
        );
    }

    #[test]
    fn playsound_restarts_sources_when_sound_is_enabled() {
        assert_eq!(
            legacy_play_sound_commands(true, LegacySoundEffect::BlockBreak),
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockBreak),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockBreak),
            ],
        );
    }

    #[test]
    fn playsound_is_suppressed_when_sound_is_disabled() {
        assert_eq!(
            legacy_play_sound_commands(false, LegacySoundEffect::Coin),
            Vec::new(),
        );
    }
}

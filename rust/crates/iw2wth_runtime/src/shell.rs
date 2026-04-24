//! Minimal runtime shell for one parsed legacy level.
//!
//! This module starts Phase 5 by composing the adapter boundaries already owned
//! by `iw2wth_runtime`: filesystem assets, map lookup, frame time, input,
//! audio commands, and rendering intents. It intentionally stops before taking
//! over broad gameplay behavior from the Lua/LÖVE baseline.

use std::{error::Error, f32::consts, fmt, io};

use iw2wth_core::{
    Aabb, AnimationDirection, CollisionBody, CollisionKind, Facing, HorizontalDirection,
    LegacyBlockBounceContentKind, LegacyBlockBounceContext, LegacyBlockBounceReplaySpawn,
    LegacyBlockBounceSchedule, LegacyBlockContainedRewardRevealContext,
    LegacyBlockContainedRewardRevealOutcome, LegacyBlockDebrisConstants, LegacyBlockDebrisState,
    LegacyBlockEnemyShotRequest, LegacyBlockHitSoundContext, LegacyBlockItemJumpRequest,
    LegacyBlockJumpItem, LegacyBlockJumpItemKind, LegacyBlockPortalReservation,
    LegacyBlockRevealSound, LegacyBlockSpriteset, LegacyBlockTopCoinCollectionContext,
    LegacyBlockTopCoinCollectionOutcome, LegacyBlockTopEnemy, LegacyBreakableBlockOutcome,
    LegacyCeilingTileContext, LegacyCeilingTileResponse, LegacyCoinBlockAnimationConstants,
    LegacyCoinBlockAnimationScore, LegacyCoinBlockAnimationState, LegacyCoinBlockRewardConstants,
    LegacyCoinBlockRewardContext, LegacyCoinBlockRewardKind, LegacyCoinBlockRewardOutcome,
    LegacyCoinLifeReward, LegacyCollisionActor, LegacyCollisionHandlerResult,
    LegacyCollisionTarget, LegacyEmptyBreakableBlockDestroyContext, LegacyEnemyDirection,
    LegacyEntityKind, LegacyFireballCollisionOutcome, LegacyFireballCollisionTarget,
    LegacyFireballConstants, LegacyFireballFrame, LegacyFireballState, LegacyFireballUpdate,
    LegacyFireballViewport, LegacyManyCoinsTimerEntry, LegacyMapBounds, LegacyMapTileCoord,
    LegacyPlayerCollisionSnapshot, LegacyPortalEndpoint, LegacyPortalTransitInput,
    LegacyPowerUpEntity, LegacyScrollingScoreConstants, LegacyScrollingScoreLabel,
    LegacyScrollingScorePresentation, LegacyScrollingScoreState, LegacySurfaceMovementContext,
    LegacyTileChange, LegacyWarpEntity, MappackSettings, Mari0Level, OrangeGelMovementConstants,
    ParseError, PhysicsConstants, PlayerAnimationConstants, PlayerAnimationState, PlayerBodyBounds,
    PlayerEnvironment, PlayerMovementConstants, PlayerMovementInput, PlayerMovementState,
    TileCoord, TileId, UnderwaterMovementConstants, Vec2, advance_legacy_player_animation,
    apply_legacy_floor_invisible_tile_suppression, apply_legacy_floor_landing_state,
    apply_legacy_non_invisible_ceiling_tile_response, apply_legacy_player_gravity_selection,
    apply_legacy_player_gravity_velocity, apply_legacy_player_movement_with_surface_query,
    apply_legacy_start_fall_after_vertical_move, collision_kind, in_range,
    legacy_block_bounce_schedule, legacy_block_contained_reward_reveal,
    legacy_block_enemy_shot_requests, legacy_block_hit_sound_requested,
    legacy_block_item_jump_requests, legacy_block_top_coin_collection,
    legacy_ceiling_invisible_tile_suppresses_default, legacy_coin_block_reward,
    legacy_empty_breakable_block_destroy, legacy_fireball_ceil_collision,
    legacy_fireball_floor_collision, legacy_fireball_left_collision,
    legacy_fireball_passive_collision, legacy_fireball_right_collision,
    legacy_horizontal_collision_response, legacy_many_coins_timer_state, legacy_portal_coords,
    legacy_scrolling_score_presentation, legacy_side_invisible_tile_suppresses_default,
    legacy_vertical_collision_response, prune_legacy_completed_block_bounces,
    update_legacy_block_bounce_completion, update_legacy_block_debris,
    update_legacy_coin_block_animation, update_legacy_fireball, update_legacy_many_coins_timer,
    update_legacy_scrolling_score,
};

use crate::{
    assets::{
        LegacyAssetPath, LegacyAssetSource, legacy_mappack_background_paths,
        legacy_mappack_level_path, legacy_mappack_music_path, legacy_mappack_settings_path,
        legacy_mappack_tiles_path,
    },
    audio::{LegacyAudioCommand, LegacySoundEffect, legacy_play_sound_commands},
    input::{LegacyInputSnapshot, LegacyPlayerControls, legacy_player_movement_input},
    map::{LegacyLevelMapQuery, LegacyMapQuery, LegacyMapTileMetadata, LegacyTileMetadataMapQuery},
    render::{
        LegacyColor, LegacyTileBatchDrawIntent, legacy_background_color,
        legacy_tile_batch_draw_intents,
    },
    tiles::LegacyTileMetadataTable,
    time::{LegacyFrameClock, LegacyFrameStep},
};

const LEGACY_RUNTIME_VIEWPORT_WIDTH_TILES: f32 = 25.0;
const LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE: f32 = -consts::FRAC_PI_2;
const LEGACY_RUNTIME_FIREBALL_IMAGE_PATH: &str = "graphics/SMB/fireball.png";
const LEGACY_RUNTIME_SMALL_PLAYER_IMAGE_PATH: &str = "graphics/SMB/player/marioanimations.png";
const LEGACY_RUNTIME_BIG_PLAYER_IMAGE_PATH: &str = "graphics/SMB/player/bigmarioanimations.png";
const LEGACY_RUNTIME_SMALL_PLAYER_LAYER_0_IMAGE_PATH: &str =
    "graphics/SMB/player/marioanimations0.png";
const LEGACY_RUNTIME_SMALL_PLAYER_LAYER_1_IMAGE_PATH: &str =
    "graphics/SMB/player/marioanimations1.png";
const LEGACY_RUNTIME_SMALL_PLAYER_LAYER_2_IMAGE_PATH: &str =
    "graphics/SMB/player/marioanimations2.png";
const LEGACY_RUNTIME_SMALL_PLAYER_LAYER_3_IMAGE_PATH: &str =
    "graphics/SMB/player/marioanimations3.png";
const LEGACY_RUNTIME_BIG_PLAYER_LAYER_0_IMAGE_PATH: &str =
    "graphics/SMB/player/bigmarioanimations0.png";
const LEGACY_RUNTIME_BIG_PLAYER_LAYER_1_IMAGE_PATH: &str =
    "graphics/SMB/player/bigmarioanimations1.png";
const LEGACY_RUNTIME_BIG_PLAYER_LAYER_2_IMAGE_PATH: &str =
    "graphics/SMB/player/bigmarioanimations2.png";
const LEGACY_RUNTIME_BIG_PLAYER_LAYER_3_IMAGE_PATH: &str =
    "graphics/SMB/player/bigmarioanimations3.png";
const LEGACY_RUNTIME_SMALL_STANDARD_HAT_IMAGE_PATH: &str = "graphics/SMB/hats/standard.png";
const LEGACY_RUNTIME_SMALL_TYROLEAN_HAT_IMAGE_PATH: &str = "graphics/SMB/hats/tyrolean.png";
const LEGACY_RUNTIME_SMALL_TOWERING_1_HAT_IMAGE_PATH: &str = "graphics/SMB/hats/towering1.png";
const LEGACY_RUNTIME_BIG_STANDARD_HAT_IMAGE_PATH: &str = "graphics/SMB/bighats/standard.png";
const LEGACY_RUNTIME_BIG_TYROLEAN_HAT_IMAGE_PATH: &str = "graphics/SMB/bighats/tyrolean.png";
const LEGACY_RUNTIME_BIG_TOWERING_1_HAT_IMAGE_PATH: &str = "graphics/SMB/bighats/towering1.png";
const LEGACY_RUNTIME_PORTAL_CROSSHAIR_IMAGE_PATH: &str = "graphics/SMB/portalcrosshair.png";
const LEGACY_RUNTIME_PORTAL_DOT_IMAGE_PATH: &str = "graphics/SMB/portaldot.png";
const LEGACY_RUNTIME_PORTAL_PROJECTILE_IMAGE_PATH: &str = "graphics/SMB/portalprojectile.png";
const LEGACY_RUNTIME_PORTAL_PROJECTILE_PARTICLE_IMAGE_PATH: &str =
    "graphics/SMB/portalprojectileparticle.png";
const LEGACY_RUNTIME_EMANCIPATION_GRILL_PARTICLE_IMAGE_PATH: &str =
    "graphics/SMB/emanceparticle.png";
const LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH: &str = "graphics/SMB/emanceside.png";
const LEGACY_RUNTIME_EMANCIPATION_GRILL_IMAGE_WIDTH_PX: f32 = 64.0;
const LEGACY_RUNTIME_EMANCIPATION_GRILL_LINE_COLOR: LegacyColor = LegacyColor {
    r: 0.4,
    g: 0.4,
    b: 1.0,
    a: 0.04,
};
const LEGACY_RUNTIME_DOOR_PIECE_IMAGE_PATH: &str = "graphics/SMB/doorpiece.png";
const LEGACY_RUNTIME_DOOR_CENTER_IMAGE_PATH: &str = "graphics/SMB/doorcenter.png";
const LEGACY_RUNTIME_WALL_INDICATOR_IMAGE_PATH: &str = "graphics/SMB/wallindicator.png";
const LEGACY_RUNTIME_WALL_INDICATOR_QUAD_SIZE_PX: f32 = 16.0;
const LEGACY_RUNTIME_FIRE_ANIMATION_TIME: f32 = 0.11;
const LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS: usize = 4;
const LEGACY_RUNTIME_PORTAL_DOT_TIME: f32 = 0.8;
const LEGACY_RUNTIME_PORTAL_DOT_DISTANCE_TILES: f32 = 1.2;
const LEGACY_RUNTIME_PORTAL_DOT_INNER_RADIUS_PX: f32 = 10.0;
const LEGACY_RUNTIME_PORTAL_DOT_OUTER_RADIUS_PX: f32 = 70.0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeLevelSelection {
    pub mappack: String,
    pub filename: String,
    pub world: u32,
    pub level: u32,
    pub sublevel: u32,
}

impl LegacyRuntimeLevelSelection {
    #[must_use]
    pub fn new(
        mappack: impl Into<String>,
        filename: impl Into<String>,
        world: u32,
        level: u32,
        sublevel: u32,
    ) -> Self {
        Self {
            mappack: mappack.into(),
            filename: filename.into(),
            world,
            level,
            sublevel,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeShell {
    pub selection: LegacyRuntimeLevelSelection,
    pub settings: MappackSettings,
    pub level: Mari0Level,
    pub custom_tiles: bool,
    pub custom_music_path: Option<LegacyAssetPath>,
    pub background_paths: Vec<LegacyAssetPath>,
    pub clock: LegacyFrameClock,
    pub controls: LegacyPlayerControls,
    pub sound_enabled: bool,
    pub block_bounce_queue: Vec<LegacyBlockBounceSchedule>,
    pub fireball_projectiles: Vec<LegacyFireballState>,
    pub portal_projectiles: Vec<LegacyRuntimePortalProjectileSnapshot>,
    pub emancipation_grills: Vec<LegacyRuntimeEmancipationGrillSnapshot>,
    pub doors: Vec<LegacyRuntimeDoorSnapshot>,
    pub wall_indicators: Vec<LegacyRuntimeWallIndicatorSnapshot>,
    pub coin_block_animations: Vec<LegacyCoinBlockAnimationState>,
    pub block_debris_animations: Vec<LegacyRuntimeBlockDebrisAnimationState>,
    pub scrolling_score_animations: Vec<LegacyRuntimeScrollingScoreAnimationState>,
    pub many_coins_timers: Vec<LegacyManyCoinsTimerEntry>,
    pub projected_tile_changes: LegacyRuntimeProjectedTileChangeState,
    pub projected_fireball_count: LegacyRuntimeProjectedFireballCountState,
    pub projected_fireball_projectile_collisions:
        LegacyRuntimeProjectedFireballProjectileCollisionState,
    pub projected_fireball_enemy_hits: LegacyRuntimeProjectedFireballEnemyHitState,
    pub block_portal_reservations: Vec<LegacyBlockPortalReservation>,
    pub projected_portal_state: LegacyRuntimeProjectedPortalState,
    pub projected_player_state: LegacyRuntimeProjectedPlayerState,
    pub jump_items: Vec<LegacyRuntimeBlockJumpItemSnapshot>,
    pub top_enemies: Vec<LegacyRuntimeBlockTopEnemySnapshot>,
    pub fireball_enemies: Vec<LegacyRuntimeFireballEnemySnapshot>,
    pub coin_count: u32,
    pub score_count: u32,
    pub life_count_enabled: bool,
    pub player_count: usize,
}

impl LegacyRuntimeShell {
    pub fn load(
        source: &impl LegacyAssetSource,
        selection: LegacyRuntimeLevelSelection,
        controls: LegacyPlayerControls,
    ) -> Result<Self, LegacyRuntimeLoadError> {
        let settings_path = legacy_mappack_settings_path(&selection.mappack);
        let settings_source = source
            .read_to_string(settings_path.as_str())
            .map_err(|source| LegacyRuntimeLoadError::ReadSettings {
                path: settings_path.clone(),
                source,
            })?;
        let settings = MappackSettings::parse(&settings_source);

        let level_path = legacy_mappack_level_path(&selection.mappack, &selection.filename);
        let level_source = source
            .read_to_string(level_path.as_str())
            .map_err(|source| LegacyRuntimeLoadError::ReadLevel {
                path: level_path.clone(),
                source,
            })?;
        let level = Mari0Level::parse(&level_source).map_err(|source| {
            LegacyRuntimeLoadError::ParseLevel {
                path: level_path.clone(),
                source,
            }
        })?;

        let custom_tiles = legacy_mappack_tiles_path(source, &selection.mappack).is_some();
        let custom_music_path = legacy_mappack_music_path(source, &selection.mappack);
        let background_paths = legacy_mappack_background_paths(
            source,
            &selection.mappack,
            selection.world,
            selection.level,
            selection.sublevel,
        );

        Ok(Self {
            selection,
            settings,
            level,
            custom_tiles,
            custom_music_path,
            background_paths,
            clock: LegacyFrameClock::default(),
            controls,
            sound_enabled: true,
            block_bounce_queue: Vec::new(),
            fireball_projectiles: Vec::new(),
            portal_projectiles: Vec::new(),
            emancipation_grills: Vec::new(),
            doors: Vec::new(),
            wall_indicators: Vec::new(),
            coin_block_animations: Vec::new(),
            block_debris_animations: Vec::new(),
            scrolling_score_animations: Vec::new(),
            many_coins_timers: Vec::new(),
            projected_tile_changes: LegacyRuntimeProjectedTileChangeState::default(),
            projected_fireball_count: LegacyRuntimeProjectedFireballCountState::default(),
            projected_fireball_projectile_collisions:
                LegacyRuntimeProjectedFireballProjectileCollisionState::default(),
            projected_fireball_enemy_hits: LegacyRuntimeProjectedFireballEnemyHitState::default(),
            block_portal_reservations: Vec::new(),
            projected_portal_state: LegacyRuntimeProjectedPortalState::default(),
            projected_player_state: LegacyRuntimeProjectedPlayerState::default(),
            jump_items: Vec::new(),
            top_enemies: Vec::new(),
            fireball_enemies: Vec::new(),
            coin_count: 0,
            score_count: 0,
            life_count_enabled: true,
            player_count: 1,
        })
    }

    #[must_use]
    pub const fn map_query(&self) -> LegacyLevelMapQuery<'_> {
        LegacyLevelMapQuery::new(&self.level)
    }

    #[must_use]
    pub const fn metadata_map_query<'tiles>(
        &self,
        tiles: &'tiles LegacyTileMetadataTable,
    ) -> LegacyTileMetadataMapQuery<'_, 'tiles> {
        LegacyTileMetadataMapQuery::new(&self.level, tiles)
    }

    #[must_use]
    fn projected_metadata_map_query<'tiles>(
        &self,
        tiles: &'tiles LegacyTileMetadataTable,
    ) -> LegacyRuntimeProjectedTileMetadataMapQuery<'_, 'tiles, '_> {
        LegacyRuntimeProjectedTileMetadataMapQuery::new(
            &self.level,
            tiles,
            &self.projected_tile_changes,
        )
    }

    #[must_use]
    fn report_only_block_portal_reservations(&self) -> Vec<LegacyBlockPortalReservation> {
        let mut reservations = self.block_portal_reservations.clone();
        reservations.extend(self.projected_portal_state.block_portal_reservations());
        reservations
    }

    #[must_use]
    fn report_only_block_portal_guards(&self) -> Vec<LegacyRuntimePortalBlockGuard> {
        let mut guards = self
            .block_portal_reservations
            .iter()
            .copied()
            .map(|reservation| LegacyRuntimePortalBlockGuard {
                source: LegacyRuntimePortalBlockGuardSource::ExplicitReservation,
                reservation,
            })
            .collect::<Vec<_>>();
        guards.extend(
            self.projected_portal_state
                .block_portal_reservations()
                .into_iter()
                .map(|reservation| LegacyRuntimePortalBlockGuard {
                    source: LegacyRuntimePortalBlockGuardSource::ProjectedPortalState,
                    reservation,
                }),
        );
        guards
    }

    #[must_use]
    pub fn step_frame(
        &mut self,
        raw_dt: f32,
        input: &impl LegacyInputSnapshot,
        joystick_deadzone: f32,
        render: LegacyRuntimeRenderContext,
        sound: Option<LegacySoundEffect>,
    ) -> LegacyRuntimeFrame {
        let frame_step = self.clock.step(raw_dt);
        let movement_input = legacy_player_movement_input(&self.controls, input, joystick_deadzone);
        let tile_batch_draws =
            legacy_tile_batch_draw_intents(render.xscroll, render.scale, self.custom_tiles);
        let background_color = self
            .level
            .properties
            .background
            .and_then(|background| u8::try_from(background).ok())
            .and_then(legacy_background_color);
        let audio_commands = sound
            .map(|sound| legacy_play_sound_commands(self.sound_enabled, sound))
            .unwrap_or_default();

        LegacyRuntimeFrame {
            frame_step,
            movement_input,
            background_color,
            tile_batch_draws,
            audio_commands,
        }
    }

    pub fn step_player_frame(
        &mut self,
        player: &mut LegacyRuntimePlayer,
        input: &impl LegacyInputSnapshot,
        request: LegacyRuntimeFrameRequest,
        tiles: &LegacyTileMetadataTable,
    ) -> LegacyRuntimePlayerFrame {
        let mut frame = self.step_frame(
            request.raw_dt,
            input,
            request.joystick_deadzone,
            request.render,
            request.sound,
        );
        let fireball_launch_snapshot = request.fireball_launch.map(|snapshot| {
            self.projected_fireball_count
                .apply_to_launch_snapshot(snapshot)
        });
        let fireball_launch_intent = fireball_launch_snapshot.and_then(|snapshot| {
            legacy_runtime_fireball_launch_intent(*player, snapshot, frame.frame_step.should_update)
        });
        let mut fireball_count_projections = Vec::new();
        if let Some(intent) = fireball_launch_intent {
            frame
                .audio_commands
                .extend(legacy_play_sound_commands(self.sound_enabled, intent.sound));
            self.fireball_projectiles.push(intent.spawn);
            let projection = self.projected_fireball_count.apply_launch_intent(intent);
            fireball_count_projections.push(projection);
        }
        let (
            coin_pickups,
            coin_counter_intents,
            score_counter_intents,
            collisions,
            tile_change_projections,
            breakable_block_cleanup_projections,
            fireball_projectile_progress,
            fireball_map_target_probes,
            fireball_collision_probes,
            projected_fireball_projectile_collision_snapshots,
            fireball_enemy_hit_intents,
            projected_fireball_enemy_hit_snapshots,
            block_bounce_progress,
            coin_block_animation_progress,
            block_debris_animation_progress,
            scrolling_score_animation_progress,
            many_coins_timer_progress,
        ) = if frame.frame_step.should_update {
            let fireball_projectile_progress = progress_legacy_runtime_fireball_projectiles(
                &mut self.fireball_projectiles,
                &mut self.projected_fireball_projectile_collisions,
                frame.frame_step.update_dt,
                request.render.xscroll,
            );
            fireball_count_projections.extend(project_legacy_runtime_fireball_count_from_releases(
                &mut self.projected_fireball_count,
                fireball_projectile_progress.queue_len_before_prune,
                &fireball_projectile_progress.release_summaries,
            ));
            let fireball_projectiles_for_collision_queries = self
                .projected_fireball_projectile_collisions
                .apply_to_projectiles(&self.fireball_projectiles);
            let fireball_map_target_probes = probe_legacy_runtime_fireball_map_targets(
                &fireball_projectiles_for_collision_queries,
                frame.frame_step.update_dt,
                self.projected_metadata_map_query(tiles),
            );
            let fireball_collision_probes = probe_legacy_runtime_fireball_collisions(
                &fireball_projectiles_for_collision_queries,
                request.fireball_collision_probe,
                &fireball_map_target_probes,
                &self.fireball_enemies,
                &self.projected_fireball_enemy_hits,
                frame.frame_step.update_dt,
            );
            let projected_fireball_projectile_collision_snapshots =
                project_legacy_runtime_fireball_projectile_collisions(
                    &mut self.projected_fireball_projectile_collisions,
                    &fireball_collision_probes,
                );
            let projected_fireball_enemy_hits_before_frame =
                self.projected_fireball_enemy_hits.clone();
            let fireball_enemy_hit_intents = legacy_runtime_fireball_enemy_hit_intents(
                &fireball_collision_probes,
                &self.fireball_enemies,
                &self.projected_fireball_enemy_hits,
            );
            let projected_fireball_enemy_hit_snapshots = fireball_enemy_hit_intents
                .iter()
                .copied()
                .map(|intent| self.projected_fireball_enemy_hits.apply_intent(intent))
                .collect::<Vec<_>>();
            fireball_count_projections.extend(project_legacy_runtime_fireball_count_from_releases(
                &mut self.projected_fireball_count,
                self.fireball_projectiles.len(),
                &fireball_collision_probes.release_summaries,
            ));
            let block_bounce_progress = progress_legacy_runtime_block_bounces(
                &mut self.block_bounce_queue,
                frame.frame_step.update_dt,
            );
            let mut many_coins_timer_progress = progress_legacy_runtime_many_coins_timers(
                &self.many_coins_timers,
                frame.frame_step.update_dt,
            );
            let progressed_many_coins_timers = many_coins_timer_progress
                .reports
                .iter()
                .map(|report| LegacyManyCoinsTimerEntry {
                    coord: report.coord,
                    remaining: report.remaining_after,
                })
                .collect::<Vec<_>>();
            let coin_block_animation_progress = progress_legacy_runtime_coin_block_animations(
                &mut self.coin_block_animations,
                frame.frame_step.update_dt,
                request.render.xscroll,
            );
            let block_debris_animation_progress = progress_legacy_runtime_block_debris_animations(
                &mut self.block_debris_animations,
                frame.frame_step.update_dt,
            );
            let scrolling_score_animation_progress =
                progress_legacy_runtime_scrolling_score_animations(
                    &mut self.scrolling_score_animations,
                    frame.frame_step.update_dt,
                );
            let coin_pickups = collect_legacy_player_coin_pickups(&mut self.level, tiles, *player);
            let mut report_score_count = self.score_count;
            let mut score_counter_intents = coin_block_animation_progress
                .reports
                .iter()
                .filter_map(|report| {
                    report.scrolling_score.map(|scrolling_score| {
                        legacy_runtime_score_counter_intent(
                            LegacyRuntimeScoreSource::CoinBlockAnimation {
                                source_index: report.index,
                            },
                            &mut report_score_count,
                            report
                                .score
                                .map_or(0, |score| u32::try_from(score.score_delta).unwrap_or(0)),
                            Some(scrolling_score),
                        )
                    })
                })
                .collect::<Vec<_>>();
            let mut coin_counter_intents = coin_pickups
                .iter()
                .map(|pickup| {
                    legacy_runtime_coin_counter_intent(
                        LegacyRuntimeCoinCounterSource::PlayerCoinPickup {
                            coord: pickup.coord,
                        },
                        self.coin_count,
                        self.life_count_enabled,
                        self.player_count,
                        pickup.score_delta,
                    )
                })
                .collect::<Vec<_>>();
            score_counter_intents.extend(coin_pickups.iter().map(|pickup| {
                legacy_runtime_score_counter_intent(
                    LegacyRuntimeScoreSource::PlayerCoinPickup {
                        coord: pickup.coord,
                    },
                    &mut report_score_count,
                    pickup.score_delta,
                    None,
                )
            }));
            for pickup in &coin_pickups {
                frame
                    .audio_commands
                    .extend(legacy_play_sound_commands(self.sound_enabled, pickup.sound));
            }
            for probe in &fireball_collision_probes.reports {
                if probe.outcome.play_block_hit_sound {
                    frame.audio_commands.extend(legacy_play_sound_commands(
                        self.sound_enabled,
                        LegacySoundEffect::BlockHit,
                    ));
                }
                if let Some(points) = probe.outcome.points
                    && !legacy_runtime_fireball_probe_score_suppressed_by_projected_enemy_hit(
                        *probe,
                        &self.fireball_enemies,
                        &projected_fireball_enemy_hits_before_frame,
                    )
                {
                    score_counter_intents.push(legacy_runtime_score_counter_intent(
                        LegacyRuntimeScoreSource::FireballCollisionProbe {
                            projectile_index: probe.projectile_index,
                            source: probe.source,
                            axis: probe.axis,
                            target: probe.target,
                        },
                        &mut report_score_count,
                        points,
                        Some(LegacyScrollingScoreState::spawn(
                            LegacyScrollingScoreLabel::Points(points),
                            probe.state_after.x,
                            probe.state_after.y,
                            request.render.xscroll,
                        )),
                    ));
                }
            }
            let block_portal_reservations = self.report_only_block_portal_reservations();
            let block_portal_guards = self.report_only_block_portal_guards();
            let collisions = integrate_player_over_map(
                player,
                frame.movement_input,
                frame.frame_step.update_dt,
                self.projected_metadata_map_query(tiles),
                LegacyRuntimeBlockHitReportContext {
                    spriteset: legacy_level_block_spriteset(&self.level),
                    many_coins_timers: &progressed_many_coins_timers,
                    portal_reservations: &block_portal_reservations,
                    portal_guards: &block_portal_guards,
                    jump_items: &self.jump_items,
                    top_enemies: &self.top_enemies,
                    coin_count: self.coin_count,
                    life_count_enabled: self.life_count_enabled,
                    player_count: self.player_count,
                },
            );
            for block_hit in &collisions.block_hits {
                if block_hit.play_hit_sound {
                    frame.audio_commands.extend(legacy_play_sound_commands(
                        self.sound_enabled,
                        LegacySoundEffect::BlockHit,
                    ));
                }
            }
            for destroy in &collisions.empty_breakable_block_destroys {
                if matches!(
                    &destroy.outcome,
                    LegacyBreakableBlockOutcome::Broken(effects) if effects.play_break_sound
                ) {
                    frame.audio_commands.extend(legacy_play_sound_commands(
                        self.sound_enabled,
                        LegacySoundEffect::BlockBreak,
                    ));
                }
            }
            for reward in &collisions.coin_block_rewards {
                coin_counter_intents.push(legacy_runtime_coin_counter_intent_from_reward(
                    LegacyRuntimeCoinCounterSource::CoinBlockReward {
                        coord: reward.coord,
                    },
                    self.coin_count,
                    reward.outcome.coin_count,
                    reward.outcome.life_reward,
                    reward.outcome.score_delta,
                ));
                score_counter_intents.push(legacy_runtime_score_counter_intent(
                    LegacyRuntimeScoreSource::CoinBlockReward {
                        coord: reward.coord,
                    },
                    &mut report_score_count,
                    reward.outcome.score_delta,
                    None,
                ));
                if reward.outcome.play_coin_sound {
                    frame.audio_commands.extend(legacy_play_sound_commands(
                        self.sound_enabled,
                        LegacySoundEffect::Coin,
                    ));
                }
            }
            for top_coin in &collisions.top_coin_collections {
                coin_counter_intents.push(legacy_runtime_coin_counter_intent_from_reward(
                    LegacyRuntimeCoinCounterSource::TopCoinCollection {
                        block_coord: top_coin.block_coord,
                        coin_coord: top_coin.coin_coord,
                    },
                    self.coin_count,
                    top_coin.outcome.coin_count,
                    top_coin.outcome.life_reward,
                    top_coin.outcome.score_delta,
                ));
                score_counter_intents.push(legacy_runtime_score_counter_intent(
                    LegacyRuntimeScoreSource::TopCoinCollection {
                        block_coord: top_coin.block_coord,
                        coin_coord: top_coin.coin_coord,
                    },
                    &mut report_score_count,
                    top_coin.outcome.score_delta,
                    None,
                ));
                if top_coin.outcome.play_coin_sound {
                    frame.audio_commands.extend(legacy_play_sound_commands(
                        self.sound_enabled,
                        LegacySoundEffect::Coin,
                    ));
                }
            }
            for reveal in &collisions.contained_reward_reveals {
                frame.audio_commands.extend(legacy_play_sound_commands(
                    self.sound_enabled,
                    legacy_reveal_sound_effect(reveal.outcome.sound),
                ));
            }
            for shot in &collisions.enemy_shot_requests {
                score_counter_intents.push(legacy_runtime_score_counter_intent(
                    LegacyRuntimeScoreSource::EnemyShotRequest {
                        block_coord: shot.coord,
                        enemy_index: shot.request.index,
                    },
                    &mut report_score_count,
                    shot.request.score_delta,
                    Some(LegacyScrollingScoreState::spawn(
                        LegacyScrollingScoreLabel::Points(shot.request.score_delta),
                        shot.request.score_x,
                        shot.request.score_y,
                        request.render.xscroll,
                    )),
                ));
            }
            for destroy in &collisions.empty_breakable_block_destroys {
                if let LegacyBreakableBlockOutcome::Broken(effects) = &destroy.outcome {
                    score_counter_intents.push(legacy_runtime_score_counter_intent(
                        LegacyRuntimeScoreSource::EmptyBreakableBlockDestroy {
                            coord: destroy.coord,
                        },
                        &mut report_score_count,
                        effects.score_delta,
                        None,
                    ));
                }
            }
            self.coin_block_animations.extend(
                collisions
                    .coin_block_rewards
                    .iter()
                    .map(|reward| reward.outcome.animation)
                    .chain(
                        collisions
                            .top_coin_collections
                            .iter()
                            .map(|top_coin| top_coin.outcome.animation),
                    ),
            );
            self.block_bounce_queue.extend(
                collisions
                    .block_bounce_schedules
                    .iter()
                    .map(|scheduled| scheduled.schedule),
            );
            project_legacy_runtime_many_coins_timers(
                &mut many_coins_timer_progress,
                &collisions.coin_block_rewards,
            );
            let tile_change_projections =
                project_legacy_runtime_tile_changes_from_collision_report(&collisions);
            self.projected_tile_changes
                .apply_projections(&tile_change_projections);
            let breakable_block_cleanup_projections =
                project_legacy_runtime_breakable_block_cleanup_from_collision_report(&collisions);
            self.block_debris_animations.extend(
                breakable_block_cleanup_projections
                    .iter()
                    .filter_map(legacy_runtime_block_debris_animation_from_cleanup_projection),
            );
            self.scrolling_score_animations.extend(
                score_counter_intents
                    .iter()
                    .filter_map(legacy_runtime_scrolling_score_animation_from_score_counter_intent),
            );
            (
                coin_pickups,
                coin_counter_intents,
                score_counter_intents,
                collisions,
                tile_change_projections,
                breakable_block_cleanup_projections,
                fireball_projectile_progress,
                fireball_map_target_probes,
                fireball_collision_probes,
                projected_fireball_projectile_collision_snapshots,
                fireball_enemy_hit_intents,
                projected_fireball_enemy_hit_snapshots,
                block_bounce_progress,
                coin_block_animation_progress,
                block_debris_animation_progress,
                scrolling_score_animation_progress,
                many_coins_timer_progress,
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                LegacyRuntimePlayerCollisionReport::default(),
                Vec::new(),
                Vec::new(),
                LegacyRuntimeFireballProjectileProgressReport::default(),
                LegacyRuntimeFireballMapTargetProbeReport::default(),
                LegacyRuntimeFireballCollisionProbeReport::default(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                LegacyRuntimeBlockBounceProgressReport::default(),
                LegacyRuntimeCoinBlockAnimationProgressReport::default(),
                LegacyRuntimeBlockDebrisAnimationProgressReport::default(),
                LegacyRuntimeScrollingScoreAnimationProgressReport::default(),
                LegacyRuntimeManyCoinsTimerProgressReport::default(),
            )
        };

        let portal_target_player_source =
            self.projected_player_state.portal_transit_player_source();
        let portal_target_player = self.projected_player_state.portal_transit_player(*player);
        let portal_target_probe = request.portal_aim.and_then(|aim| {
            legacy_runtime_portal_target_probe(
                portal_target_player,
                portal_target_player_source,
                aim,
                self.projected_metadata_map_query(tiles),
            )
        });
        let portal_aim_render_preview = request.portal_aim.and_then(|aim| {
            legacy_runtime_portal_aim_render_intent_preview(
                portal_target_probe,
                aim,
                request.render,
            )
        });
        let portal_outcome_intent =
            portal_target_probe.and_then(legacy_runtime_portal_outcome_intent);
        let portal_reservation_projections = portal_outcome_intent
            .and_then(legacy_runtime_portal_reservation_projection)
            .into_iter()
            .collect::<Vec<_>>();
        let mut portal_replacement_summaries = Vec::new();
        for projection in &portal_reservation_projections {
            if let Some(summary) = self
                .projected_portal_state
                .replacement_summary_for_projection(*projection)
            {
                portal_replacement_summaries.push(summary);
            }
            self.projected_portal_state.apply_projection(*projection);
        }
        let portal_pair_readiness_summary =
            self.projected_portal_state.portal_pair_readiness_summary();
        let portal_transit_player = self.projected_player_state.portal_transit_player(*player);
        let portal_transit_candidate_probe = legacy_runtime_portal_transit_candidate_probe(
            portal_transit_player,
            portal_pair_readiness_summary,
        );
        let portalcoords_preview = legacy_runtime_portalcoords_preview(
            portal_transit_player,
            portal_transit_candidate_probe,
            frame.frame_step.update_dt,
            self.projected_metadata_map_query(tiles),
        );
        let portal_transit_outcome_summary =
            legacy_runtime_portal_transit_outcome_summary(portalcoords_preview);
        let portal_transit_audio_intent =
            legacy_runtime_portal_transit_audio_intent(portal_transit_outcome_summary);
        let portal_transit_projected_player_snapshot =
            legacy_runtime_projected_player_state_snapshot(
                *player,
                portalcoords_preview,
                portal_transit_outcome_summary,
            );
        if let Some(snapshot) = portal_transit_projected_player_snapshot {
            self.projected_player_state.apply_snapshot(snapshot);
        }
        if let Some(outcome) = portal_outcome_intent {
            frame.audio_commands.extend(legacy_play_sound_commands(
                self.sound_enabled,
                outcome.sound,
            ));
        }
        if let Some(intent) = portal_transit_audio_intent {
            frame
                .audio_commands
                .extend(legacy_play_sound_commands(self.sound_enabled, intent.sound));
        }
        let fireball_render_previews = preview_legacy_runtime_fireball_render_intents(
            &self.fireball_projectiles,
            &self.projected_fireball_projectile_collisions,
            request.render,
        );
        let portal_projectile_render_previews =
            preview_legacy_runtime_portal_projectile_render_intents(
                &self.portal_projectiles,
                request.render,
            );
        let emancipation_grill_render_previews =
            preview_legacy_runtime_emancipation_grill_render_intents(
                &self.emancipation_grills,
                request.render,
            );
        let door_render_previews =
            preview_legacy_runtime_door_render_intents(&self.doors, request.render);
        let wall_indicator_render_previews = preview_legacy_runtime_wall_indicator_render_intents(
            &self.wall_indicators,
            request.render,
        );
        let player_render_preview = legacy_runtime_player_render_intent_preview(
            *player,
            request.render,
            request.player_pointing_angle,
            portal_pair_readiness_summary,
        );

        LegacyRuntimePlayerFrame {
            frame,
            player: *player,
            player_render_preview,
            portal_target_probe,
            portal_aim_render_preview,
            portal_outcome_intent,
            portal_reservation_projections,
            portal_replacement_summaries,
            projected_portal_state: self.projected_portal_state.clone(),
            portal_pair_readiness_summary,
            portal_transit_candidate_probe,
            portalcoords_preview,
            portal_transit_outcome_summary,
            portal_transit_audio_intent,
            portal_transit_projected_player_snapshot,
            projected_player_state: self.projected_player_state.clone(),
            projected_tile_change_state: self.projected_tile_changes.clone(),
            projected_fireball_count_state: self.projected_fireball_count.clone(),
            projected_fireball_projectile_collision_state: self
                .projected_fireball_projectile_collisions
                .clone(),
            fireball_count_projections,
            fireball_launch_intent,
            fireball_projectile_progress,
            fireball_map_target_probes,
            fireball_collision_probes,
            projected_fireball_projectile_collision_snapshots,
            fireball_render_previews,
            portal_projectile_render_previews,
            emancipation_grill_render_previews,
            door_render_previews,
            wall_indicator_render_previews,
            fireball_enemy_hit_intents,
            projected_fireball_enemy_hit_snapshots,
            projected_fireball_enemy_hit_state: self.projected_fireball_enemy_hits.clone(),
            collisions,
            tile_change_projections,
            breakable_block_cleanup_projections,
            coin_pickups,
            coin_counter_intents,
            score_counter_intents,
            block_bounce_progress,
            coin_block_animation_progress,
            block_debris_animation_progress,
            scrolling_score_animation_progress,
            many_coins_timer_progress,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerPowerUp {
    Small,
    Big,
    Fire,
}

impl LegacyRuntimePlayerPowerUp {
    #[must_use]
    pub const fn legacy_size(self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Big => 2,
            Self::Fire => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayer {
    pub body: PlayerBodyBounds,
    pub movement: PlayerMovementState,
    pub big_mario: bool,
    pub power_up: LegacyRuntimePlayerPowerUp,
    pub fire_animation_timer: f32,
    pub hats: [u8; LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS],
    pub hat_count: u8,
    pub draw_hat: bool,
}

impl LegacyRuntimePlayer {
    #[must_use]
    pub const fn new(body: PlayerBodyBounds, movement: PlayerMovementState) -> Self {
        Self {
            body,
            movement,
            big_mario: false,
            power_up: LegacyRuntimePlayerPowerUp::Small,
            fire_animation_timer: LEGACY_RUNTIME_FIRE_ANIMATION_TIME,
            hats: [1, 0, 0, 0],
            hat_count: 1,
            draw_hat: true,
        }
    }

    #[must_use]
    pub const fn with_big_mario(mut self, big_mario: bool) -> Self {
        self.big_mario = big_mario;
        self.power_up = if big_mario {
            LegacyRuntimePlayerPowerUp::Big
        } else {
            LegacyRuntimePlayerPowerUp::Small
        };
        self
    }

    #[must_use]
    pub const fn with_power_up(mut self, power_up: LegacyRuntimePlayerPowerUp) -> Self {
        self.power_up = power_up;
        self.big_mario = !matches!(power_up, LegacyRuntimePlayerPowerUp::Small);
        self
    }

    #[must_use]
    pub const fn with_fire_animation_timer(mut self, fire_animation_timer: f32) -> Self {
        self.fire_animation_timer = fire_animation_timer;
        self
    }

    #[must_use]
    pub const fn with_hat_slots(mut self, hats: [u8; 4], hat_count: u8) -> Self {
        self.hats = hats;
        self.hat_count = if hat_count > 4 { 4 } else { hat_count };
        self
    }

    #[must_use]
    pub const fn with_draw_hat(mut self, draw_hat: bool) -> Self {
        self.draw_hat = draw_hat;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballLaunchSnapshot {
    pub pointing_angle: f32,
    pub requested: bool,
    pub controls_enabled: bool,
    pub flower_power: bool,
    pub ducking: bool,
    pub active_fireball_count: usize,
}

impl LegacyRuntimeFireballLaunchSnapshot {
    #[must_use]
    pub const fn new(pointing_angle: f32) -> Self {
        Self {
            pointing_angle,
            requested: true,
            controls_enabled: true,
            flower_power: false,
            ducking: false,
            active_fireball_count: 0,
        }
    }

    #[must_use]
    pub const fn with_requested(mut self, requested: bool) -> Self {
        self.requested = requested;
        self
    }

    #[must_use]
    pub const fn with_controls_enabled(mut self, controls_enabled: bool) -> Self {
        self.controls_enabled = controls_enabled;
        self
    }

    #[must_use]
    pub const fn with_flower_power(mut self, flower_power: bool) -> Self {
        self.flower_power = flower_power;
        self
    }

    #[must_use]
    pub const fn with_ducking(mut self, ducking: bool) -> Self {
        self.ducking = ducking;
        self
    }

    #[must_use]
    pub const fn with_active_fireball_count(mut self, active_fireball_count: usize) -> Self {
        self.active_fireball_count = active_fireball_count;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballLaunchIntent {
    pub source_x: f32,
    pub source_y: f32,
    pub direction: LegacyEnemyDirection,
    pub spawn: LegacyFireballState,
    pub fireball_count_before: usize,
    pub fireball_count_after: usize,
    pub fire_animation_timer_reset: f32,
    pub sound: LegacySoundEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballCollisionAxis {
    Left,
    Right,
    Floor,
    Ceiling,
    Passive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballCollisionProbeSource {
    ExplicitRequest,
    MapTargetProbe {
        coord: LegacyMapTileCoord,
        tile_id: TileId,
    },
    EnemyOverlapProbe {
        enemy_index: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeFireballCollisionProbeRequest {
    pub projectile_index: usize,
    pub axis: LegacyRuntimeFireballCollisionAxis,
    pub target: LegacyFireballCollisionTarget,
}

impl LegacyRuntimeFireballCollisionProbeRequest {
    #[must_use]
    pub const fn new(
        projectile_index: usize,
        axis: LegacyRuntimeFireballCollisionAxis,
        target: LegacyFireballCollisionTarget,
    ) -> Self {
        Self {
            projectile_index,
            axis,
            target,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballCollisionProbeReport {
    pub reports: Vec<LegacyRuntimeFireballCollisionProbe>,
    pub release_summaries: Vec<LegacyRuntimeFireballProjectileReleaseSummary>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballCollisionProbe {
    pub projectile_index: usize,
    pub source: LegacyRuntimeFireballCollisionProbeSource,
    pub axis: LegacyRuntimeFireballCollisionAxis,
    pub target: LegacyFireballCollisionTarget,
    pub state_before: LegacyFireballState,
    pub state_after: LegacyFireballState,
    pub outcome: LegacyFireballCollisionOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballEnemySnapshot {
    pub target: LegacyFireballCollisionTarget,
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub has_shotted_handler: bool,
}

impl LegacyRuntimeFireballEnemySnapshot {
    #[must_use]
    pub const fn new(
        target: LegacyFireballCollisionTarget,
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        has_shotted_handler: bool,
    ) -> Self {
        Self {
            target,
            index,
            x,
            y,
            width,
            height,
            has_shotted_handler,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballEnemyHitIntent {
    pub projectile_index: usize,
    pub source: LegacyRuntimeFireballCollisionProbeSource,
    pub axis: LegacyRuntimeFireballCollisionAxis,
    pub target: LegacyFireballCollisionTarget,
    pub enemy: LegacyRuntimeFireballEnemySnapshot,
    pub shot_direction: LegacyEnemyDirection,
    pub score_delta: Option<u32>,
    pub score_x: f32,
    pub score_y: f32,
    pub live_enemy_mutated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedFireballEnemyHitSnapshot {
    pub intent: LegacyRuntimeFireballEnemyHitIntent,
    pub enemy: LegacyRuntimeFireballEnemySnapshot,
    pub active_after: bool,
    pub shot_after: bool,
    pub removed_from_future_queries: bool,
    pub live_enemy_mutated: bool,
}

impl LegacyRuntimeProjectedFireballEnemyHitSnapshot {
    #[must_use]
    pub const fn from_intent(intent: LegacyRuntimeFireballEnemyHitIntent) -> Self {
        Self {
            intent,
            enemy: intent.enemy,
            active_after: false,
            shot_after: true,
            removed_from_future_queries: true,
            live_enemy_mutated: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedFireballEnemyHitState {
    pub snapshots: Vec<LegacyRuntimeProjectedFireballEnemyHitSnapshot>,
}

impl LegacyRuntimeProjectedFireballEnemyHitState {
    pub fn apply_intent(
        &mut self,
        intent: LegacyRuntimeFireballEnemyHitIntent,
    ) -> LegacyRuntimeProjectedFireballEnemyHitSnapshot {
        let snapshot = LegacyRuntimeProjectedFireballEnemyHitSnapshot::from_intent(intent);
        self.snapshots.push(snapshot);
        snapshot
    }

    #[must_use]
    pub fn contains_removed_enemy(&self, enemy: LegacyRuntimeFireballEnemySnapshot) -> bool {
        self.snapshots.iter().rev().any(|snapshot| {
            snapshot.removed_from_future_queries
                && snapshot.enemy.target == enemy.target
                && snapshot.enemy.index == enemy.index
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballMapTargetProbeReport {
    pub reports: Vec<LegacyRuntimeFireballMapTargetProbe>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballMapTargetProbe {
    pub projectile_index: usize,
    pub state: LegacyFireballState,
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub axis: LegacyRuntimeFireballCollisionAxis,
    pub target: LegacyFireballCollisionTarget,
    pub collides: bool,
    pub invisible: bool,
    pub breakable: bool,
    pub coin_block: bool,
    pub play_block_hit_sound: bool,
    pub live_projectile_collision_mutated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballCallback {
    MarioFireballCallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeFireballCallbackMetadata {
    pub callback: LegacyRuntimeFireballCallback,
    pub fireball_count_delta: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeFireballProjectileReleaseSummary {
    pub projectile_index: usize,
    pub source: LegacyRuntimeFireballProjectileReleaseSource,
    pub callback: LegacyRuntimeFireballCallbackMetadata,
    pub live_projectile_queue_mutated: bool,
    pub live_fireball_counter_mutated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballProjectileReleaseSource {
    ProjectileUpdate,
    CollisionProbe {
        source: LegacyRuntimeFireballCollisionProbeSource,
        axis: LegacyRuntimeFireballCollisionAxis,
        target: LegacyFireballCollisionTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedFireballProjectileCollisionSnapshot {
    pub projectile_index: usize,
    pub source: LegacyRuntimeFireballCollisionProbeSource,
    pub axis: LegacyRuntimeFireballCollisionAxis,
    pub target: LegacyFireballCollisionTarget,
    pub state_before: LegacyFireballState,
    pub state_after: LegacyFireballState,
    pub removed_from_future_collision_queries: bool,
    pub live_projectile_queue_mutated: bool,
}

impl LegacyRuntimeProjectedFireballProjectileCollisionSnapshot {
    #[must_use]
    pub const fn from_probe(probe: LegacyRuntimeFireballCollisionProbe) -> Self {
        Self {
            projectile_index: probe.projectile_index,
            source: probe.source,
            axis: probe.axis,
            target: probe.target,
            state_before: probe.state_before,
            state_after: probe.state_after,
            removed_from_future_collision_queries: !probe.state_after.active
                || probe.state_after.destroy
                || probe.state_after.destroy_soon,
            live_projectile_queue_mutated: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballRenderSource {
    LiveProjectile,
    ProjectedProjectileCollision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeFireballRenderFrameKind {
    Flying,
    Explosion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeFireballRenderQuad {
    pub x_px: u32,
    pub y_px: u32,
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballRenderIntentPreview {
    pub projectile_index: usize,
    pub source: LegacyRuntimeFireballRenderSource,
    pub state: LegacyFireballState,
    pub frame: LegacyFireballFrame,
    pub frame_kind: LegacyRuntimeFireballRenderFrameKind,
    pub image_path: &'static str,
    pub quad: LegacyRuntimeFireballRenderQuad,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
    pub live_projectile_queue_mutated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballRenderIntentPreviewReport {
    pub previews: Vec<LegacyRuntimeFireballRenderIntentPreview>,
    pub suppressed_projected_removal_indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimePortalProjectileSnapshot {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
    pub time: f32,
    pub color: LegacyColor,
    pub particles: Vec<LegacyRuntimePortalProjectileParticleSnapshot>,
}

impl LegacyRuntimePortalProjectileSnapshot {
    #[must_use]
    pub fn new(x: f32, y: f32, timer: f32, time: f32, color: LegacyColor) -> Self {
        Self {
            x,
            y,
            timer,
            time,
            color,
            particles: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_particle(
        mut self,
        particle: LegacyRuntimePortalProjectileParticleSnapshot,
    ) -> Self {
        self.particles.push(particle);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalProjectileParticleSnapshot {
    pub x: f32,
    pub y: f32,
    pub color: LegacyColor,
}

impl LegacyRuntimePortalProjectileParticleSnapshot {
    #[must_use]
    pub const fn new(x: f32, y: f32, color: LegacyColor) -> Self {
        Self { x, y, color }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalProjectileHeadRenderPreview {
    pub x: f32,
    pub y: f32,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub color: LegacyColor,
    pub image_path: &'static str,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalProjectileParticleRenderPreview {
    pub particle_index: usize,
    pub x: f32,
    pub y: f32,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub color: LegacyColor,
    pub image_path: &'static str,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimePortalProjectileRenderIntentPreview {
    pub projectile_index: usize,
    pub snapshot: LegacyRuntimePortalProjectileSnapshot,
    pub particle_draws: Vec<LegacyRuntimePortalProjectileParticleRenderPreview>,
    pub head_draw: Option<LegacyRuntimePortalProjectileHeadRenderPreview>,
    pub particles_drawn_before_head: bool,
    pub color_reset_after_draw: bool,
    pub live_rendering_executed: bool,
    pub live_projectile_physics_migrated: bool,
    pub live_portal_mutated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalProjectileRenderIntentPreviewReport {
    pub previews: Vec<LegacyRuntimePortalProjectileRenderIntentPreview>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeEmancipationGrillDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeEmancipationGrillParticleDirection {
    Forward,
    Backward,
}

impl LegacyRuntimeEmancipationGrillParticleDirection {
    #[must_use]
    pub const fn from_legacy_i32(direction: i32) -> Option<Self> {
        match direction {
            1 => Some(Self::Forward),
            -1 => Some(Self::Backward),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillSnapshot {
    pub x: f32,
    pub y: f32,
    pub direction: LegacyRuntimeEmancipationGrillDirection,
    pub destroyed: bool,
    pub start: f32,
    pub end: f32,
    pub range_px: f32,
    pub particles: Vec<LegacyRuntimeEmancipationGrillParticleSnapshot>,
}

impl LegacyRuntimeEmancipationGrillSnapshot {
    #[must_use]
    pub fn horizontal(x: f32, y: f32, start_x: f32, end_x: f32, range_px: f32) -> Self {
        Self {
            x,
            y,
            direction: LegacyRuntimeEmancipationGrillDirection::Horizontal,
            destroyed: false,
            start: start_x,
            end: end_x,
            range_px,
            particles: Vec::new(),
        }
    }

    #[must_use]
    pub fn vertical(x: f32, y: f32, start_y: f32, end_y: f32, range_px: f32) -> Self {
        Self {
            x,
            y,
            direction: LegacyRuntimeEmancipationGrillDirection::Vertical,
            destroyed: false,
            start: start_y,
            end: end_y,
            range_px,
            particles: Vec::new(),
        }
    }

    #[must_use]
    pub const fn destroyed(mut self, destroyed: bool) -> Self {
        self.destroyed = destroyed;
        self
    }

    #[must_use]
    pub fn with_particle(
        mut self,
        particle: LegacyRuntimeEmancipationGrillParticleSnapshot,
    ) -> Self {
        self.particles.push(particle);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillParticleSnapshot {
    pub progress: f32,
    pub direction: LegacyRuntimeEmancipationGrillParticleDirection,
    pub modifier_px: f32,
}

impl LegacyRuntimeEmancipationGrillParticleSnapshot {
    #[must_use]
    pub const fn new(
        progress: f32,
        direction: LegacyRuntimeEmancipationGrillParticleDirection,
        modifier_px: f32,
    ) -> Self {
        Self {
            progress,
            direction,
            modifier_px,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillScissorPreview {
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillLinePreview {
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub color: LegacyColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillParticleRenderPreview {
    pub particle_index: usize,
    pub progress: f32,
    pub direction: LegacyRuntimeEmancipationGrillParticleDirection,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub image_path: &'static str,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillSideRenderPreview {
    pub side_index: usize,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub image_path: &'static str,
    pub rotation: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillRenderIntentPreview {
    pub grill_index: usize,
    pub snapshot: LegacyRuntimeEmancipationGrillSnapshot,
    pub scissor: Option<LegacyRuntimeEmancipationGrillScissorPreview>,
    pub line_rect: Option<LegacyRuntimeEmancipationGrillLinePreview>,
    pub particle_draws: Vec<LegacyRuntimeEmancipationGrillParticleRenderPreview>,
    pub side_draws: Vec<LegacyRuntimeEmancipationGrillSideRenderPreview>,
    pub scissor_cleared_after_particles: bool,
    pub color_reset_after_line: bool,
    pub live_rendering_executed: bool,
    pub live_grill_physics_migrated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeEmancipationGrillRenderIntentPreviewReport {
    pub previews: Vec<LegacyRuntimeEmancipationGrillRenderIntentPreview>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeDoorDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeDoorPartKind {
    Piece,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeDoorSnapshot {
    pub x: f32,
    pub y: f32,
    pub direction: LegacyRuntimeDoorDirection,
    pub open: bool,
    pub timer: f32,
    pub active: bool,
}

impl LegacyRuntimeDoorSnapshot {
    #[must_use]
    pub const fn horizontal(x: f32, y: f32, timer: f32) -> Self {
        Self {
            x,
            y,
            direction: LegacyRuntimeDoorDirection::Horizontal,
            open: false,
            timer,
            active: true,
        }
    }

    #[must_use]
    pub const fn vertical(x: f32, y: f32, timer: f32) -> Self {
        Self {
            x,
            y,
            direction: LegacyRuntimeDoorDirection::Vertical,
            open: false,
            timer,
            active: true,
        }
    }

    #[must_use]
    pub const fn from_legacy_horizontal_coord(cox: f32, coy: f32, timer: f32) -> Self {
        Self::horizontal(cox - 1.0, coy - 12.0 / 16.0, timer)
    }

    #[must_use]
    pub const fn from_legacy_vertical_coord(cox: f32, coy: f32, timer: f32) -> Self {
        Self::vertical(cox - 12.0 / 16.0, coy - 2.0, timer)
    }

    #[must_use]
    pub const fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeDoorRenderPartPreview {
    pub part_index: usize,
    pub kind: LegacyRuntimeDoorPartKind,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub image_path: &'static str,
    pub rotation: f32,
    pub origin_x_px: f32,
    pub origin_y_px: f32,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeDoorRenderIntentPreview {
    pub door_index: usize,
    pub snapshot: LegacyRuntimeDoorSnapshot,
    pub ymod_tiles: f32,
    pub center_rotation_delta: f32,
    pub part_draws: [LegacyRuntimeDoorRenderPartPreview; 4],
    pub live_rendering_executed: bool,
    pub live_door_physics_migrated: bool,
    pub live_door_entity_mutated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeDoorRenderIntentPreviewReport {
    pub previews: Vec<LegacyRuntimeDoorRenderIntentPreview>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeWallIndicatorSnapshot {
    pub x: f32,
    pub y: f32,
    pub lighted: bool,
}

impl LegacyRuntimeWallIndicatorSnapshot {
    #[must_use]
    pub const fn new(x: f32, y: f32, lighted: bool) -> Self {
        Self { x, y, lighted }
    }

    #[must_use]
    pub const fn from_legacy_coord(x: f32, y: f32, lighted: bool) -> Self {
        Self::new(x, y, lighted)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeWallIndicatorRenderIntentPreview {
    pub indicator_index: usize,
    pub snapshot: LegacyRuntimeWallIndicatorSnapshot,
    pub quad_index: usize,
    pub source_x_px: f32,
    pub source_y_px: f32,
    pub source_w_px: f32,
    pub source_h_px: f32,
    pub image_path: &'static str,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub color: LegacyColor,
    pub live_rendering_executed: bool,
    pub live_wall_indicator_physics_migrated: bool,
    pub live_wall_indicator_entity_mutated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeWallIndicatorRenderIntentPreviewReport {
    pub previews: Vec<LegacyRuntimeWallIndicatorRenderIntentPreview>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerRenderFrame {
    SmallRun,
    SmallIdle,
    SmallSlide,
    SmallJump,
    SmallSwim,
    SmallClimb,
    SmallDead,
    BigRun,
    BigIdle,
    BigSlide,
    BigJump,
    BigSwim,
    BigClimb,
    BigDuck,
    BigFire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePlayerRenderQuad {
    pub x_px: u32,
    pub y_px: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub atlas_width_px: u32,
    pub atlas_height_px: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderTint {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerRenderTintSource {
    PlayerColor,
    FlowerColor,
    White,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerRenderDirectionScaleSource {
    PlayerPointingAngle,
    PortalCloneAnimationDirection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderDirectionScale {
    pub source: LegacyRuntimePlayerRenderDirectionScaleSource,
    pub pointing_angle: f32,
    pub animation_facing: HorizontalDirection,
    pub direction_scale: f32,
    pub vertical_scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderColorLayerPreview {
    pub draw_order: u8,
    pub graphic_layer_index: u8,
    pub image_path: &'static str,
    pub tint: LegacyRuntimePlayerRenderTint,
    pub tint_source: LegacyRuntimePlayerRenderTintSource,
    pub quad: LegacyRuntimePlayerRenderQuad,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub direction_scale: LegacyRuntimePlayerRenderDirectionScale,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderScissorPreview {
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderPortalClonePreview {
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub entry_facing: Facing,
    pub exit_facing: Facing,
    pub entry_scissor: LegacyRuntimePlayerRenderScissorPreview,
    pub exit_scissor: LegacyRuntimePlayerRenderScissorPreview,
    pub input_body: PlayerBodyBounds,
    pub output_body: PlayerBodyBounds,
    pub input_rotation: f32,
    pub output_rotation: f32,
    pub input_animation_direction: HorizontalDirection,
    pub output_animation_direction: HorizontalDirection,
    pub animation_direction_flipped: bool,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub direction_scale: LegacyRuntimePlayerRenderDirectionScale,
    pub scissor_reset_to_current: bool,
    pub live_rendering_executed: bool,
    pub live_player_mutated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerRenderHatSize {
    Small,
    Big,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderHatPreview {
    pub drawn: bool,
    pub draw_order: u8,
    pub hat_slot_index: u8,
    pub hat_id: u8,
    pub size: LegacyRuntimePlayerRenderHatSize,
    pub image_path: &'static str,
    pub tint: LegacyRuntimePlayerRenderTint,
    pub tint_source: LegacyRuntimePlayerRenderTintSource,
    pub hat_config_x_px: i32,
    pub hat_config_y_px: i32,
    pub hat_height_px: i32,
    pub offset_x_px: i32,
    pub offset_y_px: i32,
    pub stack_y_px: i32,
    pub follows_graphic_layer_index: u8,
    pub precedes_graphic_layer_index: u8,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub origin_x_px: i32,
    pub origin_y_px: i32,
    pub rotation: f32,
    pub direction_scale: LegacyRuntimePlayerRenderDirectionScale,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderIntentPreview {
    pub player_index: usize,
    pub player: LegacyRuntimePlayer,
    pub body: PlayerBodyBounds,
    pub facing: HorizontalDirection,
    pub animation_state: PlayerAnimationState,
    pub render_frame: LegacyRuntimePlayerRenderFrame,
    pub run_frame: u8,
    pub swim_frame: u8,
    pub size: u8,
    pub power_up: LegacyRuntimePlayerPowerUp,
    pub ducking: bool,
    pub fire_animation_timer: f32,
    pub fire_animation_active: bool,
    pub image_path: &'static str,
    pub quad: LegacyRuntimePlayerRenderQuad,
    pub color_layers: [LegacyRuntimePlayerRenderColorLayerPreview; 4],
    pub hat_draw_count: u8,
    pub hat_draws: [LegacyRuntimePlayerRenderHatPreview; LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS],
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub rotation: f32,
    pub scale: f32,
    pub direction_scale: LegacyRuntimePlayerRenderDirectionScale,
    pub portal_clone: Option<LegacyRuntimePlayerRenderPortalClonePreview>,
    pub live_rendering_executed: bool,
    pub live_player_mutated: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedFireballProjectileCollisionState {
    pub snapshots: Vec<LegacyRuntimeProjectedFireballProjectileCollisionSnapshot>,
    latest_projectile_states: Vec<LegacyRuntimeProjectedFireballProjectileState>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LegacyRuntimeProjectedFireballProjectileState {
    projectile_index: usize,
    state: LegacyFireballState,
}

impl LegacyRuntimeProjectedFireballProjectileCollisionState {
    pub fn apply_probe(
        &mut self,
        probe: LegacyRuntimeFireballCollisionProbe,
    ) -> LegacyRuntimeProjectedFireballProjectileCollisionSnapshot {
        let snapshot = LegacyRuntimeProjectedFireballProjectileCollisionSnapshot::from_probe(probe);
        self.snapshots.push(snapshot);
        self.record_projected_state(snapshot.projectile_index, snapshot.state_after);
        snapshot
    }

    #[must_use]
    pub fn apply_to_projectiles(
        &self,
        projectiles: &[LegacyFireballState],
    ) -> Vec<LegacyFireballState> {
        projectiles
            .iter()
            .copied()
            .enumerate()
            .map(|(index, projectile)| self.projected_state(index).unwrap_or(projectile))
            .collect()
    }

    #[must_use]
    pub fn projected_state(&self, projectile_index: usize) -> Option<LegacyFireballState> {
        self.latest_projectile_states
            .iter()
            .rev()
            .find(|projectile| projectile.projectile_index == projectile_index)
            .map(|projectile| projectile.state)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    fn record_projected_state(&mut self, projectile_index: usize, state: LegacyFireballState) {
        if let Some(projectile) = self
            .latest_projectile_states
            .iter_mut()
            .find(|projectile| projectile.projectile_index == projectile_index)
        {
            projectile.state = state;
        } else {
            self.latest_projectile_states
                .push(LegacyRuntimeProjectedFireballProjectileState {
                    projectile_index,
                    state,
                });
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeProjectedFireballCountSource {
    LaunchIntent,
    ProjectileUpdateReleaseSummary {
        projectile_index: usize,
    },
    CollisionReleaseSummary {
        projectile_index: usize,
        collision_source: LegacyRuntimeFireballCollisionProbeSource,
        axis: LegacyRuntimeFireballCollisionAxis,
        target: LegacyFireballCollisionTarget,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeProjectedFireballCountSnapshot {
    pub source: LegacyRuntimeProjectedFireballCountSource,
    pub active_fireball_count_before: usize,
    pub fireball_count_delta: i32,
    pub active_fireball_count_after: usize,
    pub live_fireball_counter_mutated: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyRuntimeProjectedFireballCountState {
    pub latest: Option<LegacyRuntimeProjectedFireballCountSnapshot>,
}

impl LegacyRuntimeProjectedFireballCountState {
    #[must_use]
    pub const fn active_fireball_count(&self) -> Option<usize> {
        match self.latest {
            Some(snapshot) => Some(snapshot.active_fireball_count_after),
            None => None,
        }
    }

    #[must_use]
    pub const fn apply_to_launch_snapshot(
        &self,
        mut snapshot: LegacyRuntimeFireballLaunchSnapshot,
    ) -> LegacyRuntimeFireballLaunchSnapshot {
        if let Some(active_fireball_count) = self.active_fireball_count() {
            snapshot.active_fireball_count = active_fireball_count;
        }
        snapshot
    }

    pub fn apply_launch_intent(
        &mut self,
        intent: LegacyRuntimeFireballLaunchIntent,
    ) -> LegacyRuntimeProjectedFireballCountSnapshot {
        let snapshot = LegacyRuntimeProjectedFireballCountSnapshot {
            source: LegacyRuntimeProjectedFireballCountSource::LaunchIntent,
            active_fireball_count_before: intent.fireball_count_before,
            fireball_count_delta: 1,
            active_fireball_count_after: intent.fireball_count_after,
            live_fireball_counter_mutated: false,
        };
        self.latest = Some(snapshot);
        snapshot
    }

    pub fn apply_release_summary(
        &mut self,
        summary: LegacyRuntimeFireballProjectileReleaseSummary,
        fallback_active_fireball_count: usize,
    ) -> LegacyRuntimeProjectedFireballCountSnapshot {
        let active_fireball_count_before = self
            .active_fireball_count()
            .unwrap_or(fallback_active_fireball_count);
        let active_fireball_count_after = apply_legacy_runtime_fireball_count_delta(
            active_fireball_count_before,
            summary.callback.fireball_count_delta,
        );
        let source = match summary.source {
            LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate => {
                LegacyRuntimeProjectedFireballCountSource::ProjectileUpdateReleaseSummary {
                    projectile_index: summary.projectile_index,
                }
            }
            LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
                source,
                axis,
                target,
            } => LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
                projectile_index: summary.projectile_index,
                collision_source: source,
                axis,
                target,
            },
        };
        let snapshot = LegacyRuntimeProjectedFireballCountSnapshot {
            source,
            active_fireball_count_before,
            fireball_count_delta: summary.callback.fireball_count_delta,
            active_fireball_count_after,
            live_fireball_counter_mutated: summary.live_fireball_counter_mutated,
        };
        self.latest = Some(snapshot);
        snapshot
    }

    #[must_use]
    pub const fn snapshot_count(&self) -> usize {
        if self.latest.is_some() { 1 } else { 0 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalSlot {
    Portal1,
    Portal2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalAimSnapshot {
    pub pointing_angle: f32,
    pub portal_1_down: bool,
    pub portal_2_down: bool,
    pub controls_enabled: bool,
    pub portal_player_type: bool,
    pub on_vine: bool,
    pub portal_dots_timer: f32,
}

impl LegacyRuntimePortalAimSnapshot {
    #[must_use]
    pub const fn new(pointing_angle: f32) -> Self {
        Self {
            pointing_angle,
            portal_1_down: false,
            portal_2_down: false,
            controls_enabled: true,
            portal_player_type: true,
            on_vine: false,
            portal_dots_timer: 0.0,
        }
    }

    #[must_use]
    pub const fn with_portal_1_down(mut self, portal_1_down: bool) -> Self {
        self.portal_1_down = portal_1_down;
        self
    }

    #[must_use]
    pub const fn with_portal_2_down(mut self, portal_2_down: bool) -> Self {
        self.portal_2_down = portal_2_down;
        self
    }

    #[must_use]
    pub const fn with_portal_dots_timer(mut self, portal_dots_timer: f32) -> Self {
        self.portal_dots_timer = portal_dots_timer;
        self
    }

    #[must_use]
    pub const fn active(self) -> bool {
        self.controls_enabled && self.portal_player_type && !self.on_vine
    }

    #[must_use]
    pub const fn requested_slot(self) -> Option<LegacyRuntimePortalSlot> {
        if self.portal_1_down {
            Some(LegacyRuntimePortalSlot::Portal1)
        } else if self.portal_2_down {
            Some(LegacyRuntimePortalSlot::Portal2)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTraceHit {
    pub coord: LegacyMapTileCoord,
    pub side: Facing,
    pub tendency: i32,
    pub impact_x: f32,
    pub impact_y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalPlacement {
    pub coord: LegacyMapTileCoord,
    pub side: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalTargetPlayerSource {
    LivePlayer,
    ProjectedPortalTransit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTargetProbe {
    pub player_source: LegacyRuntimePortalTargetPlayerSource,
    pub source_x: f32,
    pub source_y: f32,
    pub pointing_angle: f32,
    pub requested_slot: Option<LegacyRuntimePortalSlot>,
    pub trace_hit: Option<LegacyRuntimePortalTraceHit>,
    pub placement: Option<LegacyRuntimePortalPlacement>,
}

impl LegacyRuntimePortalTargetProbe {
    #[must_use]
    pub const fn portal_possible(self) -> bool {
        self.placement.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalAimDotPreview {
    pub sequence_index: u32,
    pub phase: f32,
    pub x_px: f32,
    pub y_px: f32,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub radius_px: f32,
    pub alpha: f32,
    pub color: LegacyColor,
    pub image_path: &'static str,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalAimCrosshairPreview {
    pub x_px: f32,
    pub y_px: f32,
    pub draw_x_px: f32,
    pub draw_y_px: f32,
    pub rotation: f32,
    pub origin_x_px: i32,
    pub origin_y_px: i32,
    pub color: LegacyColor,
    pub image_path: &'static str,
    pub scale: f32,
    pub live_rendering_executed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimePortalAimRenderIntentPreview {
    pub player_source: LegacyRuntimePortalTargetPlayerSource,
    pub source_x: f32,
    pub source_y: f32,
    pub pointing_angle: f32,
    pub requested_slot: Option<LegacyRuntimePortalSlot>,
    pub trace_hit: Option<LegacyRuntimePortalTraceHit>,
    pub placement: Option<LegacyRuntimePortalPlacement>,
    pub portal_possible: bool,
    pub target_x: f32,
    pub target_y: f32,
    pub distance_tiles: f32,
    pub dots_timer: f32,
    pub dots_time: f32,
    pub dots_distance_tiles: f32,
    pub dots_inner_radius_px: f32,
    pub dots_outer_radius_px: f32,
    pub dot_draws: Vec<LegacyRuntimePortalAimDotPreview>,
    pub crosshair: Option<LegacyRuntimePortalAimCrosshairPreview>,
    pub color_reset_after_dots: bool,
    pub color_reset_after_crosshair: bool,
    pub live_rendering_executed: bool,
    pub live_portal_mutated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalOutcomeKind {
    Open,
    Fizzle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalOutcomeIntent {
    pub requested_slot: LegacyRuntimePortalSlot,
    pub kind: LegacyRuntimePortalOutcomeKind,
    pub placement: Option<LegacyRuntimePortalPlacement>,
    pub sound: LegacySoundEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalWallReservation {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl LegacyRuntimePortalWallReservation {
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalReservationProjection {
    pub requested_slot: LegacyRuntimePortalSlot,
    pub placement: LegacyRuntimePortalPlacement,
    pub tile_reservations: [LegacyMapTileCoord; 2],
    pub wall_reservations: [LegacyRuntimePortalWallReservation; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeProjectedPortal {
    pub requested_slot: LegacyRuntimePortalSlot,
    pub placement: LegacyRuntimePortalPlacement,
    pub tile_reservations: [LegacyMapTileCoord; 2],
    pub wall_reservations: [LegacyRuntimePortalWallReservation; 3],
    pub block_reservation: LegacyBlockPortalReservation,
}

impl LegacyRuntimeProjectedPortal {
    #[must_use]
    pub fn from_projection(projection: LegacyRuntimePortalReservationProjection) -> Self {
        Self {
            requested_slot: projection.requested_slot,
            placement: projection.placement,
            tile_reservations: projection.tile_reservations,
            wall_reservations: projection.wall_reservations,
            block_reservation: legacy_runtime_portal_block_reservation(projection.placement),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalBlockGuardSource {
    ExplicitReservation,
    ProjectedPortalState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalBlockGuard {
    pub source: LegacyRuntimePortalBlockGuardSource,
    pub reservation: LegacyBlockPortalReservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalReplacementSummary {
    pub requested_slot: LegacyRuntimePortalSlot,
    pub previous_slot: Option<LegacyRuntimeProjectedPortal>,
    pub replacement_slot: LegacyRuntimeProjectedPortal,
    pub preserved_other_slot: Option<LegacyRuntimeProjectedPortal>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalPairing {
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub entry: LegacyRuntimeProjectedPortal,
    pub exit: LegacyRuntimeProjectedPortal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalPairReadinessSummary {
    pub portal_1: Option<LegacyRuntimeProjectedPortal>,
    pub portal_2: Option<LegacyRuntimeProjectedPortal>,
    pub ready: bool,
    pub portal_1_to_2: Option<LegacyRuntimePortalPairing>,
    pub portal_2_to_1: Option<LegacyRuntimePortalPairing>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitCandidateProbe {
    pub center_x: f32,
    pub center_y: f32,
    pub center_coord: LegacyMapTileCoord,
    pub candidate_entry_tile: Option<LegacyMapTileCoord>,
    pub candidate_pairing: Option<LegacyRuntimePortalPairing>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalCoordsPreviewReport {
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub entry_facing: Facing,
    pub exit_facing: Facing,
    pub input_body: PlayerBodyBounds,
    pub input_speed_x: f32,
    pub input_speed_y: f32,
    pub input_rotation: f32,
    pub output_body: PlayerBodyBounds,
    pub output_speed_x: f32,
    pub output_speed_y: f32,
    pub output_rotation: f32,
    pub output_animation_direction: HorizontalDirection,
    pub exit_blocked: bool,
    pub blocked_exit_probe: Option<LegacyRuntimePortalBlockedExitProbe>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalTransitOutcomeKind {
    TeleportPreview,
    BlockedExitBouncePreview,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitOutcomeSummary {
    pub kind: LegacyRuntimePortalTransitOutcomeKind,
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub entry_facing: Facing,
    pub exit_facing: Facing,
    pub input_body: PlayerBodyBounds,
    pub output_body: PlayerBodyBounds,
    pub output_speed_x: f32,
    pub output_speed_y: f32,
    pub blocked_exit_probe: Option<LegacyRuntimePortalBlockedExitProbe>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitAudioIntent {
    pub outcome_kind: LegacyRuntimePortalTransitOutcomeKind,
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub sound: LegacySoundEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePortalBlockedExitBounceAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalBlockedExitProbe {
    pub blocking_coord: LegacyMapTileCoord,
    pub bounce_axis: LegacyRuntimePortalBlockedExitBounceAxis,
    pub bounced_speed_x: f32,
    pub bounced_speed_y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeProjectedPlayerStateSource {
    PortalTransitTeleportPreview,
    PortalTransitBlockedExitBouncePreview,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedPlayerStateSnapshot {
    pub source: LegacyRuntimeProjectedPlayerStateSource,
    pub entry_slot: LegacyRuntimePortalSlot,
    pub exit_slot: LegacyRuntimePortalSlot,
    pub entry_facing: Facing,
    pub exit_facing: Facing,
    pub body: PlayerBodyBounds,
    pub speed_x: f32,
    pub speed_y: f32,
    pub animation_direction: HorizontalDirection,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedPlayerState {
    pub latest_portal_transit: Option<LegacyRuntimeProjectedPlayerStateSnapshot>,
}

impl LegacyRuntimeProjectedPlayerState {
    pub fn apply_snapshot(&mut self, snapshot: LegacyRuntimeProjectedPlayerStateSnapshot) {
        self.latest_portal_transit = Some(snapshot);
    }

    #[must_use]
    pub const fn latest_portal_transit(&self) -> Option<LegacyRuntimeProjectedPlayerStateSnapshot> {
        self.latest_portal_transit
    }

    #[must_use]
    pub fn portal_transit_player(&self, live_player: LegacyRuntimePlayer) -> LegacyRuntimePlayer {
        let Some(snapshot) = self.latest_portal_transit else {
            return live_player;
        };

        let mut movement = live_player.movement;
        movement.speed_x = snapshot.speed_x;
        movement.speed_y = snapshot.speed_y;
        movement.animation_direction = snapshot.animation_direction;

        LegacyRuntimePlayer {
            body: snapshot.body,
            movement,
            big_mario: live_player.big_mario,
            power_up: live_player.power_up,
            fire_animation_timer: live_player.fire_animation_timer,
            hats: live_player.hats,
            hat_count: live_player.hat_count,
            draw_hat: live_player.draw_hat,
        }
    }

    #[must_use]
    pub const fn portal_transit_player_source(&self) -> LegacyRuntimePortalTargetPlayerSource {
        if self.latest_portal_transit.is_some() {
            LegacyRuntimePortalTargetPlayerSource::ProjectedPortalTransit
        } else {
            LegacyRuntimePortalTargetPlayerSource::LivePlayer
        }
    }

    #[must_use]
    pub const fn snapshot_count(&self) -> usize {
        if self.latest_portal_transit.is_some() {
            1
        } else {
            0
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedPortalState {
    pub portal_1: Option<LegacyRuntimeProjectedPortal>,
    pub portal_2: Option<LegacyRuntimeProjectedPortal>,
}

impl LegacyRuntimeProjectedPortalState {
    #[must_use]
    pub fn replacement_summary_for_projection(
        &self,
        projection: LegacyRuntimePortalReservationProjection,
    ) -> Option<LegacyRuntimePortalReplacementSummary> {
        let replacement_slot = LegacyRuntimeProjectedPortal::from_projection(projection);
        let previous_slot = self.slot(projection.requested_slot);
        if previous_slot == Some(replacement_slot) {
            return None;
        }
        let preserved_other_slot = match projection.requested_slot {
            LegacyRuntimePortalSlot::Portal1 => self.portal_2,
            LegacyRuntimePortalSlot::Portal2 => self.portal_1,
        };

        Some(LegacyRuntimePortalReplacementSummary {
            requested_slot: projection.requested_slot,
            previous_slot,
            replacement_slot,
            preserved_other_slot,
        })
    }

    pub fn apply_projection(&mut self, projection: LegacyRuntimePortalReservationProjection) {
        let projected = LegacyRuntimeProjectedPortal::from_projection(projection);
        match projection.requested_slot {
            LegacyRuntimePortalSlot::Portal1 => self.portal_1 = Some(projected),
            LegacyRuntimePortalSlot::Portal2 => self.portal_2 = Some(projected),
        }
    }

    #[must_use]
    pub const fn slot(
        &self,
        slot: LegacyRuntimePortalSlot,
    ) -> Option<LegacyRuntimeProjectedPortal> {
        match slot {
            LegacyRuntimePortalSlot::Portal1 => self.portal_1,
            LegacyRuntimePortalSlot::Portal2 => self.portal_2,
        }
    }

    #[must_use]
    pub fn block_portal_reservations(&self) -> Vec<LegacyBlockPortalReservation> {
        [self.portal_1, self.portal_2]
            .into_iter()
            .flatten()
            .map(|portal| portal.block_reservation)
            .collect()
    }

    #[must_use]
    pub fn projected_slot_count(&self) -> usize {
        [self.portal_1, self.portal_2].into_iter().flatten().count()
    }

    #[must_use]
    pub fn portal_pair_readiness_summary(&self) -> Option<LegacyRuntimePortalPairReadinessSummary> {
        if self.projected_slot_count() == 0 {
            return None;
        }

        let (portal_1_to_2, portal_2_to_1) = match (self.portal_1, self.portal_2) {
            (Some(portal_1), Some(portal_2)) => (
                Some(LegacyRuntimePortalPairing {
                    entry_slot: LegacyRuntimePortalSlot::Portal1,
                    exit_slot: LegacyRuntimePortalSlot::Portal2,
                    entry: portal_1,
                    exit: portal_2,
                }),
                Some(LegacyRuntimePortalPairing {
                    entry_slot: LegacyRuntimePortalSlot::Portal2,
                    exit_slot: LegacyRuntimePortalSlot::Portal1,
                    entry: portal_2,
                    exit: portal_1,
                }),
            ),
            _ => (None, None),
        };

        Some(LegacyRuntimePortalPairReadinessSummary {
            portal_1: self.portal_1,
            portal_2: self.portal_2,
            ready: portal_1_to_2.is_some() && portal_2_to_1.is_some(),
            portal_1_to_2,
            portal_2_to_1,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerCollisionAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePlayerTileCollision {
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub axis: LegacyRuntimePlayerCollisionAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePlayerCeilingBlockHit {
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub breakable: bool,
    pub coin_block: bool,
    pub play_hit_sound: bool,
    pub portal_guard: Option<LegacyRuntimePortalBlockGuard>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerBlockBounceSchedule {
    pub coord: LegacyMapTileCoord,
    pub schedule: LegacyBlockBounceSchedule,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockContainedRewardRevealIntent {
    pub coord: LegacyMapTileCoord,
    pub content: LegacyBlockBounceContentKind,
    pub outcome: LegacyBlockContainedRewardRevealOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCoinBlockRewardIntent {
    pub coord: LegacyMapTileCoord,
    pub outcome: LegacyCoinBlockRewardOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockTopCoinCollectionIntent {
    pub block_coord: LegacyMapTileCoord,
    pub coin_coord: LegacyMapTileCoord,
    pub outcome: LegacyBlockTopCoinCollectionOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeTileChangeSource {
    ContainedRewardReveal {
        coord: LegacyMapTileCoord,
        content: LegacyBlockBounceContentKind,
    },
    CoinBlockReward {
        coord: LegacyMapTileCoord,
    },
    TopCoinCollection {
        block_coord: LegacyMapTileCoord,
        coin_coord: LegacyMapTileCoord,
    },
    EmptyBreakableBlockDestroy {
        coord: LegacyMapTileCoord,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeTileChangeProjection {
    pub source: LegacyRuntimeTileChangeSource,
    pub tile_change: LegacyTileChange,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyRuntimeProjectedTileChangeState {
    pub projections: Vec<LegacyRuntimeTileChangeProjection>,
}

impl LegacyRuntimeProjectedTileChangeState {
    pub fn apply_projection(&mut self, projection: LegacyRuntimeTileChangeProjection) {
        self.projections.push(projection);
    }

    pub fn apply_projections(&mut self, projections: &[LegacyRuntimeTileChangeProjection]) {
        self.projections.extend(projections.iter().copied());
    }

    #[must_use]
    pub fn projected_tile_id_at(&self, coord: LegacyMapTileCoord) -> Option<TileId> {
        self.projections
            .iter()
            .rev()
            .find(|projection| {
                let projected_coord = legacy_tile_coord_to_map_coord(projection.tile_change.coord);
                projected_coord == coord
            })
            .map(|projection| projection.tile_change.tile)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.projections.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }
}

#[derive(Clone, Copy, Debug)]
struct LegacyRuntimeProjectedTileMetadataMapQuery<'level, 'tiles, 'changes> {
    base: LegacyTileMetadataMapQuery<'level, 'tiles>,
    tiles: &'tiles LegacyTileMetadataTable,
    projected: &'changes LegacyRuntimeProjectedTileChangeState,
}

impl<'level, 'tiles, 'changes>
    LegacyRuntimeProjectedTileMetadataMapQuery<'level, 'tiles, 'changes>
{
    #[must_use]
    pub const fn new(
        level: &'level Mari0Level,
        tiles: &'tiles LegacyTileMetadataTable,
        projected: &'changes LegacyRuntimeProjectedTileChangeState,
    ) -> Self {
        Self {
            base: LegacyTileMetadataMapQuery::new(level, tiles),
            tiles,
            projected,
        }
    }

    #[must_use]
    pub fn tile_metadata_at(self, coord: LegacyMapTileCoord) -> Option<LegacyMapTileMetadata> {
        let tile_id = self.tile_id_at(coord)?;
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
    pub fn legacy_entity_kind_at(self, coord: LegacyMapTileCoord) -> Option<LegacyEntityKind> {
        self.base.legacy_entity_kind_at(coord)
    }
}

impl LegacyMapQuery for LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_> {
    fn bounds(&self) -> LegacyMapBounds {
        self.base.bounds()
    }

    fn tile_id_at(&self, coord: LegacyMapTileCoord) -> Option<TileId> {
        if !self.base.contains(coord) {
            return None;
        }

        self.projected
            .projected_tile_id_at(coord)
            .or_else(|| self.base.tile_id_at(coord))
    }

    fn top_gel_at(&self, coord: LegacyMapTileCoord) -> Option<iw2wth_core::LegacyGelKind> {
        self.base.top_gel_at(coord)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeBreakableBlockCleanupSource {
    EmptyBreakableBlockDestroy { coord: LegacyMapTileCoord },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyRuntimeBreakableBlockCleanupAction {
    RemoveTileCollisionObject,
    ClearGels,
    SpawnDebris {
        index: usize,
        debris: LegacyBlockDebrisState,
    },
    RegenerateSpriteBatch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBreakableBlockCleanupProjection {
    pub source: LegacyRuntimeBreakableBlockCleanupSource,
    pub action: LegacyRuntimeBreakableBlockCleanupAction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockDebrisAnimationState {
    pub source: LegacyRuntimeBreakableBlockCleanupSource,
    pub debris_index: usize,
    pub debris: LegacyBlockDebrisState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScrollingScoreAnimationState {
    pub source: LegacyRuntimeScoreSource,
    pub score: LegacyScrollingScoreState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeCoinCounterSource {
    PlayerCoinPickup {
        coord: LegacyMapTileCoord,
    },
    CoinBlockReward {
        coord: LegacyMapTileCoord,
    },
    TopCoinCollection {
        block_coord: LegacyMapTileCoord,
        coin_coord: LegacyMapTileCoord,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeCoinCounterIntent {
    pub source: LegacyRuntimeCoinCounterSource,
    pub coin_count_before: u32,
    pub coin_count_after: u32,
    pub life_reward: Option<LegacyCoinLifeReward>,
    pub score_delta: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimeScoreSource {
    PlayerCoinPickup {
        coord: LegacyMapTileCoord,
    },
    CoinBlockReward {
        coord: LegacyMapTileCoord,
    },
    TopCoinCollection {
        block_coord: LegacyMapTileCoord,
        coin_coord: LegacyMapTileCoord,
    },
    EnemyShotRequest {
        block_coord: LegacyMapTileCoord,
        enemy_index: usize,
    },
    EmptyBreakableBlockDestroy {
        coord: LegacyMapTileCoord,
    },
    CoinBlockAnimation {
        source_index: usize,
    },
    FireballCollisionProbe {
        projectile_index: usize,
        source: LegacyRuntimeFireballCollisionProbeSource,
        axis: LegacyRuntimeFireballCollisionAxis,
        target: LegacyFireballCollisionTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScoreCounterIntent {
    pub source: LegacyRuntimeScoreSource,
    pub score_count_before: u32,
    pub score_delta: u32,
    pub score_count_after: u32,
    pub scrolling_score: Option<LegacyScrollingScoreState>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockJumpItemSnapshot {
    pub kind: LegacyBlockJumpItemKind,
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub has_jump_handler: bool,
}

impl LegacyRuntimeBlockJumpItemSnapshot {
    #[must_use]
    pub const fn new(
        kind: LegacyBlockJumpItemKind,
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        has_jump_handler: bool,
    ) -> Self {
        Self {
            kind,
            index,
            x,
            y,
            width,
            height,
            has_jump_handler,
        }
    }

    #[must_use]
    pub const fn to_core_item(self) -> LegacyBlockJumpItem {
        LegacyBlockJumpItem {
            kind: self.kind,
            index: self.index,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            has_jump_handler: self.has_jump_handler,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockItemJumpIntent {
    pub coord: LegacyMapTileCoord,
    pub request: LegacyBlockItemJumpRequest,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockTopEnemySnapshot {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub has_shotted_handler: bool,
}

impl LegacyRuntimeBlockTopEnemySnapshot {
    #[must_use]
    pub const fn new(
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        has_shotted_handler: bool,
    ) -> Self {
        Self {
            index,
            x,
            y,
            width,
            height,
            has_shotted_handler,
        }
    }

    #[must_use]
    pub const fn to_core_enemy(self) -> LegacyBlockTopEnemy {
        LegacyBlockTopEnemy {
            index: self.index,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            has_shotted_handler: self.has_shotted_handler,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockEnemyShotIntent {
    pub coord: LegacyMapTileCoord,
    pub request: LegacyBlockEnemyShotRequest,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeEmptyBreakableBlockDestroyIntent {
    pub coord: LegacyMapTileCoord,
    pub outcome: LegacyBreakableBlockOutcome,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimePlayerCollisionReport {
    pub horizontal: Option<LegacyRuntimePlayerTileCollision>,
    pub vertical: Option<LegacyRuntimePlayerTileCollision>,
    pub block_hits: Vec<LegacyRuntimePlayerCeilingBlockHit>,
    pub block_bounce_schedules: Vec<LegacyRuntimePlayerBlockBounceSchedule>,
    pub coin_block_rewards: Vec<LegacyRuntimeCoinBlockRewardIntent>,
    pub top_coin_collections: Vec<LegacyRuntimeBlockTopCoinCollectionIntent>,
    pub item_jump_requests: Vec<LegacyRuntimeBlockItemJumpIntent>,
    pub enemy_shot_requests: Vec<LegacyRuntimeBlockEnemyShotIntent>,
    pub empty_breakable_block_destroys: Vec<LegacyRuntimeEmptyBreakableBlockDestroyIntent>,
    pub contained_reward_reveals: Vec<LegacyRuntimeBlockContainedRewardRevealIntent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimePlayerFrame {
    pub frame: LegacyRuntimeFrame,
    pub player: LegacyRuntimePlayer,
    pub player_render_preview: LegacyRuntimePlayerRenderIntentPreview,
    pub portal_target_probe: Option<LegacyRuntimePortalTargetProbe>,
    pub portal_aim_render_preview: Option<LegacyRuntimePortalAimRenderIntentPreview>,
    pub portal_outcome_intent: Option<LegacyRuntimePortalOutcomeIntent>,
    pub portal_reservation_projections: Vec<LegacyRuntimePortalReservationProjection>,
    pub portal_replacement_summaries: Vec<LegacyRuntimePortalReplacementSummary>,
    pub projected_portal_state: LegacyRuntimeProjectedPortalState,
    pub portal_pair_readiness_summary: Option<LegacyRuntimePortalPairReadinessSummary>,
    pub portal_transit_candidate_probe: Option<LegacyRuntimePortalTransitCandidateProbe>,
    pub portalcoords_preview: Option<LegacyRuntimePortalCoordsPreviewReport>,
    pub portal_transit_outcome_summary: Option<LegacyRuntimePortalTransitOutcomeSummary>,
    pub portal_transit_audio_intent: Option<LegacyRuntimePortalTransitAudioIntent>,
    pub portal_transit_projected_player_snapshot: Option<LegacyRuntimeProjectedPlayerStateSnapshot>,
    pub projected_player_state: LegacyRuntimeProjectedPlayerState,
    pub projected_tile_change_state: LegacyRuntimeProjectedTileChangeState,
    pub projected_fireball_count_state: LegacyRuntimeProjectedFireballCountState,
    pub projected_fireball_projectile_collision_state:
        LegacyRuntimeProjectedFireballProjectileCollisionState,
    pub fireball_count_projections: Vec<LegacyRuntimeProjectedFireballCountSnapshot>,
    pub fireball_launch_intent: Option<LegacyRuntimeFireballLaunchIntent>,
    pub fireball_projectile_progress: LegacyRuntimeFireballProjectileProgressReport,
    pub fireball_map_target_probes: LegacyRuntimeFireballMapTargetProbeReport,
    pub fireball_collision_probes: LegacyRuntimeFireballCollisionProbeReport,
    pub projected_fireball_projectile_collision_snapshots:
        Vec<LegacyRuntimeProjectedFireballProjectileCollisionSnapshot>,
    pub fireball_render_previews: LegacyRuntimeFireballRenderIntentPreviewReport,
    pub portal_projectile_render_previews: LegacyRuntimePortalProjectileRenderIntentPreviewReport,
    pub emancipation_grill_render_previews: LegacyRuntimeEmancipationGrillRenderIntentPreviewReport,
    pub door_render_previews: LegacyRuntimeDoorRenderIntentPreviewReport,
    pub wall_indicator_render_previews: LegacyRuntimeWallIndicatorRenderIntentPreviewReport,
    pub fireball_enemy_hit_intents: Vec<LegacyRuntimeFireballEnemyHitIntent>,
    pub projected_fireball_enemy_hit_snapshots: Vec<LegacyRuntimeProjectedFireballEnemyHitSnapshot>,
    pub projected_fireball_enemy_hit_state: LegacyRuntimeProjectedFireballEnemyHitState,
    pub collisions: LegacyRuntimePlayerCollisionReport,
    pub tile_change_projections: Vec<LegacyRuntimeTileChangeProjection>,
    pub breakable_block_cleanup_projections: Vec<LegacyRuntimeBreakableBlockCleanupProjection>,
    pub coin_pickups: Vec<LegacyRuntimePlayerCoinPickup>,
    pub coin_counter_intents: Vec<LegacyRuntimeCoinCounterIntent>,
    pub score_counter_intents: Vec<LegacyRuntimeScoreCounterIntent>,
    pub block_bounce_progress: LegacyRuntimeBlockBounceProgressReport,
    pub coin_block_animation_progress: LegacyRuntimeCoinBlockAnimationProgressReport,
    pub block_debris_animation_progress: LegacyRuntimeBlockDebrisAnimationProgressReport,
    pub scrolling_score_animation_progress: LegacyRuntimeScrollingScoreAnimationProgressReport,
    pub many_coins_timer_progress: LegacyRuntimeManyCoinsTimerProgressReport,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballProjectileProgressReport {
    pub reports: Vec<LegacyRuntimeFireballProjectileUpdateReport>,
    pub release_summaries: Vec<LegacyRuntimeFireballProjectileReleaseSummary>,
    pub queue_len_before_prune: usize,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballProjectileUpdateReport {
    pub index: usize,
    pub state_before: LegacyFireballState,
    pub state_after: LegacyFireballState,
    pub update: LegacyFireballUpdate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeBlockBounceProgressReport {
    pub completions: Vec<LegacyRuntimeBlockBounceCompletionReport>,
    pub item_spawn_intents: Vec<LegacyRuntimeBlockBounceItemSpawnIntent>,
    pub regenerate_sprite_batch: bool,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockBounceCompletionReport {
    pub index: usize,
    pub coord: TileCoord,
    pub timer: f32,
    pub remove: bool,
    pub suppressed_replay_spawn: Option<LegacyBlockBounceReplaySpawn>,
    pub item_spawn_intent: Option<LegacyRuntimeBlockBounceItemSpawnIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockBounceItemSpawnIntent {
    pub source_index: usize,
    pub source_coord: TileCoord,
    pub spawn: LegacyBlockBounceReplaySpawn,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeCoinBlockAnimationProgressReport {
    pub reports: Vec<LegacyRuntimeCoinBlockAnimationUpdateReport>,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCoinBlockAnimationUpdateReport {
    pub index: usize,
    pub state: LegacyCoinBlockAnimationState,
    pub remove: bool,
    pub score: Option<LegacyCoinBlockAnimationScore>,
    pub scrolling_score: Option<LegacyScrollingScoreState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeBlockDebrisAnimationProgressReport {
    pub reports: Vec<LegacyRuntimeBlockDebrisAnimationUpdateReport>,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockDebrisAnimationUpdateReport {
    pub index: usize,
    pub source: LegacyRuntimeBreakableBlockCleanupSource,
    pub debris_index: usize,
    pub state: LegacyBlockDebrisState,
    pub remove: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeScrollingScoreAnimationProgressReport {
    pub reports: Vec<LegacyRuntimeScrollingScoreAnimationUpdateReport>,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScrollingScoreAnimationUpdateReport {
    pub index: usize,
    pub source: LegacyRuntimeScoreSource,
    pub state: LegacyScrollingScoreState,
    pub presentation: LegacyScrollingScorePresentation,
    pub remove: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerProgressReport {
    pub reports: Vec<LegacyRuntimeManyCoinsTimerUpdateReport>,
    pub starts: Vec<LegacyRuntimeManyCoinsTimerStartReport>,
    pub projected_timers: Vec<LegacyManyCoinsTimerEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerUpdateReport {
    pub index: usize,
    pub coord: TileCoord,
    pub remaining_before: f32,
    pub remaining_after: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerStartReport {
    pub reward_index: usize,
    pub coord: TileCoord,
    pub duration: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LegacyRuntimeBlockHitReportContext<'a> {
    spriteset: LegacyBlockSpriteset,
    many_coins_timers: &'a [LegacyManyCoinsTimerEntry],
    portal_reservations: &'a [LegacyBlockPortalReservation],
    portal_guards: &'a [LegacyRuntimePortalBlockGuard],
    jump_items: &'a [LegacyRuntimeBlockJumpItemSnapshot],
    top_enemies: &'a [LegacyRuntimeBlockTopEnemySnapshot],
    coin_count: u32,
    life_count_enabled: bool,
    player_count: usize,
}

fn progress_legacy_runtime_block_bounces(
    queue: &mut Vec<LegacyBlockBounceSchedule>,
    dt: f32,
) -> LegacyRuntimeBlockBounceProgressReport {
    let mut completions = Vec::new();
    let mut completed_indices = Vec::new();
    let mut item_spawn_intents = Vec::new();

    for (index, schedule) in queue.iter_mut().enumerate() {
        let update = update_legacy_block_bounce_completion(schedule, dt);
        if update.remove {
            completed_indices.push(index);
        }
        let item_spawn_intent =
            update
                .replay_spawn
                .map(|spawn| LegacyRuntimeBlockBounceItemSpawnIntent {
                    source_index: index,
                    source_coord: schedule.coord,
                    spawn,
                });
        if let Some(intent) = item_spawn_intent {
            item_spawn_intents.push(intent);
        }
        completions.push(LegacyRuntimeBlockBounceCompletionReport {
            index,
            coord: schedule.coord,
            timer: schedule.timer,
            remove: update.remove,
            suppressed_replay_spawn: update.replay_spawn,
            item_spawn_intent,
        });
    }

    let prune = prune_legacy_completed_block_bounces(queue, &completed_indices);

    LegacyRuntimeBlockBounceProgressReport {
        completions,
        item_spawn_intents,
        regenerate_sprite_batch: prune.regenerate_sprite_batch,
        queue_len_after_prune: queue.len(),
    }
}

fn progress_legacy_runtime_coin_block_animations(
    queue: &mut Vec<LegacyCoinBlockAnimationState>,
    dt: f32,
    xscroll: f32,
) -> LegacyRuntimeCoinBlockAnimationProgressReport {
    let mut reports = Vec::new();
    let mut completed_indices = Vec::new();

    for (index, animation) in queue.iter_mut().enumerate() {
        let update = update_legacy_coin_block_animation(
            animation,
            LegacyCoinBlockAnimationConstants::default(),
            dt,
        );
        if update.remove {
            completed_indices.push(index);
        }
        let scrolling_score = update.score.map(|score| {
            LegacyScrollingScoreState::spawn(
                LegacyScrollingScoreLabel::Points(score.floating_score),
                score.x,
                score.y,
                xscroll,
            )
        });
        reports.push(LegacyRuntimeCoinBlockAnimationUpdateReport {
            index,
            state: *animation,
            remove: update.remove,
            score: update.score,
            scrolling_score,
        });
    }

    for index in completed_indices.into_iter().rev() {
        queue.remove(index);
    }

    LegacyRuntimeCoinBlockAnimationProgressReport {
        reports,
        queue_len_after_prune: queue.len(),
    }
}

fn progress_legacy_runtime_block_debris_animations(
    queue: &mut Vec<LegacyRuntimeBlockDebrisAnimationState>,
    dt: f32,
) -> LegacyRuntimeBlockDebrisAnimationProgressReport {
    let mut reports = Vec::new();
    let mut completed_indices = Vec::new();

    for (index, animation) in queue.iter_mut().enumerate() {
        let update = update_legacy_block_debris(
            &mut animation.debris,
            LegacyBlockDebrisConstants::default(),
            dt,
        );
        if update.remove {
            completed_indices.push(index);
        }
        reports.push(LegacyRuntimeBlockDebrisAnimationUpdateReport {
            index,
            source: animation.source,
            debris_index: animation.debris_index,
            state: animation.debris,
            remove: update.remove,
        });
    }

    for index in completed_indices.into_iter().rev() {
        queue.remove(index);
    }

    LegacyRuntimeBlockDebrisAnimationProgressReport {
        reports,
        queue_len_after_prune: queue.len(),
    }
}

fn progress_legacy_runtime_scrolling_score_animations(
    queue: &mut Vec<LegacyRuntimeScrollingScoreAnimationState>,
    dt: f32,
) -> LegacyRuntimeScrollingScoreAnimationProgressReport {
    let constants = LegacyScrollingScoreConstants::default();
    let mut reports = Vec::new();
    let mut completed_indices = Vec::new();

    for (index, animation) in queue.iter_mut().enumerate() {
        let update = update_legacy_scrolling_score(&mut animation.score, constants, dt);
        if update.remove {
            completed_indices.push(index);
        }
        reports.push(LegacyRuntimeScrollingScoreAnimationUpdateReport {
            index,
            source: animation.source,
            state: animation.score,
            presentation: legacy_scrolling_score_presentation(&animation.score, constants),
            remove: update.remove,
        });
    }

    for index in completed_indices.into_iter().rev() {
        queue.remove(index);
    }

    LegacyRuntimeScrollingScoreAnimationProgressReport {
        reports,
        queue_len_after_prune: queue.len(),
    }
}

fn progress_legacy_runtime_many_coins_timers(
    timers: &[LegacyManyCoinsTimerEntry],
    dt: f32,
) -> LegacyRuntimeManyCoinsTimerProgressReport {
    let reports = timers
        .iter()
        .enumerate()
        .map(|(index, timer)| LegacyRuntimeManyCoinsTimerUpdateReport {
            index,
            coord: timer.coord,
            remaining_before: timer.remaining,
            remaining_after: update_legacy_many_coins_timer(timer.remaining, dt),
        })
        .collect::<Vec<_>>();
    let projected_timers = reports
        .iter()
        .map(|report| LegacyManyCoinsTimerEntry {
            coord: report.coord,
            remaining: report.remaining_after,
        })
        .collect();

    LegacyRuntimeManyCoinsTimerProgressReport {
        reports,
        starts: Vec::new(),
        projected_timers,
    }
}

fn project_legacy_runtime_many_coins_timers(
    progress: &mut LegacyRuntimeManyCoinsTimerProgressReport,
    rewards: &[LegacyRuntimeCoinBlockRewardIntent],
) {
    for (reward_index, reward) in rewards.iter().enumerate() {
        if let Some(spawn) = reward.outcome.start_many_coins_timer {
            progress
                .starts
                .push(LegacyRuntimeManyCoinsTimerStartReport {
                    reward_index,
                    coord: spawn.coord,
                    duration: spawn.duration,
                });
            progress.projected_timers.push(LegacyManyCoinsTimerEntry {
                coord: spawn.coord,
                remaining: spawn.duration,
            });
        }
    }
}

fn project_legacy_runtime_tile_changes_from_collision_report(
    collisions: &LegacyRuntimePlayerCollisionReport,
) -> Vec<LegacyRuntimeTileChangeProjection> {
    let mut projections = Vec::new();

    for reveal in &collisions.contained_reward_reveals {
        projections.push(LegacyRuntimeTileChangeProjection {
            source: LegacyRuntimeTileChangeSource::ContainedRewardReveal {
                coord: reveal.coord,
                content: reveal.content,
            },
            tile_change: reveal.outcome.tile_change,
        });
    }

    for reward in &collisions.coin_block_rewards {
        if let Some(tile_change) = reward.outcome.tile_change {
            projections.push(LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                    coord: reward.coord,
                },
                tile_change,
            });
        }
    }

    for top_coin in &collisions.top_coin_collections {
        projections.push(LegacyRuntimeTileChangeProjection {
            source: LegacyRuntimeTileChangeSource::TopCoinCollection {
                block_coord: top_coin.block_coord,
                coin_coord: top_coin.coin_coord,
            },
            tile_change: top_coin.outcome.tile_change,
        });
    }

    for destroy in &collisions.empty_breakable_block_destroys {
        if let LegacyBreakableBlockOutcome::Broken(effects) = &destroy.outcome {
            projections.push(LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::EmptyBreakableBlockDestroy {
                    coord: destroy.coord,
                },
                tile_change: effects.tile_change,
            });
        }
    }

    projections
}

fn project_legacy_runtime_breakable_block_cleanup_from_collision_report(
    collisions: &LegacyRuntimePlayerCollisionReport,
) -> Vec<LegacyRuntimeBreakableBlockCleanupProjection> {
    let mut projections = Vec::new();

    for destroy in &collisions.empty_breakable_block_destroys {
        let LegacyBreakableBlockOutcome::Broken(effects) = &destroy.outcome else {
            continue;
        };
        let source = LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
            coord: destroy.coord,
        };

        if effects.remove_tile_collision_object {
            projections.push(LegacyRuntimeBreakableBlockCleanupProjection {
                source,
                action: LegacyRuntimeBreakableBlockCleanupAction::RemoveTileCollisionObject,
            });
        }
        if effects.clear_gels {
            projections.push(LegacyRuntimeBreakableBlockCleanupProjection {
                source,
                action: LegacyRuntimeBreakableBlockCleanupAction::ClearGels,
            });
        }
        projections.extend(effects.debris.iter().enumerate().map(|(index, debris)| {
            LegacyRuntimeBreakableBlockCleanupProjection {
                source,
                action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                    index,
                    debris: *debris,
                },
            }
        }));
        if effects.regenerate_sprite_batch {
            projections.push(LegacyRuntimeBreakableBlockCleanupProjection {
                source,
                action: LegacyRuntimeBreakableBlockCleanupAction::RegenerateSpriteBatch,
            });
        }
    }

    projections
}

fn legacy_runtime_block_debris_animation_from_cleanup_projection(
    projection: &LegacyRuntimeBreakableBlockCleanupProjection,
) -> Option<LegacyRuntimeBlockDebrisAnimationState> {
    let LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris { index, debris } = projection.action
    else {
        return None;
    };

    Some(LegacyRuntimeBlockDebrisAnimationState {
        source: projection.source,
        debris_index: index,
        debris,
    })
}

fn integrate_player_over_map(
    player: &mut LegacyRuntimePlayer,
    input: PlayerMovementInput,
    dt: f32,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
    report_context: LegacyRuntimeBlockHitReportContext<'_>,
) -> LegacyRuntimePlayerCollisionReport {
    apply_legacy_player_movement_with_surface_query(
        &mut player.movement,
        input,
        dt,
        LegacySurfaceMovementContext::new(
            PlayerMovementConstants::default(),
            OrangeGelMovementConstants::default(),
            player.body,
            map.bounds(),
        ),
        |coord| map.top_gel_at(coord),
    );
    advance_legacy_player_animation(
        &mut player.movement,
        dt,
        PlayerAnimationConstants::default(),
    );
    apply_legacy_player_gravity_selection(
        &mut player.movement,
        PlayerEnvironment::Normal,
        PhysicsConstants::default(),
        UnderwaterMovementConstants::default(),
    );
    apply_legacy_player_gravity_velocity(&mut player.movement, dt, PhysicsConstants::default());

    let mut report = LegacyRuntimePlayerCollisionReport::default();
    let mut vertical_blocked = false;
    let mut horizontal_blocked = false;
    let bounds = map.bounds();
    let hitter_size = if player.big_mario { 2 } else { 1 };

    for coord in legacy_tile_collision_candidates(player.body, &player.movement, dt, bounds) {
        let Some(tile) = map.tile_metadata_at(coord) else {
            continue;
        };
        if !tile.collides() {
            continue;
        }

        let moving = CollisionBody::new(
            player.body.x,
            player.body.y,
            player.body.width,
            player.body.height,
            player.movement.speed_x,
            player.movement.speed_y,
        )
        .with_gravity(player.movement.gravity);
        let target = legacy_runtime_tile_bounds(coord);

        match collision_kind(moving, target, dt, false) {
            CollisionKind::Vertical if !vertical_blocked => {
                if moving.velocity.y >= 0.0 && tile.invisible() {
                    let snapshot = LegacyPlayerCollisionSnapshot::from_state(&player.movement);
                    apply_legacy_floor_landing_state(&mut player.movement);
                    apply_legacy_floor_invisible_tile_suppression(
                        &mut player.movement,
                        snapshot,
                        true,
                    );
                    continue;
                }
                if moving.velocity.y < 0.0
                    && legacy_ceiling_invisible_tile_suppresses_default(
                        player.body,
                        player.movement,
                        legacy_player_body_from_aabb(target),
                        tile.invisible(),
                    )
                {
                    continue;
                }

                let vertical_response = legacy_vertical_collision_response(
                    legacy_player_collision_actor(player),
                    LegacyCollisionTarget::new(
                        target.min.x,
                        target.min.y,
                        target.width(),
                        target.height(),
                        Some(Vec2::ZERO),
                    ),
                    LegacyCollisionHandlerResult::ApplyDefault,
                    LegacyCollisionHandlerResult::ApplyDefault,
                );

                if !vertical_response.resolved {
                    continue;
                }

                if moving.velocity.y < 0.0 && !tile.invisible() {
                    match apply_legacy_non_invisible_ceiling_tile_response(
                        &mut player.movement,
                        &mut player.body,
                        LegacyCeilingTileContext {
                            tile: coord,
                            big_mario: player.big_mario,
                            map_width: bounds.width,
                            left_neighbor_solid: map
                                .tile_collides_at(LegacyMapTileCoord::new(coord.x - 1, coord.y)),
                            right_neighbor_solid: map
                                .tile_collides_at(LegacyMapTileCoord::new(coord.x + 1, coord.y)),
                            push_left_clear: !map
                                .tile_collides_at(LegacyMapTileCoord::new(coord.x - 1, coord.y)),
                            push_right_clear: !map
                                .tile_collides_at(LegacyMapTileCoord::new(coord.x + 1, coord.y)),
                            physics: PhysicsConstants::default(),
                        },
                    ) {
                        LegacyCeilingTileResponse::HitBlock { coord: hit_coord }
                        | LegacyCeilingTileResponse::BreakBlock { coord: hit_coord } => {
                            player.body.y = vertical_response.moving.bounds.min.y;
                            if let Some(hit_tile) = map.tile_metadata_at(hit_coord) {
                                let breakable = hit_tile.breakable();
                                let coin_block = hit_tile.coin_block();
                                let tile_coord = legacy_map_coord_to_tile_coord(hit_coord);
                                let portal_guard = report_context
                                    .portal_guards
                                    .iter()
                                    .copied()
                                    .find(|guard| guard.reservation.protects(tile_coord));
                                let blocked_by_portal_guard = portal_guard.is_some();
                                report.block_hits.push(LegacyRuntimePlayerCeilingBlockHit {
                                    coord: hit_coord,
                                    tile_id: hit_tile.tile_id,
                                    breakable,
                                    coin_block,
                                    play_hit_sound: legacy_block_hit_sound_requested(
                                        LegacyBlockHitSoundContext {
                                            blocked_by_portal_guard,
                                            editor_mode: false,
                                            in_map: bounds.contains(hit_coord),
                                        },
                                    ),
                                    portal_guard,
                                });
                                if breakable || coin_block {
                                    let content = map
                                        .legacy_entity_kind_at(hit_coord)
                                        .and_then(legacy_block_bounce_content_from_entity);
                                    if !blocked_by_portal_guard {
                                        report.block_bounce_schedules.push(
                                            LegacyRuntimePlayerBlockBounceSchedule {
                                                coord: hit_coord,
                                                schedule: legacy_block_bounce_schedule(
                                                    LegacyBlockBounceContext {
                                                        coord: tile_coord,
                                                        hitter_size,
                                                        content,
                                                    },
                                                ),
                                            },
                                        );
                                        if let Some(outcome) = legacy_block_contained_reward_reveal(
                                            LegacyBlockContainedRewardRevealContext {
                                                coord: tile_coord,
                                                spriteset: report_context.spriteset,
                                                invisible: hit_tile.invisible(),
                                                content,
                                            },
                                        ) {
                                            if let Some(content) = content {
                                                report.contained_reward_reveals.push(
                                                    LegacyRuntimeBlockContainedRewardRevealIntent {
                                                        coord: hit_coord,
                                                        content,
                                                        outcome,
                                                    },
                                                );
                                            }
                                        }
                                        if coin_block && content.is_none() {
                                            report.coin_block_rewards.push(
                                                legacy_runtime_coin_block_reward_intent(
                                                    hit_coord,
                                                    report_context.spriteset,
                                                    LegacyCoinBlockRewardKind::Single {
                                                        invisible: hit_tile.invisible(),
                                                    },
                                                    report_context.coin_count,
                                                    report_context.life_count_enabled,
                                                    report_context.player_count,
                                                ),
                                            );
                                        }
                                        if content == Some(LegacyBlockBounceContentKind::ManyCoins)
                                        {
                                            report.coin_block_rewards.push(
                                                legacy_runtime_coin_block_reward_intent(
                                                    hit_coord,
                                                    report_context.spriteset,
                                                    LegacyCoinBlockRewardKind::ManyCoins {
                                                        timer: legacy_many_coins_timer_state(
                                                            tile_coord,
                                                            report_context.many_coins_timers,
                                                        ),
                                                    },
                                                    report_context.coin_count,
                                                    report_context.life_count_enabled,
                                                    report_context.player_count,
                                                ),
                                            );
                                        }
                                        if let Some(outcome) =
                                            legacy_runtime_block_top_coin_collection(
                                                hit_coord,
                                                map,
                                                report_context.coin_count,
                                                report_context.life_count_enabled,
                                                report_context.player_count,
                                            )
                                        {
                                            report.top_coin_collections.push(outcome);
                                        }
                                        report.item_jump_requests.extend(
                                            legacy_runtime_block_item_jump_requests(
                                                hit_coord,
                                                report_context.jump_items,
                                            ),
                                        );
                                        report.enemy_shot_requests.extend(
                                            legacy_runtime_block_enemy_shot_requests(
                                                hit_coord,
                                                report_context.top_enemies,
                                            ),
                                        );
                                    }
                                    if let Some(outcome) = legacy_empty_breakable_block_destroy(
                                        LegacyEmptyBreakableBlockDestroyContext {
                                            coord: tile_coord,
                                            hitter_size: Some(hitter_size),
                                            is_coin_block: coin_block,
                                            content,
                                        },
                                        report_context.portal_reservations,
                                    ) {
                                        report.empty_breakable_block_destroys.push(
                                            LegacyRuntimeEmptyBreakableBlockDestroyIntent {
                                                coord: hit_coord,
                                                outcome,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                        LegacyCeilingTileResponse::PushPlayer { .. } => continue,
                    }
                } else {
                    player.body.y = vertical_response.moving.bounds.min.y;
                    player.movement.speed_y = vertical_response.moving.velocity.y;
                    apply_legacy_floor_landing_state(&mut player.movement);
                }

                vertical_blocked = true;
                report.vertical = Some(LegacyRuntimePlayerTileCollision {
                    coord,
                    tile_id: tile.tile_id,
                    axis: LegacyRuntimePlayerCollisionAxis::Vertical,
                });
            }
            CollisionKind::Horizontal if !horizontal_blocked => {
                if legacy_side_invisible_tile_suppresses_default(tile.invisible()) {
                    continue;
                }

                let response = legacy_horizontal_collision_response(
                    legacy_player_collision_actor(player),
                    LegacyCollisionTarget::new(
                        target.min.x,
                        target.min.y,
                        target.width(),
                        target.height(),
                        Some(Vec2::ZERO),
                    ),
                    LegacyCollisionHandlerResult::ApplyDefault,
                    LegacyCollisionHandlerResult::ApplyDefault,
                );

                if response.resolved {
                    player.body.x = response.moving.bounds.min.x;
                    player.movement.speed_x = response.moving.velocity.x;
                    horizontal_blocked = true;
                    report.horizontal = Some(LegacyRuntimePlayerTileCollision {
                        coord,
                        tile_id: tile.tile_id,
                        axis: LegacyRuntimePlayerCollisionAxis::Horizontal,
                    });
                }
            }
            CollisionKind::None
            | CollisionKind::Passive
            | CollisionKind::Horizontal
            | CollisionKind::Vertical => {}
        }
    }

    if !vertical_blocked {
        player.body.y += player.movement.speed_y * dt;
        apply_legacy_start_fall_after_vertical_move(&mut player.movement, dt);
    }

    if !horizontal_blocked {
        player.body.x += player.movement.speed_x * dt;
    }

    report
}

fn legacy_runtime_block_item_jump_requests(
    block_coord: LegacyMapTileCoord,
    items: &[LegacyRuntimeBlockJumpItemSnapshot],
) -> Vec<LegacyRuntimeBlockItemJumpIntent> {
    let core_items: Vec<_> = items.iter().map(|item| item.to_core_item()).collect();

    legacy_block_item_jump_requests(legacy_map_coord_to_tile_coord(block_coord), &core_items)
        .into_iter()
        .map(|request| LegacyRuntimeBlockItemJumpIntent {
            coord: block_coord,
            request,
        })
        .collect()
}

fn legacy_runtime_block_enemy_shot_requests(
    block_coord: LegacyMapTileCoord,
    enemies: &[LegacyRuntimeBlockTopEnemySnapshot],
) -> Vec<LegacyRuntimeBlockEnemyShotIntent> {
    let core_enemies: Vec<_> = enemies.iter().map(|enemy| enemy.to_core_enemy()).collect();

    legacy_block_enemy_shot_requests(legacy_map_coord_to_tile_coord(block_coord), &core_enemies)
        .into_iter()
        .map(|request| LegacyRuntimeBlockEnemyShotIntent {
            coord: block_coord,
            request,
        })
        .collect()
}

fn legacy_runtime_block_top_coin_collection(
    block_coord: LegacyMapTileCoord,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
    coin_count: u32,
    life_count_enabled: bool,
    player_count: usize,
) -> Option<LegacyRuntimeBlockTopCoinCollectionIntent> {
    let coin_coord = LegacyMapTileCoord::new(block_coord.x, block_coord.y - 1);
    if !map
        .tile_metadata_at(coin_coord)
        .is_some_and(|tile| tile.coin())
    {
        return None;
    }

    legacy_block_top_coin_collection(
        LegacyBlockTopCoinCollectionContext {
            top_coin_coord: Some(legacy_map_coord_to_tile_coord(coin_coord)),
            coin_count,
            life_count_enabled,
            player_count,
        },
        LegacyCoinBlockRewardConstants::default(),
    )
    .map(|outcome| LegacyRuntimeBlockTopCoinCollectionIntent {
        block_coord,
        coin_coord,
        outcome,
    })
}

fn legacy_runtime_coin_block_reward_intent(
    coord: LegacyMapTileCoord,
    spriteset: LegacyBlockSpriteset,
    kind: LegacyCoinBlockRewardKind,
    coin_count: u32,
    life_count_enabled: bool,
    player_count: usize,
) -> LegacyRuntimeCoinBlockRewardIntent {
    LegacyRuntimeCoinBlockRewardIntent {
        coord,
        outcome: legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord: legacy_map_coord_to_tile_coord(coord),
                spriteset,
                kind,
                coin_count,
                life_count_enabled,
                player_count,
            },
            LegacyCoinBlockRewardConstants::default(),
        ),
    }
}

fn legacy_runtime_coin_counter_intent(
    source: LegacyRuntimeCoinCounterSource,
    coin_count_before: u32,
    life_count_enabled: bool,
    player_count: usize,
    score_delta: u32,
) -> LegacyRuntimeCoinCounterIntent {
    let incremented_coin_count = coin_count_before + 1;
    let (coin_count_after, life_reward) = if incremented_coin_count
        == LegacyCoinBlockRewardConstants::default().coin_life_threshold
    {
        (
            0,
            Some(LegacyCoinLifeReward {
                grant_lives_to_players: if life_count_enabled { player_count } else { 0 },
                respawn_players: life_count_enabled,
                play_sound: true,
            }),
        )
    } else {
        (incremented_coin_count, None)
    };

    legacy_runtime_coin_counter_intent_from_reward(
        source,
        coin_count_before,
        coin_count_after,
        life_reward,
        score_delta,
    )
}

const fn legacy_runtime_coin_counter_intent_from_reward(
    source: LegacyRuntimeCoinCounterSource,
    coin_count_before: u32,
    coin_count_after: u32,
    life_reward: Option<LegacyCoinLifeReward>,
    score_delta: u32,
) -> LegacyRuntimeCoinCounterIntent {
    LegacyRuntimeCoinCounterIntent {
        source,
        coin_count_before,
        coin_count_after,
        life_reward,
        score_delta,
    }
}

fn legacy_runtime_score_counter_intent(
    source: LegacyRuntimeScoreSource,
    score_count: &mut u32,
    score_delta: u32,
    scrolling_score: Option<LegacyScrollingScoreState>,
) -> LegacyRuntimeScoreCounterIntent {
    let score_count_before = *score_count;
    *score_count = score_count.saturating_add(score_delta);

    LegacyRuntimeScoreCounterIntent {
        source,
        score_count_before,
        score_delta,
        score_count_after: *score_count,
        scrolling_score,
    }
}

const fn legacy_runtime_scrolling_score_animation_from_score_counter_intent(
    intent: &LegacyRuntimeScoreCounterIntent,
) -> Option<LegacyRuntimeScrollingScoreAnimationState> {
    match intent.scrolling_score {
        Some(scrolling_score) => Some(LegacyRuntimeScrollingScoreAnimationState {
            source: intent.source,
            score: scrolling_score,
        }),
        None => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePlayerCoinPickup {
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub clear_tile_id: TileId,
    pub score_delta: u32,
    pub sound: LegacySoundEffect,
}

fn collect_legacy_player_coin_pickups(
    level: &mut Mari0Level,
    tiles: &LegacyTileMetadataTable,
    player: LegacyRuntimePlayer,
) -> Vec<LegacyRuntimePlayerCoinPickup> {
    let query = LegacyTileMetadataMapQuery::new(level, tiles);
    let mut pickups = Vec::new();

    for coord in legacy_small_player_coin_probe_coords(player.body) {
        if pickups
            .iter()
            .any(|pickup: &LegacyRuntimePlayerCoinPickup| pickup.coord == coord)
        {
            continue;
        }
        let Some(tile) = query.tile_metadata_at(coord) else {
            continue;
        };
        if !tile.coin() {
            continue;
        }

        pickups.push(LegacyRuntimePlayerCoinPickup {
            coord,
            tile_id: tile.tile_id,
            clear_tile_id: TileId(1),
            score_delta: 200,
            sound: LegacySoundEffect::Coin,
        });
    }

    for pickup in &pickups {
        if let Some((x, y)) = legacy_coord_to_zero_based(pickup.coord) {
            level.set_cell_tile_id(x, y, pickup.clear_tile_id.0);
        }
    }

    pickups
}

fn legacy_small_player_coin_probe_coords(body: PlayerBodyBounds) -> [LegacyMapTileCoord; 2] {
    let x = (body.x + body.width / 2.0).floor() as i32 + 1;
    [
        LegacyMapTileCoord::new(x, (body.y + body.height).floor() as i32 + 1),
        LegacyMapTileCoord::new(x, (body.y + body.height / 2.0).floor() as i32 + 1),
    ]
}

#[must_use]
fn legacy_runtime_fireball_launch_intent(
    player: LegacyRuntimePlayer,
    snapshot: LegacyRuntimeFireballLaunchSnapshot,
    should_update: bool,
) -> Option<LegacyRuntimeFireballLaunchIntent> {
    let constants = LegacyFireballConstants::default();
    if !should_update
        || !snapshot.requested
        || !snapshot.controls_enabled
        || !snapshot.flower_power
        || snapshot.ducking
        || snapshot.active_fireball_count >= constants.max_count
    {
        return None;
    }

    let direction = if snapshot.pointing_angle > 0.0 {
        LegacyEnemyDirection::Left
    } else {
        LegacyEnemyDirection::Right
    };
    let source_x = player.body.x + 0.5;
    let source_y = player.body.y;

    Some(LegacyRuntimeFireballLaunchIntent {
        source_x,
        source_y,
        direction,
        spawn: LegacyFireballState::spawn(source_x, source_y, direction, constants),
        fireball_count_before: snapshot.active_fireball_count,
        fireball_count_after: snapshot.active_fireball_count + 1,
        fire_animation_timer_reset: 0.0,
        sound: LegacySoundEffect::Fireball,
    })
}

fn progress_legacy_runtime_fireball_projectiles(
    projectiles: &mut Vec<LegacyFireballState>,
    projected_collisions: &mut LegacyRuntimeProjectedFireballProjectileCollisionState,
    dt: f32,
    x_scroll: f32,
) -> LegacyRuntimeFireballProjectileProgressReport {
    let constants = LegacyFireballConstants::default();
    let viewport = LegacyFireballViewport {
        x_scroll,
        width: LEGACY_RUNTIME_VIEWPORT_WIDTH_TILES,
    };
    let mut reports = Vec::new();
    let mut release_summaries = Vec::new();
    let queue_len_before_prune = projectiles.len();

    for (index, projectile) in projectiles.iter_mut().enumerate() {
        let Some(projected_state_before) = projected_collisions.projected_state(index) else {
            let state_before = *projectile;
            let update = update_legacy_fireball(projectile, constants, dt, viewport);
            if let Some(summary) =
                legacy_runtime_fireball_projectile_update_release_summary(index, update)
            {
                release_summaries.push(summary);
            }
            reports.push(LegacyRuntimeFireballProjectileUpdateReport {
                index,
                state_before,
                state_after: *projectile,
                update,
            });
            continue;
        };

        if projected_state_before.destroy {
            continue;
        }

        let mut projected_state_after = projected_state_before;
        let update = update_legacy_fireball(&mut projected_state_after, constants, dt, viewport);
        if let Some(summary) =
            legacy_runtime_fireball_projectile_update_release_summary(index, update)
        {
            release_summaries.push(summary);
        }
        projected_collisions.record_projected_state(index, projected_state_after);
        reports.push(LegacyRuntimeFireballProjectileUpdateReport {
            index,
            state_before: projected_state_before,
            state_after: projected_state_after,
            update,
        });
    }

    let queue_len_after_prune = projectiles
        .iter()
        .enumerate()
        .filter(|(index, projectile)| {
            projected_collisions
                .projected_state(*index)
                .map_or(!projectile.destroy, |projected| !projected.destroy)
        })
        .count();

    let mut index = 0;
    projectiles.retain(|projectile| {
        let keep = projected_collisions.projected_state(index).is_some() || !projectile.destroy;
        index += 1;
        keep
    });

    LegacyRuntimeFireballProjectileProgressReport {
        reports,
        release_summaries,
        queue_len_before_prune,
        queue_len_after_prune,
    }
}

fn preview_legacy_runtime_fireball_render_intents(
    projectiles: &[LegacyFireballState],
    projected_collisions: &LegacyRuntimeProjectedFireballProjectileCollisionState,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeFireballRenderIntentPreviewReport {
    let mut report = LegacyRuntimeFireballRenderIntentPreviewReport::default();

    for (index, projectile) in projectiles.iter().copied().enumerate() {
        let (source, state) = match projected_collisions.projected_state(index) {
            Some(projected) if projected.destroy => {
                report.suppressed_projected_removal_indices.push(index);
                continue;
            }
            Some(projected) => (
                LegacyRuntimeFireballRenderSource::ProjectedProjectileCollision,
                projected,
            ),
            None if projectile.destroy => continue,
            None => (
                LegacyRuntimeFireballRenderSource::LiveProjectile,
                projectile,
            ),
        };

        report
            .previews
            .push(legacy_runtime_fireball_render_intent_preview(
                index, source, state, render,
            ));
    }

    report
}

fn legacy_runtime_fireball_render_intent_preview(
    projectile_index: usize,
    source: LegacyRuntimeFireballRenderSource,
    state: LegacyFireballState,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeFireballRenderIntentPreview {
    LegacyRuntimeFireballRenderIntentPreview {
        projectile_index,
        source,
        state,
        frame: state.frame,
        frame_kind: legacy_runtime_fireball_render_frame_kind(state.frame),
        image_path: LEGACY_RUNTIME_FIREBALL_IMAGE_PATH,
        quad: legacy_runtime_fireball_render_quad(state.frame),
        draw_x_px: (((state.x - render.xscroll) * 16.0 + 4.0) * render.scale).floor(),
        draw_y_px: ((state.y * 16.0 - 4.0) * render.scale).floor(),
        rotation: state.rotation,
        scale: render.scale,
        live_rendering_executed: false,
        live_projectile_queue_mutated: false,
    }
}

fn preview_legacy_runtime_portal_projectile_render_intents(
    projectiles: &[LegacyRuntimePortalProjectileSnapshot],
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimePortalProjectileRenderIntentPreviewReport {
    LegacyRuntimePortalProjectileRenderIntentPreviewReport {
        previews: projectiles
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, projectile)| {
                legacy_runtime_portal_projectile_render_intent_preview(index, projectile, render)
            })
            .collect(),
    }
}

fn legacy_runtime_portal_projectile_render_intent_preview(
    projectile_index: usize,
    snapshot: LegacyRuntimePortalProjectileSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimePortalProjectileRenderIntentPreview {
    let particle_draws = snapshot
        .particles
        .iter()
        .copied()
        .enumerate()
        .map(|(particle_index, particle)| {
            legacy_runtime_portal_projectile_particle_render_preview(
                particle_index,
                particle,
                render,
            )
        })
        .collect::<Vec<_>>();
    let head_draw = (snapshot.timer < snapshot.time)
        .then(|| legacy_runtime_portal_projectile_head_render_preview(&snapshot, render));

    LegacyRuntimePortalProjectileRenderIntentPreview {
        projectile_index,
        snapshot,
        particle_draws,
        head_draw,
        particles_drawn_before_head: true,
        color_reset_after_draw: false,
        live_rendering_executed: false,
        live_projectile_physics_migrated: false,
        live_portal_mutated: false,
    }
}

fn legacy_runtime_portal_projectile_head_render_preview(
    snapshot: &LegacyRuntimePortalProjectileSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimePortalProjectileHeadRenderPreview {
    LegacyRuntimePortalProjectileHeadRenderPreview {
        x: snapshot.x,
        y: snapshot.y,
        draw_x_px: ((snapshot.x - render.xscroll) * 16.0 * render.scale).floor(),
        draw_y_px: ((snapshot.y - 0.5) * 16.0 * render.scale).floor(),
        color: snapshot.color,
        image_path: LEGACY_RUNTIME_PORTAL_PROJECTILE_IMAGE_PATH,
        origin_x_px: 3.0,
        origin_y_px: 3.0,
        rotation: 0.0,
        scale: render.scale,
        live_rendering_executed: false,
    }
}

fn legacy_runtime_portal_projectile_particle_render_preview(
    particle_index: usize,
    particle: LegacyRuntimePortalProjectileParticleSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimePortalProjectileParticleRenderPreview {
    LegacyRuntimePortalProjectileParticleRenderPreview {
        particle_index,
        x: particle.x,
        y: particle.y,
        draw_x_px: ((particle.x - render.xscroll) * 16.0 * render.scale).floor(),
        draw_y_px: ((particle.y - 0.5) * 16.0 * render.scale).floor(),
        color: particle.color,
        image_path: LEGACY_RUNTIME_PORTAL_PROJECTILE_PARTICLE_IMAGE_PATH,
        origin_x_px: 0.5,
        origin_y_px: 0.5,
        rotation: 0.0,
        scale: render.scale,
        live_rendering_executed: false,
    }
}

fn preview_legacy_runtime_emancipation_grill_render_intents(
    grills: &[LegacyRuntimeEmancipationGrillSnapshot],
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeEmancipationGrillRenderIntentPreviewReport {
    LegacyRuntimeEmancipationGrillRenderIntentPreviewReport {
        previews: grills
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, grill)| {
                legacy_runtime_emancipation_grill_render_intent_preview(index, grill, render)
            })
            .collect(),
    }
}

fn legacy_runtime_emancipation_grill_render_intent_preview(
    grill_index: usize,
    snapshot: LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeEmancipationGrillRenderIntentPreview {
    if snapshot.destroyed {
        return LegacyRuntimeEmancipationGrillRenderIntentPreview {
            grill_index,
            snapshot,
            scissor: None,
            line_rect: None,
            particle_draws: Vec::new(),
            side_draws: Vec::new(),
            scissor_cleared_after_particles: false,
            color_reset_after_line: false,
            live_rendering_executed: false,
            live_grill_physics_migrated: false,
        };
    }

    let scissor = legacy_runtime_emancipation_grill_scissor_preview(&snapshot, render);
    let line_rect = legacy_runtime_emancipation_grill_line_preview(&snapshot, render);
    let particle_draws = snapshot
        .particles
        .iter()
        .copied()
        .enumerate()
        .map(|(particle_index, particle)| {
            legacy_runtime_emancipation_grill_particle_render_preview(
                &snapshot,
                particle_index,
                particle,
                render,
            )
        })
        .collect();
    let side_draws = legacy_runtime_emancipation_grill_side_render_previews(&snapshot, render);

    LegacyRuntimeEmancipationGrillRenderIntentPreview {
        grill_index,
        snapshot,
        scissor: Some(scissor),
        line_rect: Some(line_rect),
        particle_draws,
        side_draws,
        scissor_cleared_after_particles: true,
        color_reset_after_line: true,
        live_rendering_executed: false,
        live_grill_physics_migrated: false,
    }
}

fn legacy_runtime_emancipation_grill_scissor_preview(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeEmancipationGrillScissorPreview {
    match snapshot.direction {
        LegacyRuntimeEmancipationGrillDirection::Horizontal => {
            LegacyRuntimeEmancipationGrillScissorPreview {
                x_px: legacy_runtime_emancipation_grill_horizontal_start_px(snapshot, render),
                y_px: ((snapshot.y - 1.0) * 16.0 - 2.0) * render.scale,
                width_px: snapshot.range_px
                    - LEGACY_RUNTIME_EMANCIPATION_GRILL_IMAGE_WIDTH_PX * render.scale,
                height_px: render.scale * 4.0,
            }
        }
        LegacyRuntimeEmancipationGrillDirection::Vertical => {
            LegacyRuntimeEmancipationGrillScissorPreview {
                x_px: (((snapshot.x - 1.0 - render.xscroll) * 16.0 + 6.0) * render.scale).floor(),
                y_px: legacy_runtime_emancipation_grill_vertical_start_px(snapshot, render)
                    - 8.0 * render.scale,
                width_px: render.scale * 4.0,
                height_px: snapshot.range_px
                    - LEGACY_RUNTIME_EMANCIPATION_GRILL_IMAGE_WIDTH_PX * render.scale,
            }
        }
    }
}

fn legacy_runtime_emancipation_grill_line_preview(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeEmancipationGrillLinePreview {
    let scissor = legacy_runtime_emancipation_grill_scissor_preview(snapshot, render);
    LegacyRuntimeEmancipationGrillLinePreview {
        x_px: scissor.x_px,
        y_px: scissor.y_px,
        width_px: match snapshot.direction {
            LegacyRuntimeEmancipationGrillDirection::Horizontal => snapshot.range_px,
            LegacyRuntimeEmancipationGrillDirection::Vertical => render.scale * 4.0,
        },
        height_px: match snapshot.direction {
            LegacyRuntimeEmancipationGrillDirection::Horizontal => render.scale * 4.0,
            LegacyRuntimeEmancipationGrillDirection::Vertical => {
                snapshot.range_px - LEGACY_RUNTIME_EMANCIPATION_GRILL_IMAGE_WIDTH_PX * render.scale
            }
        },
        color: LEGACY_RUNTIME_EMANCIPATION_GRILL_LINE_COLOR,
    }
}

fn legacy_runtime_emancipation_grill_particle_render_preview(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    particle_index: usize,
    particle: LegacyRuntimeEmancipationGrillParticleSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeEmancipationGrillParticleRenderPreview {
    let (draw_x_px, draw_y_px, rotation, origin_x_px) = match snapshot.direction {
        LegacyRuntimeEmancipationGrillDirection::Horizontal => {
            let start_left =
                legacy_runtime_emancipation_grill_horizontal_start_px(snapshot, render);
            let start_right = legacy_runtime_emancipation_grill_horizontal_end_px(snapshot, render);
            let y_px = ((snapshot.y - 1.0) * 16.0 - particle.modifier_px) * render.scale;
            match particle.direction {
                LegacyRuntimeEmancipationGrillParticleDirection::Forward => (
                    (start_left + snapshot.range_px * particle.progress).floor(),
                    y_px,
                    consts::FRAC_PI_2,
                    0.0,
                ),
                LegacyRuntimeEmancipationGrillParticleDirection::Backward => (
                    (start_right - snapshot.range_px * particle.progress).floor(),
                    y_px,
                    -consts::FRAC_PI_2,
                    1.0,
                ),
            }
        }
        LegacyRuntimeEmancipationGrillDirection::Vertical => {
            let start_up = legacy_runtime_emancipation_grill_vertical_start_px(snapshot, render);
            let start_down = legacy_runtime_emancipation_grill_vertical_end_px(snapshot, render);
            let x_px = ((snapshot.x - 1.0 - render.xscroll) * 16.0 - particle.modifier_px + 9.0)
                * render.scale;
            match particle.direction {
                LegacyRuntimeEmancipationGrillParticleDirection::Forward => (
                    x_px.floor(),
                    start_up + snapshot.range_px * particle.progress,
                    consts::PI,
                    0.0,
                ),
                LegacyRuntimeEmancipationGrillParticleDirection::Backward => (
                    x_px.floor(),
                    start_down - snapshot.range_px * particle.progress,
                    0.0,
                    1.0,
                ),
            }
        }
    };

    LegacyRuntimeEmancipationGrillParticleRenderPreview {
        particle_index,
        progress: particle.progress,
        direction: particle.direction,
        draw_x_px,
        draw_y_px,
        image_path: LEGACY_RUNTIME_EMANCIPATION_GRILL_PARTICLE_IMAGE_PATH,
        origin_x_px,
        origin_y_px: 0.0,
        rotation,
        scale: render.scale,
        live_rendering_executed: false,
    }
}

fn legacy_runtime_emancipation_grill_side_render_previews(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> Vec<LegacyRuntimeEmancipationGrillSideRenderPreview> {
    match snapshot.direction {
        LegacyRuntimeEmancipationGrillDirection::Horizontal => {
            let start_left =
                legacy_runtime_emancipation_grill_horizontal_start_px(snapshot, render);
            let start_right = legacy_runtime_emancipation_grill_horizontal_end_px(snapshot, render);
            vec![
                LegacyRuntimeEmancipationGrillSideRenderPreview {
                    side_index: 0,
                    draw_x_px: start_left,
                    draw_y_px: ((snapshot.y - 1.0) * 16.0 - 4.0) * render.scale,
                    image_path: LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
                    rotation: 0.0,
                    scale: render.scale,
                    live_rendering_executed: false,
                },
                LegacyRuntimeEmancipationGrillSideRenderPreview {
                    side_index: 1,
                    draw_x_px: start_right + 16.0 * render.scale,
                    draw_y_px: ((snapshot.y - 1.0) * 16.0 + 4.0) * render.scale,
                    image_path: LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
                    rotation: consts::PI,
                    scale: render.scale,
                    live_rendering_executed: false,
                },
            ]
        }
        LegacyRuntimeEmancipationGrillDirection::Vertical => {
            let start_up = legacy_runtime_emancipation_grill_vertical_start_px(snapshot, render);
            let start_down = legacy_runtime_emancipation_grill_vertical_end_px(snapshot, render);
            vec![
                LegacyRuntimeEmancipationGrillSideRenderPreview {
                    side_index: 0,
                    draw_x_px: (((snapshot.x - render.xscroll) * 16.0 - 4.0) * render.scale)
                        .floor(),
                    draw_y_px: start_up - 8.0 * render.scale,
                    image_path: LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
                    rotation: consts::FRAC_PI_2,
                    scale: render.scale,
                    live_rendering_executed: false,
                },
                LegacyRuntimeEmancipationGrillSideRenderPreview {
                    side_index: 1,
                    draw_x_px: (((snapshot.x - render.xscroll) * 16.0 - 12.0) * render.scale)
                        .floor(),
                    draw_y_px: start_down + 8.0 * render.scale,
                    image_path: LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
                    rotation: -consts::FRAC_PI_2,
                    scale: render.scale,
                    live_rendering_executed: false,
                },
            ]
        }
    }
}

fn legacy_runtime_emancipation_grill_horizontal_start_px(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> f32 {
    ((snapshot.start - 1.0 - render.xscroll) * 16.0 * render.scale).floor()
}

fn legacy_runtime_emancipation_grill_horizontal_end_px(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> f32 {
    ((snapshot.end - 1.0 - render.xscroll) * 16.0 * render.scale).floor()
}

fn legacy_runtime_emancipation_grill_vertical_start_px(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> f32 {
    ((snapshot.start - 1.0) * 16.0 * render.scale).floor()
}

fn legacy_runtime_emancipation_grill_vertical_end_px(
    snapshot: &LegacyRuntimeEmancipationGrillSnapshot,
    render: LegacyRuntimeRenderContext,
) -> f32 {
    ((snapshot.end - 1.0) * 16.0 * render.scale).floor()
}

fn preview_legacy_runtime_door_render_intents(
    doors: &[LegacyRuntimeDoorSnapshot],
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeDoorRenderIntentPreviewReport {
    LegacyRuntimeDoorRenderIntentPreviewReport {
        previews: doors
            .iter()
            .copied()
            .enumerate()
            .map(|(index, door)| legacy_runtime_door_render_intent_preview(index, door, render))
            .collect(),
    }
}

fn legacy_runtime_door_render_intent_preview(
    door_index: usize,
    snapshot: LegacyRuntimeDoorSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeDoorRenderIntentPreview {
    let (ymod_tiles, center_rotation_delta) = legacy_runtime_door_motion(snapshot.timer);
    let part_draws = match snapshot.direction {
        LegacyRuntimeDoorDirection::Horizontal => [
            legacy_runtime_door_part_render_preview(
                0,
                LegacyRuntimeDoorPartKind::Piece,
                LegacyRuntimeDoorPartRenderSpec {
                    draw_x_px: ((snapshot.x + 14.0 / 16.0 - render.xscroll - ymod_tiles)
                        * 16.0
                        * render.scale)
                        .floor(),
                    draw_y_px: (snapshot.y - 4.0 / 16.0) * 16.0 * render.scale,
                    rotation: consts::FRAC_PI_2,
                    origin_x_px: 4.0,
                    origin_y_px: 0.0,
                    scale: render.scale,
                },
            ),
            legacy_runtime_door_part_render_preview(
                1,
                LegacyRuntimeDoorPartKind::Piece,
                LegacyRuntimeDoorPartRenderSpec {
                    draw_x_px: ((snapshot.x + 18.0 / 16.0 - render.xscroll + ymod_tiles)
                        * 16.0
                        * render.scale)
                        .floor(),
                    draw_y_px: (snapshot.y - 4.0 / 16.0) * 16.0 * render.scale,
                    rotation: consts::PI * 1.5,
                    origin_x_px: 4.0,
                    origin_y_px: 0.0,
                    scale: render.scale,
                },
            ),
            legacy_runtime_door_part_render_preview(
                2,
                LegacyRuntimeDoorPartKind::Center,
                LegacyRuntimeDoorPartRenderSpec {
                    draw_x_px: ((snapshot.x + 1.0 - render.xscroll - ymod_tiles)
                        * 16.0
                        * render.scale)
                        .floor(),
                    draw_y_px: (snapshot.y - 4.0 / 16.0) * 16.0 * render.scale,
                    rotation: consts::FRAC_PI_2 - center_rotation_delta,
                    origin_x_px: 4.0,
                    origin_y_px: 2.0,
                    scale: render.scale,
                },
            ),
            legacy_runtime_door_part_render_preview(
                3,
                LegacyRuntimeDoorPartKind::Center,
                LegacyRuntimeDoorPartRenderSpec {
                    draw_x_px: ((snapshot.x + 1.0 - render.xscroll + ymod_tiles)
                        * 16.0
                        * render.scale)
                        .floor(),
                    draw_y_px: (snapshot.y - 4.0 / 16.0) * 16.0 * render.scale,
                    rotation: consts::PI * 1.5 - center_rotation_delta,
                    origin_x_px: 4.0,
                    origin_y_px: 2.0,
                    scale: render.scale,
                },
            ),
        ],
        LegacyRuntimeDoorDirection::Vertical => {
            let x = ((snapshot.x + 0.25 - render.xscroll) * 16.0 * render.scale).floor();
            [
                legacy_runtime_door_part_render_preview(
                    0,
                    LegacyRuntimeDoorPartKind::Piece,
                    LegacyRuntimeDoorPartRenderSpec {
                        draw_x_px: x,
                        draw_y_px: (snapshot.y + 6.0 / 16.0 - ymod_tiles) * 16.0 * render.scale,
                        rotation: consts::PI,
                        origin_x_px: 4.0,
                        origin_y_px: 0.0,
                        scale: render.scale,
                    },
                ),
                legacy_runtime_door_part_render_preview(
                    1,
                    LegacyRuntimeDoorPartKind::Piece,
                    LegacyRuntimeDoorPartRenderSpec {
                        draw_x_px: x,
                        draw_y_px: (snapshot.y + 10.0 / 16.0 + ymod_tiles) * 16.0 * render.scale,
                        rotation: 0.0,
                        origin_x_px: 4.0,
                        origin_y_px: 0.0,
                        scale: render.scale,
                    },
                ),
                legacy_runtime_door_part_render_preview(
                    2,
                    LegacyRuntimeDoorPartKind::Center,
                    LegacyRuntimeDoorPartRenderSpec {
                        draw_x_px: x,
                        draw_y_px: (snapshot.y + 8.0 / 16.0 - ymod_tiles) * 16.0 * render.scale,
                        rotation: center_rotation_delta,
                        origin_x_px: 4.0,
                        origin_y_px: 2.0,
                        scale: render.scale,
                    },
                ),
                legacy_runtime_door_part_render_preview(
                    3,
                    LegacyRuntimeDoorPartKind::Center,
                    LegacyRuntimeDoorPartRenderSpec {
                        draw_x_px: x,
                        draw_y_px: (snapshot.y + 8.0 / 16.0 + ymod_tiles) * 16.0 * render.scale,
                        rotation: consts::PI + center_rotation_delta,
                        origin_x_px: 4.0,
                        origin_y_px: 2.0,
                        scale: render.scale,
                    },
                ),
            ]
        }
    };

    LegacyRuntimeDoorRenderIntentPreview {
        door_index,
        snapshot,
        ymod_tiles,
        center_rotation_delta,
        part_draws,
        live_rendering_executed: false,
        live_door_physics_migrated: false,
        live_door_entity_mutated: false,
    }
}

fn legacy_runtime_door_motion(timer: f32) -> (f32, f32) {
    if timer > 0.5 {
        ((timer - 0.5) * 2.0, consts::FRAC_PI_2)
    } else {
        (0.0, timer * consts::PI)
    }
}

struct LegacyRuntimeDoorPartRenderSpec {
    draw_x_px: f32,
    draw_y_px: f32,
    rotation: f32,
    origin_x_px: f32,
    origin_y_px: f32,
    scale: f32,
}

fn legacy_runtime_door_part_render_preview(
    part_index: usize,
    kind: LegacyRuntimeDoorPartKind,
    spec: LegacyRuntimeDoorPartRenderSpec,
) -> LegacyRuntimeDoorRenderPartPreview {
    LegacyRuntimeDoorRenderPartPreview {
        part_index,
        kind,
        draw_x_px: spec.draw_x_px,
        draw_y_px: spec.draw_y_px,
        image_path: match kind {
            LegacyRuntimeDoorPartKind::Piece => LEGACY_RUNTIME_DOOR_PIECE_IMAGE_PATH,
            LegacyRuntimeDoorPartKind::Center => LEGACY_RUNTIME_DOOR_CENTER_IMAGE_PATH,
        },
        rotation: spec.rotation,
        origin_x_px: spec.origin_x_px,
        origin_y_px: spec.origin_y_px,
        scale: spec.scale,
        live_rendering_executed: false,
    }
}

fn preview_legacy_runtime_wall_indicator_render_intents(
    indicators: &[LegacyRuntimeWallIndicatorSnapshot],
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeWallIndicatorRenderIntentPreviewReport {
    LegacyRuntimeWallIndicatorRenderIntentPreviewReport {
        previews: indicators
            .iter()
            .copied()
            .enumerate()
            .map(|(index, indicator)| {
                legacy_runtime_wall_indicator_render_intent_preview(index, indicator, render)
            })
            .collect(),
    }
}

fn legacy_runtime_wall_indicator_render_intent_preview(
    indicator_index: usize,
    snapshot: LegacyRuntimeWallIndicatorSnapshot,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimeWallIndicatorRenderIntentPreview {
    let quad_index = if snapshot.lighted { 2 } else { 1 };
    LegacyRuntimeWallIndicatorRenderIntentPreview {
        indicator_index,
        snapshot,
        quad_index,
        source_x_px: if snapshot.lighted {
            LEGACY_RUNTIME_WALL_INDICATOR_QUAD_SIZE_PX
        } else {
            0.0
        },
        source_y_px: 0.0,
        source_w_px: LEGACY_RUNTIME_WALL_INDICATOR_QUAD_SIZE_PX,
        source_h_px: LEGACY_RUNTIME_WALL_INDICATOR_QUAD_SIZE_PX,
        image_path: LEGACY_RUNTIME_WALL_INDICATOR_IMAGE_PATH,
        draw_x_px: ((snapshot.x - 1.0 - render.xscroll) * 16.0 * render.scale).floor(),
        draw_y_px: ((snapshot.y - 1.0) * 16.0 - 8.0) * render.scale,
        rotation: 0.0,
        scale_x: render.scale,
        scale_y: render.scale,
        color: LegacyColor::rgb(1.0, 1.0, 1.0),
        live_rendering_executed: false,
        live_wall_indicator_physics_migrated: false,
        live_wall_indicator_entity_mutated: false,
    }
}

fn legacy_runtime_player_render_intent_preview(
    player: LegacyRuntimePlayer,
    render: LegacyRuntimeRenderContext,
    pointing_angle: f32,
    portal_pair_readiness: Option<LegacyRuntimePortalPairReadinessSummary>,
) -> LegacyRuntimePlayerRenderIntentPreview {
    let size = player.power_up.legacy_size();
    let fire_animation_active =
        size > 1 && player.fire_animation_timer < LEGACY_RUNTIME_FIRE_ANIMATION_TIME;
    let render_frame = legacy_runtime_player_render_frame(
        size,
        player.movement.animation_state,
        player.movement.ducking,
        fire_animation_active,
    );
    let quad = legacy_runtime_player_render_quad(
        render_frame,
        player.movement.run_frame,
        player.movement.swim_frame,
    );
    let draw_x_px = (((player.body.x - render.xscroll) * 16.0
        + legacy_runtime_player_offset_x(size))
        * render.scale)
        .floor();
    let draw_y_px =
        ((player.body.y * 16.0 - legacy_runtime_player_offset_y(size)) * render.scale).floor();
    let direction_scale =
        legacy_runtime_player_render_direction_scale(player, render.scale, pointing_angle);

    LegacyRuntimePlayerRenderIntentPreview {
        player_index: 0,
        player,
        body: player.body,
        facing: player.movement.animation_direction,
        animation_state: player.movement.animation_state,
        render_frame,
        run_frame: player.movement.run_frame,
        swim_frame: player.movement.swim_frame,
        size,
        power_up: player.power_up,
        ducking: player.movement.ducking,
        fire_animation_timer: player.fire_animation_timer,
        fire_animation_active,
        image_path: legacy_runtime_player_render_image_path(size),
        quad,
        color_layers: legacy_runtime_player_render_color_layers(
            size,
            player.power_up,
            quad,
            draw_x_px,
            draw_y_px,
            render.scale,
            direction_scale,
        ),
        hat_draw_count: legacy_runtime_player_render_hat_draw_count(
            player,
            size,
            fire_animation_active,
        ),
        hat_draws: legacy_runtime_player_render_hat_draws(
            player,
            size,
            fire_animation_active,
            draw_x_px,
            draw_y_px,
            direction_scale,
        ),
        draw_x_px,
        draw_y_px,
        rotation: 0.0,
        scale: render.scale,
        direction_scale,
        portal_clone: legacy_runtime_player_render_portal_clone_preview(
            player,
            render,
            direction_scale,
            portal_pair_readiness,
        ),
        live_rendering_executed: false,
        live_player_mutated: false,
    }
}

fn legacy_runtime_player_render_portal_clone_preview(
    player: LegacyRuntimePlayer,
    render: LegacyRuntimeRenderContext,
    base_direction_scale: LegacyRuntimePlayerRenderDirectionScale,
    readiness: Option<LegacyRuntimePortalPairReadinessSummary>,
) -> Option<LegacyRuntimePlayerRenderPortalClonePreview> {
    const INPUT_ROTATION: f32 = 0.0;
    let candidate = legacy_runtime_portal_transit_candidate_probe(player, readiness)?;
    let pairing = candidate.candidate_pairing?;
    let transit = legacy_portal_coords(LegacyPortalTransitInput {
        position: Vec2::new(player.body.x, player.body.y),
        velocity: Vec2::new(0.0, 0.0),
        size: Vec2::new(player.body.width, player.body.height),
        rotation: INPUT_ROTATION,
        animation_direction: Some(legacy_runtime_wormhole_animation_direction(
            player.movement.animation_direction,
        )),
        entry: LegacyPortalEndpoint::new(
            pairing.entry.placement.coord.x as f32,
            pairing.entry.placement.coord.y as f32,
            pairing.entry.placement.side,
        ),
        exit: LegacyPortalEndpoint::new(
            pairing.exit.placement.coord.x as f32,
            pairing.exit.placement.coord.y as f32,
            pairing.exit.placement.side,
        ),
        live: false,
        gravity: player.movement.gravity,
        frame_dt: 0.0,
    });
    let output_animation_direction = legacy_runtime_player_animation_direction(
        transit.animation_direction,
        player.movement.animation_direction,
    );
    let animation_direction_flipped =
        output_animation_direction != player.movement.animation_direction;
    let output_body = PlayerBodyBounds::new(
        transit.position.x,
        transit.position.y,
        player.body.width,
        player.body.height,
    );
    let mut direction_scale = base_direction_scale;
    direction_scale.source =
        LegacyRuntimePlayerRenderDirectionScaleSource::PortalCloneAnimationDirection;
    direction_scale.animation_facing = output_animation_direction;
    if animation_direction_flipped {
        direction_scale.direction_scale = -direction_scale.direction_scale;
    }

    Some(LegacyRuntimePlayerRenderPortalClonePreview {
        entry_slot: pairing.entry_slot,
        exit_slot: pairing.exit_slot,
        entry_facing: pairing.entry.placement.side,
        exit_facing: pairing.exit.placement.side,
        entry_scissor: legacy_runtime_player_render_portal_scissor(pairing.entry.placement, render),
        exit_scissor: legacy_runtime_player_render_portal_scissor(pairing.exit.placement, render),
        input_body: player.body,
        output_body,
        input_rotation: INPUT_ROTATION,
        output_rotation: transit.rotation,
        input_animation_direction: player.movement.animation_direction,
        output_animation_direction,
        animation_direction_flipped,
        draw_x_px: (((output_body.x - render.xscroll) * 16.0
            + legacy_runtime_player_offset_x(player.power_up.legacy_size()))
            * render.scale)
            .ceil(),
        draw_y_px: ((output_body.y * 16.0
            - legacy_runtime_player_offset_y(player.power_up.legacy_size()))
            * render.scale)
            .ceil(),
        direction_scale,
        scissor_reset_to_current: true,
        live_rendering_executed: false,
        live_player_mutated: false,
    })
}

fn legacy_runtime_player_render_portal_scissor(
    placement: LegacyRuntimePortalPlacement,
    render: LegacyRuntimeRenderContext,
) -> LegacyRuntimePlayerRenderScissorPreview {
    let (x_tiles, y_tiles, width_tiles, height_tiles) = match placement.side {
        Facing::Right => (
            placement.coord.x as f32,
            placement.coord.y as f32 - 3.5,
            4.0,
            6.0,
        ),
        Facing::Left => (
            placement.coord.x as f32 - 5.0,
            placement.coord.y as f32 - 4.5,
            4.0,
            6.0,
        ),
        Facing::Up => (
            placement.coord.x as f32 - 3.0,
            placement.coord.y as f32 - 5.5,
            6.0,
            4.0,
        ),
        Facing::Down => (
            placement.coord.x as f32 - 4.0,
            placement.coord.y as f32 - 0.5,
            6.0,
            4.0,
        ),
    };

    LegacyRuntimePlayerRenderScissorPreview {
        x_px: ((x_tiles - render.xscroll) * 16.0 * render.scale).floor(),
        y_px: (y_tiles * 16.0 * render.scale).floor(),
        width_px: width_tiles * 16.0 * render.scale,
        height_px: height_tiles * 16.0 * render.scale,
    }
}

const fn legacy_runtime_player_render_direction_scale(
    player: LegacyRuntimePlayer,
    scale: f32,
    pointing_angle: f32,
) -> LegacyRuntimePlayerRenderDirectionScale {
    LegacyRuntimePlayerRenderDirectionScale {
        source: LegacyRuntimePlayerRenderDirectionScaleSource::PlayerPointingAngle,
        pointing_angle,
        animation_facing: player.movement.animation_direction,
        direction_scale: if pointing_angle > 0.0 { -scale } else { scale },
        vertical_scale: scale,
    }
}

const fn legacy_runtime_player_render_frame(
    size: u8,
    animation_state: PlayerAnimationState,
    ducking: bool,
    fire_animation_active: bool,
) -> LegacyRuntimePlayerRenderFrame {
    if size == 1 {
        return match animation_state {
            PlayerAnimationState::Running | PlayerAnimationState::Falling => {
                LegacyRuntimePlayerRenderFrame::SmallRun
            }
            PlayerAnimationState::Idle => LegacyRuntimePlayerRenderFrame::SmallIdle,
            PlayerAnimationState::Sliding => LegacyRuntimePlayerRenderFrame::SmallSlide,
            PlayerAnimationState::Jumping => LegacyRuntimePlayerRenderFrame::SmallJump,
            PlayerAnimationState::Swimming => LegacyRuntimePlayerRenderFrame::SmallSwim,
            PlayerAnimationState::Climbing => LegacyRuntimePlayerRenderFrame::SmallClimb,
            PlayerAnimationState::Dead => LegacyRuntimePlayerRenderFrame::SmallDead,
        };
    }

    if ducking {
        LegacyRuntimePlayerRenderFrame::BigDuck
    } else if fire_animation_active {
        LegacyRuntimePlayerRenderFrame::BigFire
    } else {
        match animation_state {
            PlayerAnimationState::Running | PlayerAnimationState::Falling => {
                LegacyRuntimePlayerRenderFrame::BigRun
            }
            PlayerAnimationState::Idle => LegacyRuntimePlayerRenderFrame::BigIdle,
            PlayerAnimationState::Sliding => LegacyRuntimePlayerRenderFrame::BigSlide,
            PlayerAnimationState::Jumping => LegacyRuntimePlayerRenderFrame::BigJump,
            PlayerAnimationState::Swimming => LegacyRuntimePlayerRenderFrame::BigSwim,
            PlayerAnimationState::Climbing => LegacyRuntimePlayerRenderFrame::BigClimb,
            PlayerAnimationState::Dead => LegacyRuntimePlayerRenderFrame::BigJump,
        }
    }
}

const fn legacy_runtime_player_render_image_path(size: u8) -> &'static str {
    if size == 1 {
        LEGACY_RUNTIME_SMALL_PLAYER_IMAGE_PATH
    } else {
        LEGACY_RUNTIME_BIG_PLAYER_IMAGE_PATH
    }
}

const fn legacy_runtime_player_render_quad(
    frame: LegacyRuntimePlayerRenderFrame,
    run_frame: u8,
    swim_frame: u8,
) -> LegacyRuntimePlayerRenderQuad {
    const DEFAULT_ANGLE_FRAME: u32 = 3;
    let row_index = DEFAULT_ANGLE_FRAME - 1;

    match frame {
        LegacyRuntimePlayerRenderFrame::SmallRun => {
            legacy_runtime_player_small_quad((run_frame as u32).saturating_sub(1) * 20 + 20)
        }
        LegacyRuntimePlayerRenderFrame::SmallIdle => legacy_runtime_player_small_quad(0),
        LegacyRuntimePlayerRenderFrame::SmallSlide => legacy_runtime_player_small_quad(80),
        LegacyRuntimePlayerRenderFrame::SmallJump => legacy_runtime_player_small_quad(100),
        LegacyRuntimePlayerRenderFrame::SmallSwim => {
            legacy_runtime_player_small_quad((swim_frame as u32).saturating_sub(1) * 20 + 180)
        }
        LegacyRuntimePlayerRenderFrame::SmallClimb => legacy_runtime_player_small_quad(140),
        LegacyRuntimePlayerRenderFrame::SmallDead => legacy_runtime_player_small_quad(120),
        LegacyRuntimePlayerRenderFrame::BigRun => legacy_runtime_player_big_quad(
            (run_frame as u32).saturating_sub(1) * 20 + 20,
            row_index,
        ),
        LegacyRuntimePlayerRenderFrame::BigIdle => legacy_runtime_player_big_quad(0, row_index),
        LegacyRuntimePlayerRenderFrame::BigSlide => legacy_runtime_player_big_quad(80, row_index),
        LegacyRuntimePlayerRenderFrame::BigJump => legacy_runtime_player_big_quad(100, row_index),
        LegacyRuntimePlayerRenderFrame::BigSwim => legacy_runtime_player_big_quad(
            (swim_frame as u32).saturating_sub(1) * 20 + 180,
            row_index,
        ),
        LegacyRuntimePlayerRenderFrame::BigClimb => legacy_runtime_player_big_quad(140, row_index),
        LegacyRuntimePlayerRenderFrame::BigDuck => legacy_runtime_player_big_quad(260, row_index),
        LegacyRuntimePlayerRenderFrame::BigFire => legacy_runtime_player_big_quad(120, row_index),
    }
}

const fn legacy_runtime_player_render_layer_image_path(size: u8, layer_index: u8) -> &'static str {
    match (size == 1, layer_index) {
        (true, 0) => LEGACY_RUNTIME_SMALL_PLAYER_LAYER_0_IMAGE_PATH,
        (true, 1) => LEGACY_RUNTIME_SMALL_PLAYER_LAYER_1_IMAGE_PATH,
        (true, 2) => LEGACY_RUNTIME_SMALL_PLAYER_LAYER_2_IMAGE_PATH,
        (true, _) => LEGACY_RUNTIME_SMALL_PLAYER_LAYER_3_IMAGE_PATH,
        (false, 0) => LEGACY_RUNTIME_BIG_PLAYER_LAYER_0_IMAGE_PATH,
        (false, 1) => LEGACY_RUNTIME_BIG_PLAYER_LAYER_1_IMAGE_PATH,
        (false, 2) => LEGACY_RUNTIME_BIG_PLAYER_LAYER_2_IMAGE_PATH,
        (false, _) => LEGACY_RUNTIME_BIG_PLAYER_LAYER_3_IMAGE_PATH,
    }
}

const fn legacy_runtime_player_render_hat_draw_count(
    player: LegacyRuntimePlayer,
    size: u8,
    fire_animation_active: bool,
) -> u8 {
    if !player.draw_hat || player.hat_count == 0 {
        return 0;
    }

    if legacy_runtime_player_render_hat_offsets(
        size,
        player.movement.animation_state,
        player.movement.ducking,
        player.movement.run_frame,
        player.movement.swim_frame,
        fire_animation_active,
    )
    .is_none()
    {
        return 0;
    }

    if player.hat_count > LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS as u8 {
        LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS as u8
    } else {
        player.hat_count
    }
}

const fn legacy_runtime_player_render_color_layers(
    size: u8,
    power_up: LegacyRuntimePlayerPowerUp,
    quad: LegacyRuntimePlayerRenderQuad,
    draw_x_px: f32,
    draw_y_px: f32,
    scale: f32,
    direction_scale: LegacyRuntimePlayerRenderDirectionScale,
) -> [LegacyRuntimePlayerRenderColorLayerPreview; 4] {
    let geometry = LegacyRuntimePlayerRenderLayerGeometry {
        quad,
        draw_x_px,
        draw_y_px,
        scale,
        direction_scale,
    };
    [
        legacy_runtime_player_render_color_layer(size, power_up, 1, 0, geometry),
        legacy_runtime_player_render_color_layer(size, power_up, 2, 1, geometry),
        legacy_runtime_player_render_color_layer(size, power_up, 3, 2, geometry),
        legacy_runtime_player_render_color_layer(size, power_up, 0, 3, geometry),
    ]
}

fn legacy_runtime_player_render_hat_draws(
    player: LegacyRuntimePlayer,
    size: u8,
    fire_animation_active: bool,
    draw_x_px: f32,
    draw_y_px: f32,
    direction_scale: LegacyRuntimePlayerRenderDirectionScale,
) -> [LegacyRuntimePlayerRenderHatPreview; LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS] {
    let mut hat_draws =
        [legacy_runtime_player_render_empty_hat_preview(); LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS];
    if !player.draw_hat || player.hat_count == 0 {
        return hat_draws;
    }

    let Some((offset_x_px, offset_y_px)) = legacy_runtime_player_render_hat_offsets(
        size,
        player.movement.animation_state,
        player.movement.ducking,
        player.movement.run_frame,
        player.movement.swim_frame,
        fire_animation_active,
    ) else {
        return hat_draws;
    };

    let mut stack_y_px = 0;
    let count = usize::from(
        player
            .hat_count
            .min(LEGACY_RUNTIME_PLAYER_MAX_HAT_PREVIEWS as u8),
    );
    for (slot_index, preview) in hat_draws.iter_mut().enumerate().take(count) {
        let hat_id = player.hats[slot_index];
        let config = legacy_runtime_player_render_hat_config(size, hat_id);
        let (tint, tint_source) = if hat_id == 1 {
            legacy_runtime_player_render_layer_tint(player.power_up, 1)
        } else {
            (
                LegacyRuntimePlayerRenderTint {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                LegacyRuntimePlayerRenderTintSource::White,
            )
        };
        *preview = LegacyRuntimePlayerRenderHatPreview {
            drawn: true,
            draw_order: slot_index as u8,
            hat_slot_index: slot_index as u8,
            hat_id,
            size: if size == 1 {
                LegacyRuntimePlayerRenderHatSize::Small
            } else {
                LegacyRuntimePlayerRenderHatSize::Big
            },
            image_path: config.image_path,
            tint,
            tint_source,
            hat_config_x_px: config.x_px,
            hat_config_y_px: config.y_px,
            hat_height_px: config.height_px,
            offset_x_px,
            offset_y_px,
            stack_y_px,
            follows_graphic_layer_index: 3,
            precedes_graphic_layer_index: 0,
            draw_x_px,
            draw_y_px,
            origin_x_px: legacy_runtime_player_quad_center_x(size) - config.x_px + offset_x_px,
            origin_y_px: legacy_runtime_player_quad_center_y(size) - config.y_px
                + offset_y_px
                + stack_y_px,
            rotation: 0.0,
            direction_scale,
            live_rendering_executed: false,
        };
        stack_y_px += config.height_px;
    }
    hat_draws
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LegacyRuntimePlayerRenderHatConfig {
    image_path: &'static str,
    x_px: i32,
    y_px: i32,
    height_px: i32,
}

const fn legacy_runtime_player_render_empty_hat_preview() -> LegacyRuntimePlayerRenderHatPreview {
    LegacyRuntimePlayerRenderHatPreview {
        drawn: false,
        draw_order: 0,
        hat_slot_index: 0,
        hat_id: 0,
        size: LegacyRuntimePlayerRenderHatSize::Small,
        image_path: "",
        tint: LegacyRuntimePlayerRenderTint {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        tint_source: LegacyRuntimePlayerRenderTintSource::White,
        hat_config_x_px: 0,
        hat_config_y_px: 0,
        hat_height_px: 0,
        offset_x_px: 0,
        offset_y_px: 0,
        stack_y_px: 0,
        follows_graphic_layer_index: 3,
        precedes_graphic_layer_index: 0,
        draw_x_px: 0.0,
        draw_y_px: 0.0,
        origin_x_px: 0,
        origin_y_px: 0,
        rotation: 0.0,
        direction_scale: LegacyRuntimePlayerRenderDirectionScale {
            source: LegacyRuntimePlayerRenderDirectionScaleSource::PlayerPointingAngle,
            pointing_angle: 0.0,
            animation_facing: HorizontalDirection::Right,
            direction_scale: 0.0,
            vertical_scale: 0.0,
        },
        live_rendering_executed: false,
    }
}

const fn legacy_runtime_player_render_hat_config(
    size: u8,
    hat_id: u8,
) -> LegacyRuntimePlayerRenderHatConfig {
    if size == 1 {
        match hat_id {
            2 => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_SMALL_TYROLEAN_HAT_IMAGE_PATH,
                x_px: 5,
                y_px: -3,
                height_px: 4,
            },
            3 => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_SMALL_TOWERING_1_HAT_IMAGE_PATH,
                x_px: 5,
                y_px: -1,
                height_px: 4,
            },
            _ => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_SMALL_STANDARD_HAT_IMAGE_PATH,
                x_px: 7,
                y_px: 2,
                height_px: 2,
            },
        }
    } else {
        match hat_id {
            2 => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_BIG_TYROLEAN_HAT_IMAGE_PATH,
                x_px: -2,
                y_px: -3,
                height_px: 5,
            },
            3 => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_BIG_TOWERING_1_HAT_IMAGE_PATH,
                x_px: -2,
                y_px: -2,
                height_px: 5,
            },
            _ => LegacyRuntimePlayerRenderHatConfig {
                image_path: LEGACY_RUNTIME_BIG_STANDARD_HAT_IMAGE_PATH,
                x_px: 0,
                y_px: 0,
                height_px: 4,
            },
        }
    }
}

const fn legacy_runtime_player_render_hat_offsets(
    size: u8,
    animation_state: PlayerAnimationState,
    ducking: bool,
    run_frame: u8,
    swim_frame: u8,
    fire_animation_active: bool,
) -> Option<(i32, i32)> {
    if size == 1 {
        match animation_state {
            PlayerAnimationState::Idle => Some((0, 0)),
            PlayerAnimationState::Running | PlayerAnimationState::Falling => {
                legacy_runtime_player_render_small_running_hat_offset(run_frame)
            }
            PlayerAnimationState::Sliding => Some((0, 0)),
            PlayerAnimationState::Jumping => Some((0, -1)),
            PlayerAnimationState::Swimming => {
                legacy_runtime_player_render_small_swimming_hat_offset(swim_frame)
            }
            PlayerAnimationState::Climbing => {
                legacy_runtime_player_render_small_climbing_hat_offset(swim_frame)
            }
            PlayerAnimationState::Dead => None,
        }
    } else if fire_animation_active {
        Some((-5, -4))
    } else if ducking {
        Some((-5, -12))
    } else {
        match animation_state {
            PlayerAnimationState::Idle => Some((-4, -2)),
            PlayerAnimationState::Running | PlayerAnimationState::Falling => {
                legacy_runtime_player_render_big_running_hat_offset(run_frame)
            }
            PlayerAnimationState::Sliding => Some((-5, -2)),
            PlayerAnimationState::Jumping => Some((-4, -4)),
            PlayerAnimationState::Swimming => {
                legacy_runtime_player_render_big_swimming_hat_offset(swim_frame)
            }
            PlayerAnimationState::Climbing => Some((-4, -4)),
            PlayerAnimationState::Dead => None,
        }
    }
}

const fn legacy_runtime_player_render_small_running_hat_offset(
    run_frame: u8,
) -> Option<(i32, i32)> {
    match run_frame {
        3 => Some((-1, -1)),
        _ => Some((0, 0)),
    }
}

const fn legacy_runtime_player_render_big_running_hat_offset(run_frame: u8) -> Option<(i32, i32)> {
    match run_frame {
        1 => Some((-5, -4)),
        2 => Some((-4, -3)),
        _ => Some((-3, -2)),
    }
}

const fn legacy_runtime_player_render_small_swimming_hat_offset(
    _swim_frame: u8,
) -> Option<(i32, i32)> {
    Some((1, -1))
}

const fn legacy_runtime_player_render_big_swimming_hat_offset(
    _swim_frame: u8,
) -> Option<(i32, i32)> {
    Some((-5, -4))
}

const fn legacy_runtime_player_render_small_climbing_hat_offset(
    swim_frame: u8,
) -> Option<(i32, i32)> {
    match swim_frame {
        2 => Some((2, -1)),
        _ => Some((2, 0)),
    }
}

const fn legacy_runtime_player_quad_center_x(size: u8) -> i32 {
    if size == 1 { 11 } else { 9 }
}

const fn legacy_runtime_player_quad_center_y(size: u8) -> i32 {
    if size == 1 { 10 } else { 20 }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LegacyRuntimePlayerRenderLayerGeometry {
    quad: LegacyRuntimePlayerRenderQuad,
    draw_x_px: f32,
    draw_y_px: f32,
    scale: f32,
    direction_scale: LegacyRuntimePlayerRenderDirectionScale,
}

const fn legacy_runtime_player_render_color_layer(
    size: u8,
    power_up: LegacyRuntimePlayerPowerUp,
    graphic_layer_index: u8,
    draw_order: u8,
    geometry: LegacyRuntimePlayerRenderLayerGeometry,
) -> LegacyRuntimePlayerRenderColorLayerPreview {
    let (tint, tint_source) =
        legacy_runtime_player_render_layer_tint(power_up, graphic_layer_index);
    LegacyRuntimePlayerRenderColorLayerPreview {
        draw_order,
        graphic_layer_index,
        image_path: legacy_runtime_player_render_layer_image_path(size, graphic_layer_index),
        tint,
        tint_source,
        quad: geometry.quad,
        draw_x_px: geometry.draw_x_px,
        draw_y_px: geometry.draw_y_px,
        rotation: 0.0,
        scale: geometry.scale,
        direction_scale: geometry.direction_scale,
        live_rendering_executed: false,
    }
}

const fn legacy_runtime_player_render_layer_tint(
    power_up: LegacyRuntimePlayerPowerUp,
    graphic_layer_index: u8,
) -> (
    LegacyRuntimePlayerRenderTint,
    LegacyRuntimePlayerRenderTintSource,
) {
    if graphic_layer_index == 0 {
        return (
            LegacyRuntimePlayerRenderTint {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            LegacyRuntimePlayerRenderTintSource::White,
        );
    }

    if matches!(power_up, LegacyRuntimePlayerPowerUp::Fire) {
        return (
            legacy_runtime_player_flower_tint(graphic_layer_index),
            LegacyRuntimePlayerRenderTintSource::FlowerColor,
        );
    }

    (
        legacy_runtime_player_one_tint(graphic_layer_index),
        LegacyRuntimePlayerRenderTintSource::PlayerColor,
    )
}

const fn legacy_runtime_player_one_tint(layer_index: u8) -> LegacyRuntimePlayerRenderTint {
    match layer_index {
        1 => LegacyRuntimePlayerRenderTint {
            r: 224.0 / 255.0,
            g: 32.0 / 255.0,
            b: 0.0,
        },
        2 => LegacyRuntimePlayerRenderTint {
            r: 136.0 / 255.0,
            g: 112.0 / 255.0,
            b: 0.0,
        },
        _ => LegacyRuntimePlayerRenderTint {
            r: 252.0 / 255.0,
            g: 152.0 / 255.0,
            b: 56.0 / 255.0,
        },
    }
}

const fn legacy_runtime_player_flower_tint(layer_index: u8) -> LegacyRuntimePlayerRenderTint {
    match layer_index {
        1 => LegacyRuntimePlayerRenderTint {
            r: 252.0 / 255.0,
            g: 216.0 / 255.0,
            b: 168.0 / 255.0,
        },
        2 => LegacyRuntimePlayerRenderTint {
            r: 216.0 / 255.0,
            g: 40.0 / 255.0,
            b: 0.0,
        },
        _ => LegacyRuntimePlayerRenderTint {
            r: 252.0 / 255.0,
            g: 152.0 / 255.0,
            b: 56.0 / 255.0,
        },
    }
}

const fn legacy_runtime_player_small_quad(x_px: u32) -> LegacyRuntimePlayerRenderQuad {
    const DEFAULT_ANGLE_FRAME: u32 = 3;
    LegacyRuntimePlayerRenderQuad {
        x_px,
        y_px: (DEFAULT_ANGLE_FRAME - 1) * 20,
        width_px: 20,
        height_px: 20,
        atlas_width_px: 512,
        atlas_height_px: 128,
    }
}

const fn legacy_runtime_player_big_quad(
    x_px: u32,
    row_index: u32,
) -> LegacyRuntimePlayerRenderQuad {
    LegacyRuntimePlayerRenderQuad {
        x_px,
        y_px: row_index * 36,
        width_px: 20,
        height_px: 36,
        atlas_width_px: 512,
        atlas_height_px: 256,
    }
}

const fn legacy_runtime_player_offset_x(_size: u8) -> f32 {
    6.0
}

const fn legacy_runtime_player_offset_y(size: u8) -> f32 {
    if size == 1 { 3.0 } else { -3.0 }
}

const fn legacy_runtime_fireball_render_frame_kind(
    frame: LegacyFireballFrame,
) -> LegacyRuntimeFireballRenderFrameKind {
    match frame {
        LegacyFireballFrame::FlyingOne
        | LegacyFireballFrame::FlyingTwo
        | LegacyFireballFrame::FlyingThree
        | LegacyFireballFrame::FlyingFour => LegacyRuntimeFireballRenderFrameKind::Flying,
        LegacyFireballFrame::ExplosionOne
        | LegacyFireballFrame::ExplosionTwo
        | LegacyFireballFrame::ExplosionThree => LegacyRuntimeFireballRenderFrameKind::Explosion,
    }
}

const fn legacy_runtime_fireball_render_quad(
    frame: LegacyFireballFrame,
) -> LegacyRuntimeFireballRenderQuad {
    match frame {
        LegacyFireballFrame::FlyingOne => LegacyRuntimeFireballRenderQuad {
            x_px: 0,
            y_px: 0,
            width_px: 8,
            height_px: 8,
        },
        LegacyFireballFrame::FlyingTwo => LegacyRuntimeFireballRenderQuad {
            x_px: 8,
            y_px: 0,
            width_px: 8,
            height_px: 8,
        },
        LegacyFireballFrame::FlyingThree => LegacyRuntimeFireballRenderQuad {
            x_px: 16,
            y_px: 0,
            width_px: 8,
            height_px: 8,
        },
        LegacyFireballFrame::FlyingFour => LegacyRuntimeFireballRenderQuad {
            x_px: 24,
            y_px: 0,
            width_px: 8,
            height_px: 8,
        },
        LegacyFireballFrame::ExplosionOne => LegacyRuntimeFireballRenderQuad {
            x_px: 32,
            y_px: 0,
            width_px: 16,
            height_px: 16,
        },
        LegacyFireballFrame::ExplosionTwo => LegacyRuntimeFireballRenderQuad {
            x_px: 48,
            y_px: 0,
            width_px: 16,
            height_px: 16,
        },
        LegacyFireballFrame::ExplosionThree => LegacyRuntimeFireballRenderQuad {
            x_px: 64,
            y_px: 0,
            width_px: 16,
            height_px: 16,
        },
    }
}

fn probe_legacy_runtime_fireball_map_targets(
    projectiles: &[LegacyFireballState],
    dt: f32,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> LegacyRuntimeFireballMapTargetProbeReport {
    let mut reports = Vec::new();
    let bounds = map.bounds();

    for (index, projectile) in projectiles.iter().copied().enumerate() {
        if !projectile.active {
            continue;
        }

        let moving = CollisionBody::new(
            projectile.x,
            projectile.y,
            projectile.width,
            projectile.height,
            projectile.speed_x,
            projectile.speed_y,
        );

        for coord in legacy_fireball_tile_collision_candidates(projectile, dt, bounds) {
            let Some(tile) = map.tile_metadata_at(coord) else {
                continue;
            };
            if !tile.collides() || tile.invisible() {
                continue;
            }

            let kind = collision_kind(moving, legacy_runtime_tile_bounds(coord), dt, false);
            let Some(axis) = legacy_runtime_fireball_collision_axis(projectile, kind) else {
                continue;
            };

            reports.push(LegacyRuntimeFireballMapTargetProbe {
                projectile_index: index,
                state: projectile,
                coord,
                tile_id: tile.tile_id,
                axis,
                target: LegacyFireballCollisionTarget::Tile,
                collides: tile.collides(),
                invisible: tile.invisible(),
                breakable: tile.breakable(),
                coin_block: tile.coin_block(),
                play_block_hit_sound: !matches!(axis, LegacyRuntimeFireballCollisionAxis::Floor),
                live_projectile_collision_mutated: false,
            });
            break;
        }
    }

    LegacyRuntimeFireballMapTargetProbeReport { reports }
}

fn legacy_fireball_tile_collision_candidates(
    projectile: LegacyFireballState,
    dt: f32,
    bounds: LegacyMapBounds,
) -> Vec<LegacyMapTileCoord> {
    let x_start = (projectile.x + projectile.speed_x * dt - 2.0 / 16.0).floor() as i32 + 1;
    let y_start = (projectile.y + projectile.speed_y * dt - 2.0 / 16.0).floor() as i32 + 1;
    let x_end = x_start + projectile.width.ceil() as i32;
    let y_end = y_start + projectile.height.ceil() as i32;

    let mut coords = Vec::new();
    if projectile.speed_x < 0.0 {
        for x in (x_start..=x_end).rev() {
            push_candidate_column(&mut coords, bounds, x, y_start, y_end);
        }
    } else {
        for x in x_start..=x_end {
            push_candidate_column(&mut coords, bounds, x, y_start, y_end);
        }
    }

    coords
}

const fn legacy_runtime_fireball_collision_axis(
    projectile: LegacyFireballState,
    kind: CollisionKind,
) -> Option<LegacyRuntimeFireballCollisionAxis> {
    match kind {
        CollisionKind::Passive => Some(LegacyRuntimeFireballCollisionAxis::Passive),
        CollisionKind::Horizontal => {
            if projectile.speed_x < 0.0 {
                Some(LegacyRuntimeFireballCollisionAxis::Left)
            } else {
                Some(LegacyRuntimeFireballCollisionAxis::Right)
            }
        }
        CollisionKind::Vertical => {
            if projectile.speed_y < 0.0 {
                Some(LegacyRuntimeFireballCollisionAxis::Ceiling)
            } else {
                Some(LegacyRuntimeFireballCollisionAxis::Floor)
            }
        }
        CollisionKind::None => None,
    }
}

fn probe_legacy_runtime_fireball_collisions(
    projectiles: &[LegacyFireballState],
    request: Option<LegacyRuntimeFireballCollisionProbeRequest>,
    map_target_probes: &LegacyRuntimeFireballMapTargetProbeReport,
    enemies: &[LegacyRuntimeFireballEnemySnapshot],
    projected_enemy_hits: &LegacyRuntimeProjectedFireballEnemyHitState,
    dt: f32,
) -> LegacyRuntimeFireballCollisionProbeReport {
    let mut reports = Vec::new();

    let explicit_request = request.is_some();
    if let Some(request) = request {
        if let Some(projectile) = projectiles.get(request.projectile_index).copied() {
            reports.push(probe_legacy_runtime_fireball_collision(
                request.projectile_index,
                LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                projectile,
                request.axis,
                request.target,
            ));
        }
    }

    if !explicit_request {
        reports.extend(probe_legacy_runtime_fireball_enemy_overlaps(
            projectiles,
            enemies,
            projected_enemy_hits,
            dt,
        ));
    }

    reports.extend(map_target_probes.reports.iter().copied().map(|probe| {
        probe_legacy_runtime_fireball_collision(
            probe.projectile_index,
            LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe {
                coord: probe.coord,
                tile_id: probe.tile_id,
            },
            probe.state,
            probe.axis,
            probe.target,
        )
    }));

    let release_summaries = reports
        .iter()
        .copied()
        .filter_map(legacy_runtime_fireball_collision_release_summary)
        .collect();

    LegacyRuntimeFireballCollisionProbeReport {
        reports,
        release_summaries,
    }
}

fn probe_legacy_runtime_fireball_collision(
    projectile_index: usize,
    source: LegacyRuntimeFireballCollisionProbeSource,
    projectile: LegacyFireballState,
    axis: LegacyRuntimeFireballCollisionAxis,
    target: LegacyFireballCollisionTarget,
) -> LegacyRuntimeFireballCollisionProbe {
    let mut state_after = projectile;
    let constants = LegacyFireballConstants::default();
    let outcome = match axis {
        LegacyRuntimeFireballCollisionAxis::Left => {
            legacy_fireball_left_collision(&mut state_after, constants, target)
        }
        LegacyRuntimeFireballCollisionAxis::Right => {
            legacy_fireball_right_collision(&mut state_after, constants, target)
        }
        LegacyRuntimeFireballCollisionAxis::Floor => {
            legacy_fireball_floor_collision(&mut state_after, constants, target)
        }
        LegacyRuntimeFireballCollisionAxis::Ceiling => {
            legacy_fireball_ceil_collision(&mut state_after, target)
        }
        LegacyRuntimeFireballCollisionAxis::Passive => {
            legacy_fireball_passive_collision(&mut state_after, target)
        }
    };

    LegacyRuntimeFireballCollisionProbe {
        projectile_index,
        source,
        axis,
        target,
        state_before: projectile,
        state_after,
        outcome,
    }
}

fn probe_legacy_runtime_fireball_enemy_overlaps(
    projectiles: &[LegacyFireballState],
    enemies: &[LegacyRuntimeFireballEnemySnapshot],
    projected_enemy_hits: &LegacyRuntimeProjectedFireballEnemyHitState,
    dt: f32,
) -> Vec<LegacyRuntimeFireballCollisionProbe> {
    let mut reports = Vec::new();

    for (projectile_index, projectile) in projectiles.iter().copied().enumerate() {
        if !projectile.active {
            continue;
        }

        let moving = CollisionBody::new(
            projectile.x,
            projectile.y,
            projectile.width,
            projectile.height,
            projectile.speed_x,
            projectile.speed_y,
        );

        for enemy in enemies.iter().copied() {
            if !enemy.has_shotted_handler || projected_enemy_hits.contains_removed_enemy(enemy) {
                continue;
            }

            let kind = collision_kind(
                moving,
                Aabb::from_xywh(enemy.x, enemy.y, enemy.width, enemy.height),
                dt,
                false,
            );
            let Some(axis) = legacy_runtime_fireball_collision_axis(projectile, kind) else {
                continue;
            };

            reports.push(probe_legacy_runtime_fireball_collision(
                projectile_index,
                LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe {
                    enemy_index: enemy.index,
                },
                projectile,
                axis,
                enemy.target,
            ));
            break;
        }
    }

    reports
}

fn legacy_runtime_fireball_enemy_hit_intents(
    probes: &LegacyRuntimeFireballCollisionProbeReport,
    enemies: &[LegacyRuntimeFireballEnemySnapshot],
    projected_enemy_hits: &LegacyRuntimeProjectedFireballEnemyHitState,
) -> Vec<LegacyRuntimeFireballEnemyHitIntent> {
    probes
        .reports
        .iter()
        .copied()
        .filter_map(|probe| {
            if !matches!(
                probe.source,
                LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest
                    | LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { .. }
            ) {
                return None;
            }

            let shot_direction = probe.outcome.shoot_target?;
            let enemy = enemies.iter().copied().find(|enemy| {
                enemy.target == probe.target
                    && enemy.has_shotted_handler
                    && !projected_enemy_hits.contains_removed_enemy(*enemy)
            })?;

            Some(LegacyRuntimeFireballEnemyHitIntent {
                projectile_index: probe.projectile_index,
                source: probe.source,
                axis: probe.axis,
                target: probe.target,
                enemy,
                shot_direction,
                score_delta: probe.outcome.points,
                score_x: probe.state_after.x,
                score_y: probe.state_after.y,
                live_enemy_mutated: false,
            })
        })
        .collect()
}

fn legacy_runtime_fireball_probe_score_suppressed_by_projected_enemy_hit(
    probe: LegacyRuntimeFireballCollisionProbe,
    enemies: &[LegacyRuntimeFireballEnemySnapshot],
    projected_enemy_hits: &LegacyRuntimeProjectedFireballEnemyHitState,
) -> bool {
    if !matches!(
        probe.source,
        LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest
            | LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { .. }
    ) || probe.outcome.shoot_target.is_none()
    {
        return false;
    }

    let mut matched_seeded_enemy = false;
    for enemy in enemies
        .iter()
        .copied()
        .filter(|enemy| enemy.target == probe.target && enemy.has_shotted_handler)
    {
        matched_seeded_enemy = true;
        if !projected_enemy_hits.contains_removed_enemy(enemy) {
            return false;
        }
    }

    matched_seeded_enemy
}

const fn legacy_runtime_fireball_collision_release_summary(
    probe: LegacyRuntimeFireballCollisionProbe,
) -> Option<LegacyRuntimeFireballProjectileReleaseSummary> {
    if !probe.outcome.released_thrower {
        return None;
    }

    Some(LegacyRuntimeFireballProjectileReleaseSummary {
        projectile_index: probe.projectile_index,
        source: LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
            source: probe.source,
            axis: probe.axis,
            target: probe.target,
        },
        callback: LegacyRuntimeFireballCallbackMetadata {
            callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
            fireball_count_delta: -1,
        },
        live_projectile_queue_mutated: false,
        live_fireball_counter_mutated: false,
    })
}

fn project_legacy_runtime_fireball_projectile_collisions(
    state: &mut LegacyRuntimeProjectedFireballProjectileCollisionState,
    probes: &LegacyRuntimeFireballCollisionProbeReport,
) -> Vec<LegacyRuntimeProjectedFireballProjectileCollisionSnapshot> {
    let mut projected_indices = Vec::new();
    let mut snapshots = Vec::new();

    for probe in probes.reports.iter().copied() {
        if projected_indices.contains(&probe.projectile_index) {
            continue;
        }
        projected_indices.push(probe.projectile_index);
        snapshots.push(state.apply_probe(probe));
    }

    snapshots
}

const fn legacy_runtime_fireball_projectile_update_release_summary(
    projectile_index: usize,
    update: LegacyFireballUpdate,
) -> Option<LegacyRuntimeFireballProjectileReleaseSummary> {
    if !update.released_thrower {
        return None;
    }

    Some(LegacyRuntimeFireballProjectileReleaseSummary {
        projectile_index,
        source: LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate,
        callback: LegacyRuntimeFireballCallbackMetadata {
            callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
            fireball_count_delta: -1,
        },
        live_projectile_queue_mutated: false,
        live_fireball_counter_mutated: false,
    })
}

fn project_legacy_runtime_fireball_count_from_releases(
    state: &mut LegacyRuntimeProjectedFireballCountState,
    fallback_active_fireball_count: usize,
    summaries: &[LegacyRuntimeFireballProjectileReleaseSummary],
) -> Vec<LegacyRuntimeProjectedFireballCountSnapshot> {
    summaries
        .iter()
        .copied()
        .map(|summary| state.apply_release_summary(summary, fallback_active_fireball_count))
        .collect()
}

const fn apply_legacy_runtime_fireball_count_delta(count: usize, delta: i32) -> usize {
    if delta >= 0 {
        count.saturating_add(delta as usize)
    } else {
        count.saturating_sub(delta.unsigned_abs() as usize)
    }
}

fn legacy_coord_to_zero_based(coord: LegacyMapTileCoord) -> Option<(usize, usize)> {
    if coord.x < 1 || coord.y < 1 {
        return None;
    }

    Some((
        usize::try_from(coord.x - 1).ok()?,
        usize::try_from(coord.y - 1).ok()?,
    ))
}

const fn legacy_map_coord_to_tile_coord(coord: LegacyMapTileCoord) -> TileCoord {
    TileCoord::new(coord.x, coord.y)
}

const fn legacy_tile_coord_to_map_coord(coord: TileCoord) -> LegacyMapTileCoord {
    LegacyMapTileCoord::new(coord.x, coord.y)
}

fn legacy_level_block_spriteset(level: &Mari0Level) -> LegacyBlockSpriteset {
    level
        .properties
        .spriteset
        .and_then(|index| u8::try_from(index).ok())
        .map_or(
            LegacyBlockSpriteset::One,
            LegacyBlockSpriteset::from_legacy_index,
        )
}

const fn legacy_reveal_sound_effect(sound: LegacyBlockRevealSound) -> LegacySoundEffect {
    match sound {
        LegacyBlockRevealSound::MushroomAppear => LegacySoundEffect::MushroomAppear,
        LegacyBlockRevealSound::Vine => LegacySoundEffect::Vine,
    }
}

const fn legacy_block_bounce_content_from_entity(
    kind: LegacyEntityKind,
) -> Option<LegacyBlockBounceContentKind> {
    match kind {
        LegacyEntityKind::PowerUp(LegacyPowerUpEntity::Mushroom) => {
            Some(LegacyBlockBounceContentKind::Mushroom)
        }
        LegacyEntityKind::PowerUp(LegacyPowerUpEntity::OneUp) => {
            Some(LegacyBlockBounceContentKind::OneUp)
        }
        LegacyEntityKind::PowerUp(LegacyPowerUpEntity::Star) => {
            Some(LegacyBlockBounceContentKind::Star)
        }
        LegacyEntityKind::PowerUp(LegacyPowerUpEntity::ManyCoins) => {
            Some(LegacyBlockBounceContentKind::ManyCoins)
        }
        LegacyEntityKind::Warp(LegacyWarpEntity::Vine) => Some(LegacyBlockBounceContentKind::Vine),
        _ => None,
    }
}

fn legacy_tile_collision_candidates(
    body: PlayerBodyBounds,
    movement: &PlayerMovementState,
    dt: f32,
    bounds: LegacyMapBounds,
) -> Vec<LegacyMapTileCoord> {
    let x_start = (body.x + movement.speed_x * dt - 2.0 / 16.0).floor() as i32 + 1;
    let y_start = (body.y + movement.speed_y * dt - 2.0 / 16.0).floor() as i32 + 1;
    let x_end = x_start + body.width.ceil() as i32;
    let y_end = y_start + body.height.ceil() as i32;

    let mut coords = Vec::new();
    if movement.speed_x < 0.0 {
        for x in (x_start..=x_end).rev() {
            push_candidate_column(&mut coords, bounds, x, y_start, y_end);
        }
    } else {
        for x in x_start..=x_end {
            push_candidate_column(&mut coords, bounds, x, y_start, y_end);
        }
    }

    coords
}

fn push_candidate_column(
    coords: &mut Vec<LegacyMapTileCoord>,
    bounds: LegacyMapBounds,
    x: i32,
    y_start: i32,
    y_end: i32,
) {
    for y in y_start..=y_end {
        let coord = LegacyMapTileCoord::new(x, y);
        if bounds.contains(coord) {
            coords.push(coord);
        }
    }
}

fn legacy_player_collision_actor(player: &LegacyRuntimePlayer) -> LegacyCollisionActor {
    LegacyCollisionActor::new(
        player.body.x,
        player.body.y,
        player.body.width,
        player.body.height,
        player.movement.speed_x,
        player.movement.speed_y,
    )
}

fn legacy_runtime_tile_bounds(coord: LegacyMapTileCoord) -> Aabb {
    Aabb::from_xywh((coord.x - 1) as f32, (coord.y - 1) as f32, 1.0, 1.0)
}

fn legacy_player_body_from_aabb(bounds: Aabb) -> PlayerBodyBounds {
    PlayerBodyBounds::new(bounds.min.x, bounds.min.y, bounds.width(), bounds.height())
}

fn legacy_runtime_portal_target_probe(
    player: LegacyRuntimePlayer,
    player_source: LegacyRuntimePortalTargetPlayerSource,
    aim: LegacyRuntimePortalAimSnapshot,
    query: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> Option<LegacyRuntimePortalTargetProbe> {
    if !aim.active() {
        return None;
    }

    let source_x = player.body.x + 6.0 / 16.0;
    let source_y = player.body.y + 6.0 / 16.0;
    let trace_hit = legacy_runtime_trace_portal_line(source_x, source_y, aim.pointing_angle, query);
    let placement = trace_hit.and_then(|hit| {
        legacy_runtime_portal_position(query, hit.coord, hit.side, hit.tendency).map(|coord| {
            LegacyRuntimePortalPlacement {
                coord,
                side: hit.side,
            }
        })
    });

    Some(LegacyRuntimePortalTargetProbe {
        player_source,
        source_x,
        source_y,
        pointing_angle: aim.pointing_angle,
        requested_slot: aim.requested_slot(),
        trace_hit,
        placement,
    })
}

fn legacy_runtime_portal_aim_render_intent_preview(
    probe: Option<LegacyRuntimePortalTargetProbe>,
    aim: LegacyRuntimePortalAimSnapshot,
    render: LegacyRuntimeRenderContext,
) -> Option<LegacyRuntimePortalAimRenderIntentPreview> {
    let probe = probe?;
    let trace_hit = probe.trace_hit;
    let portal_possible = probe.portal_possible();
    let color = legacy_runtime_portal_aim_color(portal_possible, 1.0);
    let (target_x, target_y) = trace_hit
        .map(|hit| (hit.impact_x, hit.impact_y))
        .unwrap_or((probe.source_x, probe.source_y));
    let distance_tiles =
        ((target_x - probe.source_x).powi(2) + (target_y - probe.source_y).powi(2)).sqrt();
    let dot_draws = trace_hit
        .map(|_| {
            legacy_runtime_portal_aim_dot_previews(
                probe.source_x,
                probe.source_y,
                target_x,
                target_y,
                aim.portal_dots_timer,
                portal_possible,
                render,
            )
        })
        .unwrap_or_default();
    let crosshair = trace_hit.map(|hit| {
        let x_px = (target_x - render.xscroll) * 16.0 * render.scale;
        let y_px = (target_y - 0.5) * 16.0 * render.scale;
        LegacyRuntimePortalAimCrosshairPreview {
            x_px,
            y_px,
            draw_x_px: x_px.floor(),
            draw_y_px: y_px.floor(),
            rotation: legacy_runtime_portal_crosshair_rotation(hit.side),
            origin_x_px: 4,
            origin_y_px: 8,
            color,
            image_path: LEGACY_RUNTIME_PORTAL_CROSSHAIR_IMAGE_PATH,
            scale: render.scale,
            live_rendering_executed: false,
        }
    });

    Some(LegacyRuntimePortalAimRenderIntentPreview {
        player_source: probe.player_source,
        source_x: probe.source_x,
        source_y: probe.source_y,
        pointing_angle: probe.pointing_angle,
        requested_slot: probe.requested_slot,
        trace_hit,
        placement: probe.placement,
        portal_possible,
        target_x,
        target_y,
        distance_tiles,
        dots_timer: aim.portal_dots_timer,
        dots_time: LEGACY_RUNTIME_PORTAL_DOT_TIME,
        dots_distance_tiles: LEGACY_RUNTIME_PORTAL_DOT_DISTANCE_TILES,
        dots_inner_radius_px: LEGACY_RUNTIME_PORTAL_DOT_INNER_RADIUS_PX,
        dots_outer_radius_px: LEGACY_RUNTIME_PORTAL_DOT_OUTER_RADIUS_PX,
        dot_draws,
        crosshair,
        color_reset_after_dots: true,
        color_reset_after_crosshair: crosshair.is_some(),
        live_rendering_executed: false,
        live_portal_mutated: false,
    })
}

fn legacy_runtime_portal_aim_dot_previews(
    source_x: f32,
    source_y: f32,
    target_x: f32,
    target_y: f32,
    dots_timer: f32,
    portal_possible: bool,
    render: LegacyRuntimeRenderContext,
) -> Vec<LegacyRuntimePortalAimDotPreview> {
    let distance_tiles = ((target_x - source_x).powi(2) + (target_y - source_y).powi(2)).sqrt();
    let denominator = distance_tiles / LEGACY_RUNTIME_PORTAL_DOT_DISTANCE_TILES;
    if denominator <= 0.0 {
        return Vec::new();
    }

    let timer_phase = dots_timer / LEGACY_RUNTIME_PORTAL_DOT_TIME;
    let upper = denominator + 1.0;
    let mut dots = Vec::new();
    let mut sequence_index = 1_u32;
    while (sequence_index as f32) <= upper {
        let phase = (sequence_index as f32 - 1.0 + timer_phase) / denominator;
        if phase < 1.0 {
            let xplus = (target_x - source_x) * 16.0 * render.scale * phase;
            let yplus = (target_y - source_y) * 16.0 * render.scale * phase;
            let x_px = (source_x - render.xscroll) * 16.0 * render.scale + xplus;
            let y_px = (source_y - 0.5) * 16.0 * render.scale + yplus;
            let radius_px = (xplus.powi(2) + yplus.powi(2)).sqrt() / render.scale;
            let alpha = if radius_px < LEGACY_RUNTIME_PORTAL_DOT_OUTER_RADIUS_PX {
                ((radius_px - LEGACY_RUNTIME_PORTAL_DOT_INNER_RADIUS_PX)
                    * (LEGACY_RUNTIME_PORTAL_DOT_OUTER_RADIUS_PX
                        - LEGACY_RUNTIME_PORTAL_DOT_INNER_RADIUS_PX))
                    .max(0.0)
            } else {
                1.0
            };
            dots.push(LegacyRuntimePortalAimDotPreview {
                sequence_index,
                phase,
                x_px,
                y_px,
                draw_x_px: (x_px - 0.25 * render.scale).floor(),
                draw_y_px: (y_px - 0.25 * render.scale).floor(),
                radius_px,
                alpha,
                color: legacy_runtime_portal_aim_color(portal_possible, alpha),
                image_path: LEGACY_RUNTIME_PORTAL_DOT_IMAGE_PATH,
                scale: render.scale,
                live_rendering_executed: false,
            });
        }
        sequence_index += 1;
    }
    dots
}

const fn legacy_runtime_portal_aim_color(portal_possible: bool, alpha: f32) -> LegacyColor {
    if portal_possible {
        LegacyColor {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: alpha,
        }
    } else {
        LegacyColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: alpha,
        }
    }
}

const fn legacy_runtime_portal_crosshair_rotation(side: Facing) -> f32 {
    match side {
        Facing::Right => consts::FRAC_PI_2,
        Facing::Down => consts::PI,
        Facing::Left => consts::FRAC_PI_2 * 3.0,
        Facing::Up => 0.0,
    }
}

fn legacy_runtime_portal_transit_candidate_probe(
    player: LegacyRuntimePlayer,
    readiness: Option<LegacyRuntimePortalPairReadinessSummary>,
) -> Option<LegacyRuntimePortalTransitCandidateProbe> {
    let readiness = readiness?;
    if !readiness.ready {
        return None;
    }

    let center_x = player.body.x + player.body.width / 2.0;
    let center_y = player.body.y + player.body.height / 2.0;
    let center_coord =
        LegacyMapTileCoord::new(center_x.floor() as i32 + 1, center_y.floor() as i32 + 1);
    let candidate_pairing = [readiness.portal_1_to_2, readiness.portal_2_to_1]
        .into_iter()
        .flatten()
        .find(|pairing| legacy_runtime_projected_portal_contains_tile(pairing.entry, center_coord));
    let candidate_entry_tile = candidate_pairing.map(|_| center_coord);

    Some(LegacyRuntimePortalTransitCandidateProbe {
        center_x,
        center_y,
        center_coord,
        candidate_entry_tile,
        candidate_pairing,
    })
}

fn legacy_runtime_portalcoords_preview(
    player: LegacyRuntimePlayer,
    candidate: Option<LegacyRuntimePortalTransitCandidateProbe>,
    frame_dt: f32,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> Option<LegacyRuntimePortalCoordsPreviewReport> {
    const INPUT_ROTATION: f32 = 0.0;

    let pairing = candidate?.candidate_pairing?;
    let transit = legacy_portal_coords(LegacyPortalTransitInput {
        position: Vec2::new(player.body.x, player.body.y),
        velocity: Vec2::new(player.movement.speed_x, player.movement.speed_y),
        size: Vec2::new(player.body.width, player.body.height),
        rotation: INPUT_ROTATION,
        animation_direction: Some(legacy_runtime_wormhole_animation_direction(
            player.movement.animation_direction,
        )),
        entry: LegacyPortalEndpoint::new(
            pairing.entry.placement.coord.x as f32,
            pairing.entry.placement.coord.y as f32,
            pairing.entry.placement.side,
        ),
        exit: LegacyPortalEndpoint::new(
            pairing.exit.placement.coord.x as f32,
            pairing.exit.placement.coord.y as f32,
            pairing.exit.placement.side,
        ),
        live: true,
        gravity: player.movement.gravity,
        frame_dt,
    });
    let output_animation_direction = legacy_runtime_player_animation_direction(
        transit.animation_direction,
        player.movement.animation_direction,
    );
    let output_body = PlayerBodyBounds::new(
        transit.position.x,
        transit.position.y,
        player.body.width,
        player.body.height,
    );
    let blocked_exit_probe =
        legacy_runtime_portal_blocked_exit_probe(output_body, player, pairing, map);

    Some(LegacyRuntimePortalCoordsPreviewReport {
        entry_slot: pairing.entry_slot,
        exit_slot: pairing.exit_slot,
        entry_facing: pairing.entry.placement.side,
        exit_facing: pairing.exit.placement.side,
        input_body: player.body,
        input_speed_x: player.movement.speed_x,
        input_speed_y: player.movement.speed_y,
        input_rotation: INPUT_ROTATION,
        output_body,
        output_speed_x: transit.velocity.x,
        output_speed_y: transit.velocity.y,
        output_rotation: transit.rotation,
        output_animation_direction,
        exit_blocked: blocked_exit_probe.is_some(),
        blocked_exit_probe,
    })
}

fn legacy_runtime_portal_transit_outcome_summary(
    preview: Option<LegacyRuntimePortalCoordsPreviewReport>,
) -> Option<LegacyRuntimePortalTransitOutcomeSummary> {
    let preview = preview?;
    let kind = if preview.exit_blocked {
        LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview
    } else {
        LegacyRuntimePortalTransitOutcomeKind::TeleportPreview
    };

    Some(LegacyRuntimePortalTransitOutcomeSummary {
        kind,
        entry_slot: preview.entry_slot,
        exit_slot: preview.exit_slot,
        entry_facing: preview.entry_facing,
        exit_facing: preview.exit_facing,
        input_body: preview.input_body,
        output_body: preview.output_body,
        output_speed_x: preview.output_speed_x,
        output_speed_y: preview.output_speed_y,
        blocked_exit_probe: preview.blocked_exit_probe,
    })
}

fn legacy_runtime_portal_transit_audio_intent(
    summary: Option<LegacyRuntimePortalTransitOutcomeSummary>,
) -> Option<LegacyRuntimePortalTransitAudioIntent> {
    let summary = summary?;

    Some(LegacyRuntimePortalTransitAudioIntent {
        outcome_kind: summary.kind,
        entry_slot: summary.entry_slot,
        exit_slot: summary.exit_slot,
        sound: LegacySoundEffect::PortalEnter,
    })
}

fn legacy_runtime_projected_player_state_snapshot(
    player: LegacyRuntimePlayer,
    preview: Option<LegacyRuntimePortalCoordsPreviewReport>,
    summary: Option<LegacyRuntimePortalTransitOutcomeSummary>,
) -> Option<LegacyRuntimeProjectedPlayerStateSnapshot> {
    let preview = preview?;
    let summary = summary?;
    let (source, body, speed_x, speed_y, animation_direction) = match summary.kind {
        LegacyRuntimePortalTransitOutcomeKind::TeleportPreview => (
            LegacyRuntimeProjectedPlayerStateSource::PortalTransitTeleportPreview,
            summary.output_body,
            summary.output_speed_x,
            summary.output_speed_y,
            preview.output_animation_direction,
        ),
        LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview => {
            let blocked_exit = summary.blocked_exit_probe?;
            (
                LegacyRuntimeProjectedPlayerStateSource::PortalTransitBlockedExitBouncePreview,
                summary.input_body,
                blocked_exit.bounced_speed_x,
                blocked_exit.bounced_speed_y,
                player.movement.animation_direction,
            )
        }
    };

    Some(LegacyRuntimeProjectedPlayerStateSnapshot {
        source,
        entry_slot: summary.entry_slot,
        exit_slot: summary.exit_slot,
        entry_facing: summary.entry_facing,
        exit_facing: summary.exit_facing,
        body,
        speed_x,
        speed_y,
        animation_direction,
    })
}

fn legacy_runtime_portal_blocked_exit_probe(
    output_body: PlayerBodyBounds,
    player: LegacyRuntimePlayer,
    pairing: LegacyRuntimePortalPairing,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> Option<LegacyRuntimePortalBlockedExitProbe> {
    let blocking_coord = legacy_runtime_first_solid_overlap(output_body, map)?;
    let (bounce_axis, bounced_speed_x, bounced_speed_y) = match pairing.entry.placement.side {
        Facing::Up | Facing::Down => (
            LegacyRuntimePortalBlockedExitBounceAxis::Vertical,
            player.movement.speed_x,
            legacy_runtime_vertical_blocked_exit_bounce_speed(player.movement.speed_y),
        ),
        Facing::Left | Facing::Right => (
            LegacyRuntimePortalBlockedExitBounceAxis::Horizontal,
            -player.movement.speed_x,
            player.movement.speed_y,
        ),
    };

    Some(LegacyRuntimePortalBlockedExitProbe {
        blocking_coord,
        bounce_axis,
        bounced_speed_x,
        bounced_speed_y,
    })
}

fn legacy_runtime_first_solid_overlap(
    body: PlayerBodyBounds,
    map: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> Option<LegacyMapTileCoord> {
    let bounds = map.bounds();
    let body_bounds = Aabb::from_xywh(body.x, body.y, body.width, body.height);
    let x_start = body.x.floor() as i32 + 1;
    let x_end = (body.x + body.width).floor() as i32 + 1;
    let y_start = body.y.floor() as i32 + 1;
    let y_end = (body.y + body.height).floor() as i32 + 1;

    for x in x_start..=x_end {
        for y in y_start..=y_end {
            let coord = LegacyMapTileCoord::new(x, y);
            if !bounds.contains(coord) || !map.tile_collides_at(coord) {
                continue;
            }
            if body_bounds.intersects(legacy_runtime_tile_bounds(coord)) {
                return Some(coord);
            }
        }
    }

    None
}

fn legacy_runtime_vertical_blocked_exit_bounce_speed(speed_y: f32) -> f32 {
    let bounced = -speed_y * 0.95;
    if bounced.abs() < 2.0 {
        if bounced > 0.0 { 2.0 } else { -2.0 }
    } else {
        bounced
    }
}

fn legacy_runtime_wormhole_animation_direction(
    direction: HorizontalDirection,
) -> AnimationDirection {
    match direction {
        HorizontalDirection::Left => AnimationDirection::Left,
        HorizontalDirection::Right => AnimationDirection::Right,
    }
}

fn legacy_runtime_player_animation_direction(
    direction: Option<AnimationDirection>,
    fallback: HorizontalDirection,
) -> HorizontalDirection {
    match direction {
        Some(AnimationDirection::Left) => HorizontalDirection::Left,
        Some(AnimationDirection::Right) => HorizontalDirection::Right,
        None => fallback,
    }
}

fn legacy_runtime_projected_portal_contains_tile(
    portal: LegacyRuntimeProjectedPortal,
    coord: LegacyMapTileCoord,
) -> bool {
    portal.tile_reservations.contains(&coord)
}

fn legacy_runtime_trace_portal_line(
    source_x: f32,
    source_y: f32,
    radians: f32,
    query: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
) -> Option<LegacyRuntimePortalTraceHit> {
    let bounds = query.bounds();
    let mut x = source_x;
    let mut y = source_y;
    let mut current_x = x.floor() as i32;
    let mut current_y = (y + 1.0).floor() as i32;
    let source_floor_y = (source_y + 0.5).floor() as i32;

    while legacy_runtime_trace_block_in_bounds(current_x, current_y, source_floor_y, bounds) {
        let old_x = x;
        let old_y = y;
        let (y_diff, y_side) = if in_range(radians, -consts::FRAC_PI_2, consts::FRAC_PI_2, true) {
            y = current_y as f32 - 1.0;
            ((old_y - y) / radians.cos(), Facing::Down)
        } else {
            y = current_y as f32;
            ((old_y - y) / radians.cos(), Facing::Up)
        };
        let (x_diff, x_side) = if in_range(radians, 0.0, consts::PI, true) {
            x = current_x as f32;
            ((old_x - x) / radians.sin(), Facing::Right)
        } else {
            x = current_x as f32 + 1.0;
            ((old_x - x) / radians.sin(), Facing::Left)
        };

        let side = if x_diff < y_diff {
            y = old_y - radians.cos() * x_diff;
            x_side
        } else {
            x = old_x - radians.sin() * y_diff;
            y_side
        };

        match side {
            Facing::Down => current_y -= 1,
            Facing::Up => current_y += 1,
            Facing::Left => current_x += 1,
            Facing::Right => current_x -= 1,
        }

        let coord = LegacyMapTileCoord::new(current_x + 1, current_y);
        if query.tile_collides_at(coord) {
            let tendency = legacy_runtime_portal_tendency(side, x, y);
            return Some(LegacyRuntimePortalTraceHit {
                coord,
                side,
                tendency,
                impact_x: x,
                impact_y: y,
            });
        }
    }

    None
}

fn legacy_runtime_trace_block_in_bounds(
    current_x: i32,
    current_y: i32,
    source_floor_y: i32,
    bounds: LegacyMapBounds,
) -> bool {
    current_x + 1 > 0
        && current_x < bounds.width
        && (current_y > 0 || current_y >= source_floor_y)
        && current_y < bounds.height + 1
}

fn legacy_runtime_portal_tendency(side: Facing, impact_x: f32, impact_y: f32) -> i32 {
    match side {
        Facing::Up | Facing::Down => {
            if impact_x.rem_euclid(1.0) > 0.5 {
                1
            } else {
                -1
            }
        }
        Facing::Left | Facing::Right => {
            if impact_y.rem_euclid(1.0) > 0.5 {
                1
            } else {
                -1
            }
        }
    }
}

fn legacy_runtime_portal_position(
    query: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
    coord: LegacyMapTileCoord,
    side: Facing,
    tendency: i32,
) -> Option<LegacyMapTileCoord> {
    let (xplus, yplus) = legacy_runtime_portal_side_offset(side);

    if matches!(side, Facing::Up | Facing::Down) {
        if tendency == -1 {
            if legacy_runtime_portal_surface_open(query, coord.x - 1, coord.y, 0, yplus)
                && legacy_runtime_portal_surface_open(query, coord.x, coord.y, 0, yplus)
            {
                return Some(if side == Facing::Up {
                    LegacyMapTileCoord::new(coord.x - 1, coord.y)
                } else {
                    coord
                });
            }
            if legacy_runtime_portal_surface_open(query, coord.x, coord.y, 0, yplus)
                && legacy_runtime_portal_surface_open(query, coord.x + 1, coord.y, 0, yplus)
            {
                return Some(if side == Facing::Up {
                    coord
                } else {
                    LegacyMapTileCoord::new(coord.x + 1, coord.y)
                });
            }
        } else {
            if legacy_runtime_portal_surface_open(query, coord.x, coord.y, 0, yplus)
                && legacy_runtime_portal_surface_open(query, coord.x + 1, coord.y, 0, yplus)
            {
                return Some(if side == Facing::Up {
                    coord
                } else {
                    LegacyMapTileCoord::new(coord.x + 1, coord.y)
                });
            }
            if legacy_runtime_portal_surface_open(query, coord.x - 1, coord.y, 0, yplus)
                && legacy_runtime_portal_surface_open(query, coord.x, coord.y, 0, yplus)
            {
                return Some(if side == Facing::Up {
                    LegacyMapTileCoord::new(coord.x - 1, coord.y)
                } else {
                    coord
                });
            }
        }
    } else if tendency == -1 {
        if legacy_runtime_portal_surface_open(query, coord.x, coord.y - 1, xplus, 0)
            && legacy_runtime_portal_surface_open(query, coord.x, coord.y, xplus, 0)
        {
            return Some(if side == Facing::Right {
                LegacyMapTileCoord::new(coord.x, coord.y - 1)
            } else {
                coord
            });
        }
        if legacy_runtime_portal_surface_open(query, coord.x, coord.y, xplus, 0)
            && legacy_runtime_portal_surface_open(query, coord.x, coord.y + 1, xplus, 0)
        {
            return Some(if side == Facing::Right {
                coord
            } else {
                LegacyMapTileCoord::new(coord.x, coord.y + 1)
            });
        }
    } else {
        if legacy_runtime_portal_surface_open(query, coord.x, coord.y, xplus, 0)
            && legacy_runtime_portal_surface_open(query, coord.x, coord.y + 1, xplus, 0)
        {
            return Some(if side == Facing::Right {
                coord
            } else {
                LegacyMapTileCoord::new(coord.x, coord.y + 1)
            });
        }
        if legacy_runtime_portal_surface_open(query, coord.x, coord.y - 1, xplus, 0)
            && legacy_runtime_portal_surface_open(query, coord.x, coord.y, xplus, 0)
        {
            return Some(if side == Facing::Right {
                LegacyMapTileCoord::new(coord.x, coord.y - 1)
            } else {
                coord
            });
        }
    }

    None
}

fn legacy_runtime_portal_outcome_intent(
    probe: LegacyRuntimePortalTargetProbe,
) -> Option<LegacyRuntimePortalOutcomeIntent> {
    let requested_slot = probe.requested_slot?;
    let (kind, sound) = match (requested_slot, probe.placement) {
        (LegacyRuntimePortalSlot::Portal1, Some(_)) => (
            LegacyRuntimePortalOutcomeKind::Open,
            LegacySoundEffect::Portal1Open,
        ),
        (LegacyRuntimePortalSlot::Portal2, Some(_)) => (
            LegacyRuntimePortalOutcomeKind::Open,
            LegacySoundEffect::Portal2Open,
        ),
        (_, None) => (
            LegacyRuntimePortalOutcomeKind::Fizzle,
            LegacySoundEffect::PortalFizzle,
        ),
    };

    Some(LegacyRuntimePortalOutcomeIntent {
        requested_slot,
        kind,
        placement: probe.placement,
        sound,
    })
}

pub fn legacy_runtime_portal_reservation_projection(
    outcome: LegacyRuntimePortalOutcomeIntent,
) -> Option<LegacyRuntimePortalReservationProjection> {
    if outcome.kind != LegacyRuntimePortalOutcomeKind::Open {
        return None;
    }

    let placement = outcome.placement?;
    let x = placement.coord.x;
    let y = placement.coord.y;
    let (tile_reservations, wall_reservations) = match placement.side {
        Facing::Up => (
            [
                LegacyMapTileCoord::new(x, y),
                LegacyMapTileCoord::new(x + 1, y),
            ],
            [
                LegacyRuntimePortalWallReservation::new(x - 1, y, 2, 0),
                LegacyRuntimePortalWallReservation::new(x - 1, y - 1, 0, 1),
                LegacyRuntimePortalWallReservation::new(x + 1, y - 1, 0, 1),
            ],
        ),
        Facing::Down => (
            [
                LegacyMapTileCoord::new(x, y),
                LegacyMapTileCoord::new(x - 1, y),
            ],
            [
                LegacyRuntimePortalWallReservation::new(x - 2, y - 1, 2, 0),
                LegacyRuntimePortalWallReservation::new(x - 2, y - 1, 0, 1),
                LegacyRuntimePortalWallReservation::new(x, y - 1, 0, 1),
            ],
        ),
        Facing::Left => (
            [
                LegacyMapTileCoord::new(x, y),
                LegacyMapTileCoord::new(x, y - 1),
            ],
            [
                LegacyRuntimePortalWallReservation::new(x, y - 2, 0, 2),
                LegacyRuntimePortalWallReservation::new(x - 1, y - 2, 1, 0),
                LegacyRuntimePortalWallReservation::new(x - 1, y, 1, 0),
            ],
        ),
        Facing::Right => (
            [
                LegacyMapTileCoord::new(x, y),
                LegacyMapTileCoord::new(x, y + 1),
            ],
            [
                LegacyRuntimePortalWallReservation::new(x - 1, y - 1, 0, 2),
                LegacyRuntimePortalWallReservation::new(x - 1, y - 1, 1, 0),
                LegacyRuntimePortalWallReservation::new(x - 1, y + 1, 1, 0),
            ],
        ),
    };

    Some(LegacyRuntimePortalReservationProjection {
        requested_slot: outcome.requested_slot,
        placement,
        tile_reservations,
        wall_reservations,
    })
}

fn legacy_runtime_portal_block_reservation(
    placement: LegacyRuntimePortalPlacement,
) -> LegacyBlockPortalReservation {
    let facing = match placement.side {
        Facing::Up => Facing::Right,
        Facing::Right => Facing::Down,
        Facing::Down => Facing::Left,
        Facing::Left => Facing::Up,
    };
    LegacyBlockPortalReservation::new(TileCoord::new(placement.coord.x, placement.coord.y), facing)
}

fn legacy_runtime_portal_surface_open(
    query: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
    x: i32,
    y: i32,
    adjacent_x: i32,
    adjacent_y: i32,
) -> bool {
    legacy_runtime_get_tile_for_portal(query, x, y, true)
        && !legacy_runtime_get_tile_for_portal(query, x + adjacent_x, y + adjacent_y, false)
}

fn legacy_runtime_get_tile_for_portal(
    query: LegacyRuntimeProjectedTileMetadataMapQuery<'_, '_, '_>,
    x: i32,
    y: i32,
    portalable: bool,
) -> bool {
    let bounds = query.bounds();
    if x <= 0 || y <= 0 || y >= 16 || x > bounds.width {
        return false;
    }

    let Some(tile) = query.tile_metadata_at(LegacyMapTileCoord::new(x, y)) else {
        return false;
    };
    if tile.invisible() {
        return false;
    }

    if portalable {
        tile.solid_portalable()
    } else {
        tile.collides()
    }
}

fn legacy_runtime_portal_side_offset(side: Facing) -> (i32, i32) {
    match side {
        Facing::Up => (0, -1),
        Facing::Right => (1, 0),
        Facing::Down => (0, 1),
        Facing::Left => (-1, 0),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFrameRequest {
    pub raw_dt: f32,
    pub joystick_deadzone: f32,
    pub render: LegacyRuntimeRenderContext,
    pub sound: Option<LegacySoundEffect>,
    pub player_pointing_angle: f32,
    pub portal_aim: Option<LegacyRuntimePortalAimSnapshot>,
    pub fireball_launch: Option<LegacyRuntimeFireballLaunchSnapshot>,
    pub fireball_collision_probe: Option<LegacyRuntimeFireballCollisionProbeRequest>,
}

impl LegacyRuntimeFrameRequest {
    #[must_use]
    pub const fn new(
        raw_dt: f32,
        joystick_deadzone: f32,
        render: LegacyRuntimeRenderContext,
        sound: Option<LegacySoundEffect>,
    ) -> Self {
        Self {
            raw_dt,
            joystick_deadzone,
            render,
            sound,
            player_pointing_angle: LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE,
            portal_aim: None,
            fireball_launch: None,
            fireball_collision_probe: None,
        }
    }

    #[must_use]
    pub const fn with_player_pointing_angle(mut self, player_pointing_angle: f32) -> Self {
        self.player_pointing_angle = player_pointing_angle;
        self
    }

    #[must_use]
    pub const fn with_portal_aim(mut self, portal_aim: LegacyRuntimePortalAimSnapshot) -> Self {
        self.portal_aim = Some(portal_aim);
        self
    }

    #[must_use]
    pub const fn with_fireball_launch(
        mut self,
        fireball_launch: LegacyRuntimeFireballLaunchSnapshot,
    ) -> Self {
        self.fireball_launch = Some(fireball_launch);
        self
    }

    #[must_use]
    pub const fn with_fireball_collision_probe(
        mut self,
        fireball_collision_probe: LegacyRuntimeFireballCollisionProbeRequest,
    ) -> Self {
        self.fireball_collision_probe = Some(fireball_collision_probe);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeRenderContext {
    pub xscroll: f32,
    pub scale: f32,
}

impl LegacyRuntimeRenderContext {
    #[must_use]
    pub const fn new(xscroll: f32, scale: f32) -> Self {
        Self { xscroll, scale }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrame {
    pub frame_step: LegacyFrameStep,
    pub movement_input: PlayerMovementInput,
    pub background_color: Option<LegacyColor>,
    pub tile_batch_draws: Vec<LegacyTileBatchDrawIntent>,
    pub audio_commands: Vec<LegacyAudioCommand>,
}

#[derive(Debug)]
pub enum LegacyRuntimeLoadError {
    ReadSettings {
        path: LegacyAssetPath,
        source: io::Error,
    },
    ReadLevel {
        path: LegacyAssetPath,
        source: io::Error,
    },
    ParseLevel {
        path: LegacyAssetPath,
        source: ParseError,
    },
}

impl fmt::Display for LegacyRuntimeLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadSettings { path, .. } => {
                write!(
                    formatter,
                    "failed to read mappack settings at {}",
                    path.as_str()
                )
            }
            Self::ReadLevel { path, .. } => {
                write!(
                    formatter,
                    "failed to read legacy level at {}",
                    path.as_str()
                )
            }
            Self::ParseLevel { path, .. } => {
                write!(
                    formatter,
                    "failed to parse legacy level at {}",
                    path.as_str()
                )
            }
        }
    }
}

impl Error for LegacyRuntimeLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadSettings { source, .. } | Self::ReadLevel { source, .. } => Some(source),
            Self::ParseLevel { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE, LEGACY_RUNTIME_DOOR_CENTER_IMAGE_PATH,
        LEGACY_RUNTIME_DOOR_PIECE_IMAGE_PATH, LEGACY_RUNTIME_EMANCIPATION_GRILL_LINE_COLOR,
        LEGACY_RUNTIME_EMANCIPATION_GRILL_PARTICLE_IMAGE_PATH,
        LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
        LEGACY_RUNTIME_PORTAL_CROSSHAIR_IMAGE_PATH, LEGACY_RUNTIME_PORTAL_DOT_DISTANCE_TILES,
        LEGACY_RUNTIME_PORTAL_DOT_IMAGE_PATH, LEGACY_RUNTIME_PORTAL_DOT_TIME,
        LEGACY_RUNTIME_PORTAL_PROJECTILE_IMAGE_PATH,
        LEGACY_RUNTIME_PORTAL_PROJECTILE_PARTICLE_IMAGE_PATH,
        LEGACY_RUNTIME_WALL_INDICATOR_IMAGE_PATH, LegacyRuntimeBlockBounceItemSpawnIntent,
        LegacyRuntimeBlockContainedRewardRevealIntent, LegacyRuntimeBlockDebrisAnimationState,
        LegacyRuntimeBlockEnemyShotIntent, LegacyRuntimeBlockItemJumpIntent,
        LegacyRuntimeBlockJumpItemSnapshot, LegacyRuntimeBlockTopCoinCollectionIntent,
        LegacyRuntimeBlockTopEnemySnapshot, LegacyRuntimeBreakableBlockCleanupAction,
        LegacyRuntimeBreakableBlockCleanupProjection, LegacyRuntimeBreakableBlockCleanupSource,
        LegacyRuntimeCoinBlockRewardIntent, LegacyRuntimeCoinCounterIntent,
        LegacyRuntimeCoinCounterSource, LegacyRuntimeDoorPartKind, LegacyRuntimeDoorSnapshot,
        LegacyRuntimeEmancipationGrillLinePreview, LegacyRuntimeEmancipationGrillParticleDirection,
        LegacyRuntimeEmancipationGrillParticleSnapshot,
        LegacyRuntimeEmancipationGrillScissorPreview, LegacyRuntimeEmancipationGrillSnapshot,
        LegacyRuntimeEmptyBreakableBlockDestroyIntent, LegacyRuntimeFireballCallback,
        LegacyRuntimeFireballCallbackMetadata, LegacyRuntimeFireballCollisionAxis,
        LegacyRuntimeFireballCollisionProbeRequest, LegacyRuntimeFireballCollisionProbeSource,
        LegacyRuntimeFireballEnemyHitIntent, LegacyRuntimeFireballEnemySnapshot,
        LegacyRuntimeFireballLaunchSnapshot, LegacyRuntimeFireballMapTargetProbeReport,
        LegacyRuntimeFireballProjectileReleaseSource,
        LegacyRuntimeFireballProjectileReleaseSummary, LegacyRuntimeFireballRenderFrameKind,
        LegacyRuntimeFireballRenderQuad, LegacyRuntimeFireballRenderSource,
        LegacyRuntimeFrameRequest, LegacyRuntimeLevelSelection,
        LegacyRuntimeManyCoinsTimerStartReport, LegacyRuntimeManyCoinsTimerUpdateReport,
        LegacyRuntimePlayer, LegacyRuntimePlayerCollisionAxis, LegacyRuntimePlayerPowerUp,
        LegacyRuntimePlayerRenderDirectionScaleSource, LegacyRuntimePlayerRenderFrame,
        LegacyRuntimePlayerRenderHatSize, LegacyRuntimePlayerRenderQuad,
        LegacyRuntimePlayerRenderScissorPreview, LegacyRuntimePlayerRenderTintSource,
        LegacyRuntimePortalAimSnapshot, LegacyRuntimePortalBlockedExitBounceAxis,
        LegacyRuntimePortalBlockedExitProbe, LegacyRuntimePortalCoordsPreviewReport,
        LegacyRuntimePortalOutcomeIntent, LegacyRuntimePortalOutcomeKind,
        LegacyRuntimePortalPairReadinessSummary, LegacyRuntimePortalPairing,
        LegacyRuntimePortalPlacement, LegacyRuntimePortalProjectileParticleSnapshot,
        LegacyRuntimePortalProjectileSnapshot, LegacyRuntimePortalReplacementSummary,
        LegacyRuntimePortalReservationProjection, LegacyRuntimePortalSlot,
        LegacyRuntimePortalTargetPlayerSource, LegacyRuntimePortalTransitAudioIntent,
        LegacyRuntimePortalTransitCandidateProbe, LegacyRuntimePortalTransitOutcomeKind,
        LegacyRuntimePortalTransitOutcomeSummary, LegacyRuntimePortalWallReservation,
        LegacyRuntimeProjectedFireballCountSource, LegacyRuntimeProjectedFireballEnemyHitSnapshot,
        LegacyRuntimeProjectedFireballEnemyHitState, LegacyRuntimeProjectedPlayerStateSnapshot,
        LegacyRuntimeProjectedPlayerStateSource, LegacyRuntimeProjectedPortal,
        LegacyRuntimeProjectedPortalState, LegacyRuntimeRenderContext,
        LegacyRuntimeScoreCounterIntent, LegacyRuntimeScoreSource,
        LegacyRuntimeScrollingScoreAnimationState, LegacyRuntimeShell,
        LegacyRuntimeTileChangeProjection, LegacyRuntimeTileChangeSource,
        LegacyRuntimeWallIndicatorSnapshot, legacy_runtime_player_render_intent_preview,
        probe_legacy_runtime_fireball_collisions,
    };
    use crate::{
        assets::{BufferedLegacyAssetSource, LEGACY_PORTAL_BACKGROUND_PATH},
        audio::{LegacyAudioCommand, LegacySoundEffect},
        input::{BufferedLegacyInputSnapshot, LegacyControlBinding, LegacyPlayerControls},
        map::LegacyMapQuery,
        render::{LegacyColor, LegacyTileAtlas},
        tiles::{LegacyTileMetadata, LegacyTileMetadataTable},
        time::LEGACY_MAX_UPDATE_DT,
    };
    use iw2wth_core::{
        Facing, HorizontalDirection, LEGACY_BLOCK_BOUNCE_DURATION, LEGACY_BLOCK_BOUNCE_TIMER_START,
        LegacyBlockBounceContentKind, LegacyBlockBounceReplayKind, LegacyBlockBounceReplaySpawn,
        LegacyBlockBounceSchedule, LegacyBlockBounceSpawnKind,
        LegacyBlockContainedRewardRevealOutcome, LegacyBlockDebrisState,
        LegacyBlockEnemyShotRequest, LegacyBlockItemJumpRequest, LegacyBlockJumpItemKind,
        LegacyBlockPortalReservation, LegacyBlockRevealSound, LegacyBlockTopCoinCollectionOutcome,
        LegacyBreakableBlockOutcome, LegacyBrokenBlockEffects, LegacyCoinBlockAnimationScore,
        LegacyCoinBlockAnimationState, LegacyCoinBlockRewardConstants,
        LegacyCoinBlockRewardOutcome, LegacyCoinBlockTimerSpawn, LegacyCoinLifeReward,
        LegacyEnemyDirection, LegacyFireballCollisionOutcome, LegacyFireballCollisionTarget,
        LegacyFireballFrame, LegacyFireballUpdate, LegacyManyCoinsTimerEntry, LegacyMapTileCoord,
        LegacyScrollingScoreLabel, LegacyScrollingScoreState, LegacyTileChange,
        PlayerAnimationState, PlayerBodyBounds, PlayerMovementInput, PlayerMovementState,
        TileCoord, TileId,
    };
    use std::f32::consts;

    const FIREBALL_ENEMY_OVERLAP_FIXTURES: &str =
        include_str!("../tests/fixtures/legacy_fireball_enemy_overlaps.generated.tsv");

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {actual} to be close to {expected}",
        );
    }

    fn level_source(cells: &[&str], properties: &str) -> String {
        format!("{};{properties}", cells.join(","))
    }

    fn flat_level_cells(width: usize) -> Vec<&'static str> {
        vec!["1"; width * iw2wth_core::content::MARI0_LEVEL_HEIGHT]
    }

    fn player_controls() -> LegacyPlayerControls {
        LegacyPlayerControls::new(
            LegacyControlBinding::keyboard("left"),
            LegacyControlBinding::keyboard("right"),
            LegacyControlBinding::keyboard("run"),
        )
    }

    fn test_tiles() -> LegacyTileMetadataTable {
        LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                coin: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                invisible: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                breakable: true,
                coin_block: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                breakable: true,
                ..LegacyTileMetadata::empty()
            },
        ])
    }

    fn test_tiles_with_used_block() -> LegacyTileMetadataTable {
        let mut tiles = vec![LegacyTileMetadata::empty(); 113];
        tiles[1] = LegacyTileMetadata {
            collision: true,
            ..LegacyTileMetadata::empty()
        };
        tiles[2] = LegacyTileMetadata {
            coin: true,
            ..LegacyTileMetadata::empty()
        };
        tiles[3] = LegacyTileMetadata {
            collision: true,
            invisible: true,
            ..LegacyTileMetadata::empty()
        };
        tiles[4] = LegacyTileMetadata {
            collision: true,
            breakable: true,
            coin_block: true,
            ..LegacyTileMetadata::empty()
        };
        tiles[5] = LegacyTileMetadata {
            collision: true,
            breakable: true,
            ..LegacyTileMetadata::empty()
        };
        tiles[112] = LegacyTileMetadata {
            collision: true,
            ..LegacyTileMetadata::empty()
        };
        LegacyTileMetadataTable::from_metadata_for_tests(tiles)
    }

    fn loaded_test_shell(cells: &[&str]) -> LegacyRuntimeShell {
        loaded_test_shell_with_properties(cells, "background=1")
    }

    fn loaded_test_shell_with_properties(cells: &[&str], properties: &str) -> LegacyRuntimeShell {
        let level = level_source(cells, properties);
        let source = BufferedLegacyAssetSource::new()
            .with_file_contents("mappacks/test/settings.txt", "name=Test\n")
            .with_file_contents("mappacks/test/1-1.txt", level);

        match LegacyRuntimeShell::load(
            &source,
            LegacyRuntimeLevelSelection::new("test", "1-1", 1, 1, 0),
            player_controls(),
        ) {
            Ok(shell) => shell,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn shell_loads_one_level_through_legacy_asset_paths() {
        let mut cells = flat_level_cells(2);
        cells[0] = "7";
        cells[1] = "8";
        let level = level_source(&cells, "background=1;spriteset=1;music=2");
        let source = BufferedLegacyAssetSource::new()
            .with_file_contents(
                "mappacks/smb/settings.txt",
                "name=Super Mario Bros.\nlives=3\n",
            )
            .with_file_contents("mappacks/smb/1-1.txt", level)
            .with_file("mappacks/smb/tiles.png")
            .with_file("mappacks/smb/music.mp3");

        let shell = LegacyRuntimeShell::load(
            &source,
            LegacyRuntimeLevelSelection::new("smb", "1-1", 1, 1, 0),
            player_controls(),
        );

        let shell = match shell {
            Ok(shell) => shell,
            Err(error) => panic!("{error}"),
        };
        assert_eq!(shell.settings.name.as_deref(), Some("Super Mario Bros."));
        assert_eq!(shell.level.width(), 2);
        assert!(shell.custom_tiles);
        assert_eq!(
            shell.custom_music_path.as_ref().map(|path| path.as_str()),
            Some("mappacks/smb/music.mp3"),
        );
        assert_eq!(
            shell
                .background_paths
                .iter()
                .map(|path| path.as_str())
                .collect::<Vec<_>>(),
            vec![LEGACY_PORTAL_BACKGROUND_PATH],
        );

        let query = shell.map_query();
        assert_eq!(
            query.tile_id_at(LegacyMapTileCoord::new(1, 1)),
            Some(TileId(7))
        );
        assert_eq!(
            query.tile_id_at(LegacyMapTileCoord::new(2, 1)),
            Some(TileId(8))
        );
    }

    #[test]
    fn shell_frame_composes_time_input_render_and_audio_intents() {
        let cells = flat_level_cells(1);
        let level = level_source(&cells, "background=3");
        let source = BufferedLegacyAssetSource::new()
            .with_file_contents("mappacks/test/settings.txt", "name=Test\n")
            .with_file_contents("mappacks/test/1-1.txt", level)
            .with_file("mappacks/test/tiles.png");
        let mut shell = match LegacyRuntimeShell::load(
            &source,
            LegacyRuntimeLevelSelection::new("test", "1-1", 1, 1, 0),
            player_controls(),
        ) {
            Ok(shell) => shell,
            Err(error) => panic!("{error}"),
        };
        let input = BufferedLegacyInputSnapshot::new()
            .with_keyboard_key("left")
            .with_keyboard_key("run");

        let frame = shell.step_frame(
            0.25,
            &input,
            0.2,
            LegacyRuntimeRenderContext::new(2.5, 2.0),
            Some(LegacySoundEffect::Coin),
        );

        assert!(frame.frame_step.should_update);
        assert_eq!(
            frame.movement_input,
            PlayerMovementInput::new(true, false, true),
        );
        assert_eq!(
            frame.background_color,
            Some(LegacyColor::rgb(32.0 / 255.0, 56.0 / 255.0, 236.0 / 255.0)),
        );
        assert_eq!(frame.tile_batch_draws.len(), 3);
        assert_eq!(frame.tile_batch_draws[2].atlas, LegacyTileAtlas::Custom);
        assert_eq!(frame.tile_batch_draws[0].x_px, -16.0);
        assert_eq!(
            frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
    }

    #[test]
    fn shell_reports_player_render_intent_preview_without_live_rendering_or_player_mutation() {
        let cells = flat_level_cells(4);
        let mut shell = loaded_test_shell(&cells);
        let input = BufferedLegacyInputSnapshot::new();
        let movement = PlayerMovementState {
            ducking: true,
            run_frame: 2,
            swim_frame: 1,
            animation_state: PlayerAnimationState::Running,
            animation_direction: HorizontalDirection::Left,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 24.0 / 16.0),
            movement,
        )
        .with_power_up(LegacyRuntimePlayerPowerUp::Fire)
        .with_fire_animation_timer(0.05);

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(1.25, 2.0),
                None,
            ),
            &test_tiles(),
        );

        let preview = frame.player_render_preview;
        assert_eq!(preview.player_index, 0);
        assert_eq!(preview.player, frame.player);
        assert_eq!(preview.body, frame.player.body);
        assert_eq!(preview.facing, HorizontalDirection::Left);
        assert_eq!(preview.animation_state, PlayerAnimationState::Falling);
        assert_eq!(
            preview.render_frame,
            LegacyRuntimePlayerRenderFrame::BigDuck
        );
        assert_eq!(preview.run_frame, 1);
        assert_eq!(preview.swim_frame, 1);
        assert_eq!(preview.size, 3);
        assert_eq!(preview.power_up, LegacyRuntimePlayerPowerUp::Fire);
        assert!(preview.ducking);
        assert_close(preview.fire_animation_timer, 0.05);
        assert!(preview.fire_animation_active);
        assert_eq!(
            preview.image_path,
            "graphics/SMB/player/bigmarioanimations.png"
        );
        assert_eq!(
            preview.quad,
            LegacyRuntimePlayerRenderQuad {
                x_px: 260,
                y_px: 72,
                width_px: 20,
                height_px: 36,
                atlas_width_px: 512,
                atlas_height_px: 256,
            },
        );
        assert_eq!(
            preview.color_layers.map(|layer| (
                layer.draw_order,
                layer.graphic_layer_index,
                layer.image_path
            )),
            [
                (0, 1, "graphics/SMB/player/bigmarioanimations1.png",),
                (1, 2, "graphics/SMB/player/bigmarioanimations2.png",),
                (2, 3, "graphics/SMB/player/bigmarioanimations3.png",),
                (3, 0, "graphics/SMB/player/bigmarioanimations0.png",),
            ],
        );
        assert_eq!(
            preview.color_layers.map(|layer| layer.tint_source),
            [
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::White,
            ],
        );
        assert_close(preview.color_layers[0].tint.r, 252.0 / 255.0);
        assert_close(preview.color_layers[0].tint.g, 216.0 / 255.0);
        assert_close(preview.color_layers[0].tint.b, 168.0 / 255.0);
        assert_close(preview.color_layers[3].tint.r, 1.0);
        assert_close(preview.color_layers[3].tint.g, 1.0);
        assert_close(preview.color_layers[3].tint.b, 1.0);
        assert!(
            preview
                .color_layers
                .iter()
                .all(|layer| layer.quad == preview.quad
                    && layer.draw_x_px == preview.draw_x_px
                    && layer.draw_y_px == preview.draw_y_px
                    && layer.direction_scale == preview.direction_scale
                    && !layer.live_rendering_executed)
        );
        assert_eq!(preview.hat_draw_count, 1);
        assert_eq!(preview.hat_draws[0].draw_order, 0);
        assert_eq!(preview.hat_draws[0].hat_slot_index, 0);
        assert_eq!(preview.hat_draws[0].hat_id, 1);
        assert_eq!(
            preview.hat_draws[0].size,
            LegacyRuntimePlayerRenderHatSize::Big,
        );
        assert_eq!(
            preview.hat_draws[0].image_path,
            "graphics/SMB/bighats/standard.png",
        );
        assert_eq!(preview.hat_draws[0].hat_config_x_px, 0);
        assert_eq!(preview.hat_draws[0].hat_config_y_px, 0);
        assert_eq!(preview.hat_draws[0].hat_height_px, 4);
        assert_eq!(preview.hat_draws[0].offset_x_px, -5);
        assert_eq!(preview.hat_draws[0].offset_y_px, -4);
        assert_eq!(preview.hat_draws[0].stack_y_px, 0);
        assert_eq!(preview.hat_draws[0].follows_graphic_layer_index, 3);
        assert_eq!(preview.hat_draws[0].precedes_graphic_layer_index, 0);
        assert_close(preview.hat_draws[0].draw_x_px, preview.draw_x_px);
        assert_close(preview.hat_draws[0].draw_y_px, preview.draw_y_px);
        assert_eq!(preview.hat_draws[0].origin_x_px, 4);
        assert_eq!(preview.hat_draws[0].origin_y_px, 16);
        assert_eq!(
            preview.direction_scale.source,
            LegacyRuntimePlayerRenderDirectionScaleSource::PlayerPointingAngle,
        );
        assert_eq!(
            preview.direction_scale.animation_facing,
            HorizontalDirection::Left,
        );
        assert_close(
            preview.direction_scale.pointing_angle,
            LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE,
        );
        assert_close(preview.direction_scale.direction_scale, 2.0);
        assert_close(preview.direction_scale.vertical_scale, 2.0);
        assert_eq!(
            preview.hat_draws[0].direction_scale,
            preview.direction_scale
        );
        assert_eq!(
            preview.hat_draws[0].tint_source,
            LegacyRuntimePlayerRenderTintSource::FlowerColor,
        );
        assert_close(preview.hat_draws[0].tint.r, 252.0 / 255.0);
        assert_close(preview.hat_draws[0].tint.g, 216.0 / 255.0);
        assert_close(preview.hat_draws[0].tint.b, 168.0 / 255.0);
        assert!(!preview.hat_draws[0].live_rendering_executed);
        assert_close(preview.draw_x_px, 36.0);
        assert_close(preview.draw_y_px, 102.0);
        assert!(!preview.live_rendering_executed);
        assert!(!preview.live_player_mutated);
        assert_eq!(player, frame.player);
    }

    #[test]
    fn shell_reports_player_render_portal_clone_preview_without_live_rendering_or_player_mutation()
    {
        let cells = flat_level_cells(10);
        let mut shell = loaded_test_shell(&cells);
        let portal_1_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(5, 6),
                side: Facing::Right,
            },
            tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(5, 7)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(4, 5, 0, 2),
                LegacyRuntimePortalWallReservation::new(4, 5, 1, 0),
                LegacyRuntimePortalWallReservation::new(4, 7, 1, 0),
            ],
        };
        let portal_2_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal2,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(9, 4),
                side: Facing::Right,
            },
            tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
            ],
        };
        shell
            .projected_portal_state
            .apply_projection(portal_1_projection);
        shell
            .projected_portal_state
            .apply_projection(portal_2_projection);
        let movement = PlayerMovementState {
            animation_direction: HorizontalDirection::Left,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(3.875, 4.875, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );
        let original_player = player;

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 2.0),
                None,
            )
            .with_player_pointing_angle(0.25),
            &test_tiles(),
        );

        let clone = frame
            .player_render_preview
            .portal_clone
            .expect("player center is inside a ready portal pair");
        assert_eq!(clone.entry_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(clone.exit_slot, LegacyRuntimePortalSlot::Portal2);
        assert_eq!(clone.entry_facing, Facing::Right);
        assert_eq!(clone.exit_facing, Facing::Right);
        assert_eq!(
            clone.entry_scissor,
            LegacyRuntimePlayerRenderScissorPreview {
                x_px: 160.0,
                y_px: 80.0,
                width_px: 128.0,
                height_px: 192.0,
            },
        );
        assert_eq!(
            clone.exit_scissor,
            LegacyRuntimePlayerRenderScissorPreview {
                x_px: 288.0,
                y_px: 16.0,
                width_px: 128.0,
                height_px: 192.0,
            },
        );
        assert_eq!(clone.input_body, original_player.body);
        assert_eq!(
            clone.output_body,
            PlayerBodyBounds::new(9.375, 2.875, 12.0 / 16.0, 12.0 / 16.0),
        );
        assert_close(clone.input_rotation, 0.0);
        assert_close(clone.output_rotation, 0.0);
        assert_eq!(clone.input_animation_direction, HorizontalDirection::Left);
        assert_eq!(clone.output_animation_direction, HorizontalDirection::Right);
        assert!(clone.animation_direction_flipped);
        assert_close(clone.draw_x_px, 312.0);
        assert_close(clone.draw_y_px, 86.0);
        assert_eq!(
            clone.direction_scale.source,
            LegacyRuntimePlayerRenderDirectionScaleSource::PortalCloneAnimationDirection,
        );
        assert_close(clone.direction_scale.pointing_angle, 0.25);
        assert_eq!(
            clone.direction_scale.animation_facing,
            HorizontalDirection::Right,
        );
        assert_close(clone.direction_scale.direction_scale, 2.0);
        assert_close(clone.direction_scale.vertical_scale, 2.0);
        assert!(clone.scissor_reset_to_current);
        assert!(!clone.live_rendering_executed);
        assert!(!clone.live_player_mutated);
        assert_eq!(player.body, original_player.body);
        assert_eq!(frame.player.body, original_player.body);
    }

    #[test]
    fn player_render_preview_preserves_legacy_frame_selection_matrix() {
        let render = LegacyRuntimeRenderContext::new(0.0, 1.0);
        let body = PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0);
        let cases = [
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Running,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallRun,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 60,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Falling,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallRun,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 60,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Idle,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallIdle,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 0,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Sliding,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallSlide,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 80,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Jumping,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallJump,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 100,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Swimming,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallSwim,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 180,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Climbing,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallClimb,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 140,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Small,
                PlayerAnimationState::Dead,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::SmallDead,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 120,
                    y_px: 40,
                    width_px: 20,
                    height_px: 20,
                    atlas_width_px: 512,
                    atlas_height_px: 128,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Running,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigRun,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 60,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Falling,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigRun,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 60,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Idle,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigIdle,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 0,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Sliding,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigSlide,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 80,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Jumping,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigJump,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 100,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Swimming,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigSwim,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 180,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Climbing,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigClimb,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 140,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Dead,
                false,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigJump,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 100,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Big,
                PlayerAnimationState::Running,
                true,
                0.2,
                LegacyRuntimePlayerRenderFrame::BigDuck,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 260,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
            (
                LegacyRuntimePlayerPowerUp::Fire,
                PlayerAnimationState::Idle,
                false,
                0.05,
                LegacyRuntimePlayerRenderFrame::BigFire,
                LegacyRuntimePlayerRenderQuad {
                    x_px: 120,
                    y_px: 72,
                    width_px: 20,
                    height_px: 36,
                    atlas_width_px: 512,
                    atlas_height_px: 256,
                },
            ),
        ];

        for (power_up, animation_state, ducking, fire_timer, expected_frame, expected_quad) in cases
        {
            let movement = PlayerMovementState {
                animation_state,
                ducking,
                ..PlayerMovementState::default()
            };
            let player = LegacyRuntimePlayer::new(body, movement)
                .with_power_up(power_up)
                .with_fire_animation_timer(fire_timer);

            let preview = legacy_runtime_player_render_intent_preview(
                player,
                render,
                LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE,
                None,
            );

            assert_eq!(preview.render_frame, expected_frame);
            assert_eq!(preview.quad, expected_quad);
            assert_eq!(preview.animation_state, animation_state);
            assert_eq!(preview.power_up, power_up);
            assert_eq!(preview.size, power_up.legacy_size());
            assert_eq!(preview.ducking, ducking);
            assert_eq!(
                preview.fire_animation_active,
                fire_timer < 0.11 && preview.size > 1
            );
            let expected_layer_prefix = if preview.size == 1 {
                "graphics/SMB/player/marioanimations"
            } else {
                "graphics/SMB/player/bigmarioanimations"
            };
            assert_eq!(
                preview
                    .color_layers
                    .map(|layer| (layer.draw_order, layer.graphic_layer_index)),
                [(0, 1), (1, 2), (2, 3), (3, 0)],
            );
            assert_eq!(
                preview.color_layers[0].image_path,
                format!("{expected_layer_prefix}1.png"),
            );
            assert_eq!(
                preview.color_layers[3].image_path,
                format!("{expected_layer_prefix}0.png"),
            );
            assert_eq!(
                preview.hat_draw_count,
                if matches!(animation_state, PlayerAnimationState::Dead) {
                    0
                } else {
                    1
                },
            );
            if preview.hat_draw_count == 1 {
                assert_eq!(preview.hat_draws[0].follows_graphic_layer_index, 3);
                assert_eq!(preview.hat_draws[0].precedes_graphic_layer_index, 0);
                assert!(preview.hat_draws[0].drawn);
                assert_eq!(preview.hat_draws[0].hat_id, 1);
            }
            assert!(!preview.live_rendering_executed);
            assert!(!preview.live_player_mutated);
            assert_eq!(preview.player, player);
        }
    }

    #[test]
    fn player_render_hat_previews_preserve_small_hat_stack_and_tint_rules() {
        let render = LegacyRuntimeRenderContext::new(1.0, 1.0);
        let movement = PlayerMovementState {
            animation_state: PlayerAnimationState::Running,
            animation_direction: HorizontalDirection::Right,
            run_frame: 3,
            ..PlayerMovementState::default()
        };
        let player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        )
        .with_hat_slots([1, 2, 0, 0], 2);

        let preview = legacy_runtime_player_render_intent_preview(
            player,
            render,
            LEGACY_RUNTIME_DEFAULT_PLAYER_POINTING_ANGLE,
            None,
        );

        assert_eq!(preview.hat_draw_count, 2);
        assert_eq!(
            preview.hat_draws.map(|hat| (
                hat.drawn,
                hat.draw_order,
                hat.hat_slot_index,
                hat.hat_id,
                hat.hat_config_x_px,
                hat.hat_config_y_px,
                hat.hat_height_px,
                hat.follows_graphic_layer_index,
                hat.precedes_graphic_layer_index,
            )),
            [
                (true, 0, 0, 1, 7, 2, 2, 3, 0),
                (true, 1, 1, 2, 5, -3, 4, 3, 0),
                (false, 0, 0, 0, 0, 0, 0, 3, 0),
                (false, 0, 0, 0, 0, 0, 0, 3, 0),
            ],
        );
        assert_eq!(
            preview.hat_draws.map(|hat| hat.image_path),
            [
                "graphics/SMB/hats/standard.png",
                "graphics/SMB/hats/tyrolean.png",
                "",
                "",
            ],
        );
        assert_eq!(
            preview
                .hat_draws
                .map(|hat| (hat.offset_x_px, hat.offset_y_px, hat.stack_y_px)),
            [(-1, -1, 0), (-1, -1, 2), (0, 0, 0), (0, 0, 0)],
        );
        assert_eq!(
            preview
                .hat_draws
                .map(|hat| (hat.origin_x_px, hat.origin_y_px)),
            [(3, 7), (5, 14), (0, 0), (0, 0)],
        );
        assert_eq!(
            preview.hat_draws[0].tint_source,
            LegacyRuntimePlayerRenderTintSource::PlayerColor,
        );
        assert_eq!(
            preview.hat_draws[1].tint_source,
            LegacyRuntimePlayerRenderTintSource::White,
        );
        assert_close(preview.hat_draws[0].direction_scale.direction_scale, 1.0);
        assert_close(preview.hat_draws[1].direction_scale.direction_scale, 1.0);
        assert_eq!(
            preview.hat_draws[0].direction_scale,
            preview.direction_scale
        );
        assert_eq!(
            preview.hat_draws[1].direction_scale,
            preview.direction_scale
        );
        assert!(!preview.hat_draws[0].live_rendering_executed);
        assert!(!preview.hat_draws[1].live_rendering_executed);
        assert_eq!(preview.player, player);
    }

    #[test]
    fn shell_reports_fireball_launch_intent_without_live_projectile_execution() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(-0.1)
                    .with_flower_power(true)
                    .with_active_fireball_count(1),
            ),
            &test_tiles(),
        );

        let launch = frame
            .fireball_launch_intent
            .expect("flower Mario fire request should report a fireball launch");
        assert_eq!(launch.direction, LegacyEnemyDirection::Right);
        assert_close(launch.source_x, 2.5);
        assert_close(launch.source_y, 3.0);
        assert_close(launch.spawn.x, 2.5 + 6.0 / 16.0);
        assert_close(launch.spawn.y, 3.0 + 4.0 / 16.0);
        assert_close(launch.spawn.speed_x, 15.0);
        assert_eq!(launch.spawn.speed_y, 0.0);
        assert_eq!(launch.spawn.frame, LegacyFireballFrame::FlyingOne);
        assert_eq!(launch.fireball_count_before, 1);
        assert_eq!(launch.fireball_count_after, 2);
        assert_eq!(launch.fire_animation_timer_reset, 0.0);
        assert_eq!(launch.sound, LegacySoundEffect::Fireball);
        assert_eq!(frame.fireball_projectile_progress.reports.len(), 1);
        let progress = frame.fireball_projectile_progress.reports[0];
        assert_eq!(progress.index, 0);
        assert_eq!(progress.state_before, launch.spawn);
        assert_eq!(
            progress.update,
            LegacyFireballUpdate {
                remove: false,
                released_thrower: false,
            },
        );
        assert_eq!(progress.state_after.frame, LegacyFireballFrame::FlyingOne);
        assert_close(progress.state_after.timer, LEGACY_MAX_UPDATE_DT);
        assert_eq!(frame.fireball_projectile_progress.queue_len_after_prune, 1);
        assert_eq!(shell.fireball_projectiles, vec![progress.state_after]);
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Fireball),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Fireball),
            ],
        );
    }

    #[test]
    fn shell_suppresses_fireball_launch_when_lua_fire_guards_fail() {
        let cells = flat_level_cells(2);
        let input = BufferedLegacyInputSnapshot::new();
        let tiles = test_tiles();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let mut no_flower_shell = loaded_test_shell(&cells);
        let no_flower = no_flower_shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(LegacyRuntimeFireballLaunchSnapshot::new(0.1)),
            &tiles,
        );
        assert!(no_flower.fireball_launch_intent.is_none());
        assert!(no_flower.frame.audio_commands.is_empty());

        let mut max_count_shell = loaded_test_shell(&cells);
        let max_count = max_count_shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(0.1)
                    .with_flower_power(true)
                    .with_active_fireball_count(2),
            ),
            &tiles,
        );
        assert!(max_count.fireball_launch_intent.is_none());
        assert!(max_count.frame.audio_commands.is_empty());
    }

    #[test]
    fn shell_reports_fireball_projectile_animation_and_offscreen_prune_without_collisions() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let mut animated_fireball = iw2wth_core::LegacyFireballState::spawn(
            1.0,
            4.0,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        animated_fireball.timer = iw2wth_core::LegacyFireballConstants::default().animation_delay;
        shell.fireball_projectiles.push(animated_fireball);
        shell
            .fireball_projectiles
            .push(iw2wth_core::LegacyFireballState::spawn(
                -2.0,
                4.0,
                LegacyEnemyDirection::Left,
                iw2wth_core::LegacyFireballConstants::default(),
            ));
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                0.05,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.fireball_projectile_progress.reports.len(), 2);
        let flying = frame.fireball_projectile_progress.reports[0];
        assert_eq!(flying.state_before.frame, LegacyFireballFrame::FlyingOne);
        assert_eq!(flying.state_after.frame, LegacyFireballFrame::FlyingTwo);
        assert_close(flying.state_after.timer, LEGACY_MAX_UPDATE_DT);
        assert_eq!(
            flying.update,
            LegacyFireballUpdate {
                remove: false,
                released_thrower: false,
            },
        );
        let offscreen = frame.fireball_projectile_progress.reports[1];
        assert_eq!(
            offscreen.update,
            LegacyFireballUpdate {
                remove: true,
                released_thrower: true,
            },
        );
        assert!(offscreen.state_after.destroy);
        assert_eq!(
            frame.fireball_projectile_progress.release_summaries,
            vec![LegacyRuntimeFireballProjectileReleaseSummary {
                projectile_index: 1,
                source: LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate,
                callback: LegacyRuntimeFireballCallbackMetadata {
                    callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
                    fireball_count_delta: -1,
                },
                live_projectile_queue_mutated: false,
                live_fireball_counter_mutated: false,
            }],
            "offscreen fireball removal reports Lua fireballcallback metadata without mutating live player counters",
        );
        assert_eq!(frame.fireball_count_projections.len(), 1);
        assert_eq!(
            frame.fireball_count_projections[0].source,
            LegacyRuntimeProjectedFireballCountSource::ProjectileUpdateReleaseSummary {
                projectile_index: 1,
            },
        );
        assert_eq!(
            frame.fireball_count_projections[0].active_fireball_count_before,
            2
        );
        assert_eq!(frame.fireball_count_projections[0].fireball_count_delta, -1);
        assert_eq!(
            frame.fireball_count_projections[0].active_fireball_count_after,
            1
        );
        assert!(!frame.fireball_count_projections[0].live_fireball_counter_mutated);
        assert_eq!(frame.fireball_projectile_progress.queue_len_after_prune, 1);
        assert_eq!(shell.fireball_projectiles, vec![flying.state_after]);
        assert_eq!(frame.fireball_render_previews.previews.len(), 1);
        let preview = frame.fireball_render_previews.previews[0];
        assert_eq!(preview.projectile_index, 0);
        assert_eq!(
            preview.source,
            LegacyRuntimeFireballRenderSource::LiveProjectile
        );
        assert_eq!(preview.state, flying.state_after);
        assert_eq!(preview.frame, LegacyFireballFrame::FlyingTwo);
        assert_eq!(
            preview.frame_kind,
            LegacyRuntimeFireballRenderFrameKind::Flying
        );
        assert_eq!(
            preview.quad,
            LegacyRuntimeFireballRenderQuad {
                x_px: 8,
                y_px: 0,
                width_px: 8,
                height_px: 8,
            },
        );
        assert!(!preview.live_rendering_executed);
        assert!(!preview.live_projectile_queue_mutated);
        assert_eq!(
            frame
                .fireball_render_previews
                .suppressed_projected_removal_indices,
            Vec::<usize>::new(),
        );
        assert!(
            frame.collisions.block_hits.is_empty(),
            "fireball progression reports animation/removal only; projectile collisions stay Lua-owned",
        );
    }

    #[test]
    fn shell_reports_portal_projectile_render_preview_without_physics_or_portal_mutation() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let snapshot = LegacyRuntimePortalProjectileSnapshot::new(
            4.25,
            5.5,
            0.25,
            1.0,
            LegacyColor {
                r: 0.0,
                g: 0.4,
                b: 1.0,
                a: 1.0,
            },
        )
        .with_particle(LegacyRuntimePortalProjectileParticleSnapshot::new(
            4.0,
            5.25,
            LegacyColor {
                r: 0.0,
                g: 0.2,
                b: 0.5,
                a: 0.6,
            },
        ));
        shell.portal_projectiles.push(snapshot.clone());
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 2.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.portal_projectile_render_previews.previews.len(), 1);
        let preview = &frame.portal_projectile_render_previews.previews[0];
        assert_eq!(preview.projectile_index, 0);
        assert_eq!(preview.snapshot, snapshot);
        assert_eq!(preview.particle_draws.len(), 1);
        let particle = preview.particle_draws[0];
        assert_eq!(particle.particle_index, 0);
        assert_eq!(particle.draw_x_px, 96.0);
        assert_eq!(particle.draw_y_px, 152.0);
        assert_eq!(
            particle.image_path,
            LEGACY_RUNTIME_PORTAL_PROJECTILE_PARTICLE_IMAGE_PATH,
        );
        assert_eq!(particle.origin_x_px, 0.5);
        assert_eq!(particle.origin_y_px, 0.5);
        assert!(!particle.live_rendering_executed);
        let head = preview
            .head_draw
            .expect("projectile head should draw while timer is below travel time");
        assert_eq!(head.draw_x_px, 104.0);
        assert_eq!(head.draw_y_px, 160.0);
        assert_eq!(head.image_path, LEGACY_RUNTIME_PORTAL_PROJECTILE_IMAGE_PATH);
        assert_eq!(head.origin_x_px, 3.0);
        assert_eq!(head.origin_y_px, 3.0);
        assert!(preview.particles_drawn_before_head);
        assert!(!preview.color_reset_after_draw);
        assert!(!preview.live_rendering_executed);
        assert!(!preview.live_projectile_physics_migrated);
        assert!(!preview.live_portal_mutated);
        assert_eq!(shell.portal_projectiles, vec![snapshot]);
    }

    #[test]
    fn shell_reports_emancipation_grill_render_preview_without_rendering_or_physics_migration() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let horizontal =
            LegacyRuntimeEmancipationGrillSnapshot::horizontal(3.0, 4.0, 2.0, 5.0, 160.0)
                .with_particle(LegacyRuntimeEmancipationGrillParticleSnapshot::new(
                    0.25,
                    LegacyRuntimeEmancipationGrillParticleDirection::Forward,
                    1.0,
                ))
                .with_particle(LegacyRuntimeEmancipationGrillParticleSnapshot::new(
                    0.5,
                    LegacyRuntimeEmancipationGrillParticleDirection::Backward,
                    -1.0,
                ));
        let vertical = LegacyRuntimeEmancipationGrillSnapshot::vertical(4.0, 5.0, 3.0, 8.0, 192.0)
            .with_particle(LegacyRuntimeEmancipationGrillParticleSnapshot::new(
                0.25,
                LegacyRuntimeEmancipationGrillParticleDirection::Forward,
                2.0,
            ))
            .with_particle(LegacyRuntimeEmancipationGrillParticleSnapshot::new(
                0.5,
                LegacyRuntimeEmancipationGrillParticleDirection::Backward,
                0.0,
            ));
        shell.emancipation_grills.push(horizontal.clone());
        shell.emancipation_grills.push(vertical.clone());
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 2.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.emancipation_grill_render_previews.previews.len(), 2);
        let horizontal_preview = &frame.emancipation_grill_render_previews.previews[0];
        assert_eq!(horizontal_preview.grill_index, 0);
        assert_eq!(horizontal_preview.snapshot, horizontal);
        assert_eq!(
            horizontal_preview.scissor,
            Some(LegacyRuntimeEmancipationGrillScissorPreview {
                x_px: 0.0,
                y_px: 92.0,
                width_px: 32.0,
                height_px: 8.0,
            }),
        );
        assert_eq!(
            horizontal_preview.line_rect,
            Some(LegacyRuntimeEmancipationGrillLinePreview {
                x_px: 0.0,
                y_px: 92.0,
                width_px: 160.0,
                height_px: 8.0,
                color: LEGACY_RUNTIME_EMANCIPATION_GRILL_LINE_COLOR,
            }),
        );
        assert_eq!(horizontal_preview.particle_draws.len(), 2);
        assert_eq!(horizontal_preview.particle_draws[0].draw_x_px, 40.0);
        assert_eq!(horizontal_preview.particle_draws[0].draw_y_px, 94.0);
        assert_eq!(
            horizontal_preview.particle_draws[0].image_path,
            LEGACY_RUNTIME_EMANCIPATION_GRILL_PARTICLE_IMAGE_PATH,
        );
        assert_eq!(
            horizontal_preview.particle_draws[0].rotation,
            consts::FRAC_PI_2,
        );
        assert_eq!(horizontal_preview.particle_draws[0].origin_x_px, 0.0);
        assert_eq!(horizontal_preview.particle_draws[1].draw_x_px, 16.0);
        assert_eq!(horizontal_preview.particle_draws[1].draw_y_px, 98.0);
        assert_eq!(
            horizontal_preview.particle_draws[1].rotation,
            -consts::FRAC_PI_2,
        );
        assert_eq!(horizontal_preview.particle_draws[1].origin_x_px, 1.0);
        assert_eq!(horizontal_preview.side_draws.len(), 2);
        assert_eq!(horizontal_preview.side_draws[0].draw_x_px, 0.0);
        assert_eq!(horizontal_preview.side_draws[0].draw_y_px, 88.0);
        assert_eq!(
            horizontal_preview.side_draws[0].image_path,
            LEGACY_RUNTIME_EMANCIPATION_GRILL_SIDE_IMAGE_PATH,
        );
        assert_eq!(horizontal_preview.side_draws[1].draw_x_px, 128.0);
        assert_eq!(horizontal_preview.side_draws[1].draw_y_px, 104.0);
        assert_eq!(horizontal_preview.side_draws[1].rotation, consts::PI);
        assert!(horizontal_preview.scissor_cleared_after_particles);
        assert!(horizontal_preview.color_reset_after_line);
        assert!(!horizontal_preview.live_rendering_executed);
        assert!(!horizontal_preview.live_grill_physics_migrated);

        let vertical_preview = &frame.emancipation_grill_render_previews.previews[1];
        assert_eq!(vertical_preview.grill_index, 1);
        assert_eq!(vertical_preview.snapshot, vertical);
        assert_eq!(
            vertical_preview.scissor,
            Some(LegacyRuntimeEmancipationGrillScissorPreview {
                x_px: 76.0,
                y_px: 48.0,
                width_px: 8.0,
                height_px: 64.0,
            }),
        );
        assert_eq!(
            vertical_preview.line_rect,
            Some(LegacyRuntimeEmancipationGrillLinePreview {
                x_px: 76.0,
                y_px: 48.0,
                width_px: 8.0,
                height_px: 64.0,
                color: LEGACY_RUNTIME_EMANCIPATION_GRILL_LINE_COLOR,
            }),
        );
        assert_eq!(vertical_preview.particle_draws[0].draw_x_px, 78.0);
        assert_eq!(vertical_preview.particle_draws[0].draw_y_px, 112.0);
        assert_eq!(vertical_preview.particle_draws[0].rotation, consts::PI);
        assert_eq!(vertical_preview.particle_draws[1].draw_x_px, 82.0);
        assert_eq!(vertical_preview.particle_draws[1].draw_y_px, 128.0);
        assert_eq!(vertical_preview.particle_draws[1].rotation, 0.0);
        assert_eq!(vertical_preview.side_draws[0].draw_x_px, 88.0);
        assert_eq!(vertical_preview.side_draws[0].draw_y_px, 48.0);
        assert_eq!(vertical_preview.side_draws[0].rotation, consts::FRAC_PI_2);
        assert_eq!(vertical_preview.side_draws[1].draw_x_px, 72.0);
        assert_eq!(vertical_preview.side_draws[1].draw_y_px, 240.0);
        assert_eq!(vertical_preview.side_draws[1].rotation, -consts::FRAC_PI_2,);
        assert_eq!(shell.emancipation_grills, vec![horizontal, vertical]);
    }

    #[test]
    fn shell_reports_door_render_preview_without_rendering_or_entity_migration() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let horizontal = LegacyRuntimeDoorSnapshot::from_legacy_horizontal_coord(4.0, 5.0, 0.25);
        let vertical = LegacyRuntimeDoorSnapshot::from_legacy_vertical_coord(6.0, 7.0, 0.75)
            .with_open(true)
            .with_active(false);
        shell.doors.push(horizontal);
        shell.doors.push(vertical);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 2.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.door_render_previews.previews.len(), 2);
        let horizontal_preview = &frame.door_render_previews.previews[0];
        assert_eq!(horizontal_preview.door_index, 0);
        assert_eq!(horizontal_preview.snapshot, horizontal);
        assert_eq!(horizontal_preview.ymod_tiles, 0.0);
        assert_eq!(horizontal_preview.center_rotation_delta, consts::PI * 0.25);
        assert_eq!(
            horizontal_preview.part_draws[0].kind,
            LegacyRuntimeDoorPartKind::Piece
        );
        assert_eq!(horizontal_preview.part_draws[0].draw_x_px, 92.0);
        assert_eq!(horizontal_preview.part_draws[0].draw_y_px, 128.0);
        assert_eq!(
            horizontal_preview.part_draws[0].image_path,
            LEGACY_RUNTIME_DOOR_PIECE_IMAGE_PATH,
        );
        assert_eq!(horizontal_preview.part_draws[0].rotation, consts::FRAC_PI_2);
        assert_eq!(horizontal_preview.part_draws[1].draw_x_px, 100.0);
        assert_eq!(horizontal_preview.part_draws[1].rotation, consts::PI * 1.5);
        assert_eq!(
            horizontal_preview.part_draws[2].kind,
            LegacyRuntimeDoorPartKind::Center
        );
        assert_eq!(horizontal_preview.part_draws[2].draw_x_px, 96.0);
        assert_eq!(
            horizontal_preview.part_draws[2].image_path,
            LEGACY_RUNTIME_DOOR_CENTER_IMAGE_PATH,
        );
        assert_eq!(
            horizontal_preview.part_draws[2].rotation,
            consts::FRAC_PI_2 - consts::PI * 0.25,
        );
        assert_eq!(horizontal_preview.part_draws[2].origin_y_px, 2.0);
        assert_eq!(horizontal_preview.part_draws[3].draw_x_px, 96.0);
        assert_eq!(
            horizontal_preview.part_draws[3].rotation,
            consts::PI * 1.5 - consts::PI * 0.25,
        );
        assert!(!horizontal_preview.live_rendering_executed);
        assert!(!horizontal_preview.live_door_physics_migrated);
        assert!(!horizontal_preview.live_door_entity_mutated);

        let vertical_preview = &frame.door_render_previews.previews[1];
        assert_eq!(vertical_preview.door_index, 1);
        assert_eq!(vertical_preview.snapshot, vertical);
        assert_eq!(vertical_preview.ymod_tiles, 0.5);
        assert_eq!(vertical_preview.center_rotation_delta, consts::FRAC_PI_2);
        assert_eq!(vertical_preview.part_draws[0].draw_x_px, 144.0);
        assert_eq!(vertical_preview.part_draws[0].draw_y_px, 156.0);
        assert_eq!(vertical_preview.part_draws[0].rotation, consts::PI);
        assert_eq!(vertical_preview.part_draws[1].draw_y_px, 196.0);
        assert_eq!(vertical_preview.part_draws[1].rotation, 0.0);
        assert_eq!(vertical_preview.part_draws[2].draw_y_px, 160.0);
        assert_eq!(vertical_preview.part_draws[2].rotation, consts::FRAC_PI_2);
        assert_eq!(vertical_preview.part_draws[3].draw_y_px, 192.0);
        assert_eq!(
            vertical_preview.part_draws[3].rotation,
            consts::PI + consts::FRAC_PI_2,
        );
        assert!(!vertical_preview.live_rendering_executed);
        assert!(!vertical_preview.live_door_physics_migrated);
        assert!(!vertical_preview.live_door_entity_mutated);
        assert_eq!(shell.doors, vec![horizontal, vertical]);
    }

    #[test]
    fn shell_reports_wall_indicator_render_preview_without_rendering_or_entity_migration() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let off = LegacyRuntimeWallIndicatorSnapshot::from_legacy_coord(4.0, 5.0, false);
        let on = LegacyRuntimeWallIndicatorSnapshot::from_legacy_coord(6.0, 7.0, true);
        shell.wall_indicators.push(off);
        shell.wall_indicators.push(on);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 2.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.wall_indicator_render_previews.previews.len(), 2);
        let off_preview = frame.wall_indicator_render_previews.previews[0];
        assert_eq!(off_preview.indicator_index, 0);
        assert_eq!(off_preview.snapshot, off);
        assert_eq!(off_preview.quad_index, 1);
        assert_eq!(off_preview.source_x_px, 0.0);
        assert_eq!(off_preview.source_y_px, 0.0);
        assert_eq!(off_preview.source_w_px, 16.0);
        assert_eq!(off_preview.source_h_px, 16.0);
        assert_eq!(
            off_preview.image_path,
            LEGACY_RUNTIME_WALL_INDICATOR_IMAGE_PATH
        );
        assert_eq!(off_preview.draw_x_px, 64.0);
        assert_eq!(off_preview.draw_y_px, 112.0);
        assert_eq!(off_preview.rotation, 0.0);
        assert_eq!(off_preview.scale_x, 2.0);
        assert_eq!(off_preview.scale_y, 2.0);
        assert_eq!(off_preview.color, LegacyColor::rgb(1.0, 1.0, 1.0));
        assert!(!off_preview.live_rendering_executed);
        assert!(!off_preview.live_wall_indicator_physics_migrated);
        assert!(!off_preview.live_wall_indicator_entity_mutated);

        let on_preview = frame.wall_indicator_render_previews.previews[1];
        assert_eq!(on_preview.indicator_index, 1);
        assert_eq!(on_preview.snapshot, on);
        assert_eq!(on_preview.quad_index, 2);
        assert_eq!(on_preview.source_x_px, 16.0);
        assert_eq!(on_preview.draw_x_px, 128.0);
        assert_eq!(on_preview.draw_y_px, 176.0);
        assert!(!on_preview.live_rendering_executed);
        assert!(!on_preview.live_wall_indicator_physics_migrated);
        assert!(!on_preview.live_wall_indicator_entity_mutated);
        assert_eq!(shell.wall_indicators, vec![off, on]);
    }

    #[test]
    fn shell_reports_fireball_map_target_probe_from_projectile_position_without_collision_mutation()
    {
        let mut cells = flat_level_cells(3);
        cells[(4 - 1) * 3 + (3 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        let fireball = iw2wth_core::LegacyFireballState::spawn(
            1.0,
            3.5,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 8.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.fireball_map_target_probes.reports.len(), 1);
        let probe = frame.fireball_map_target_probes.reports[0];
        let progressed = frame.fireball_projectile_progress.reports[0].state_after;
        assert_eq!(probe.projectile_index, 0);
        assert_eq!(probe.state, progressed);
        assert_eq!(probe.coord, LegacyMapTileCoord::new(3, 4));
        assert_eq!(probe.tile_id, TileId(5));
        assert_eq!(probe.axis, LegacyRuntimeFireballCollisionAxis::Right);
        assert_eq!(probe.target, LegacyFireballCollisionTarget::Tile);
        assert!(probe.collides);
        assert!(!probe.invisible);
        assert!(probe.breakable);
        assert!(probe.coin_block);
        assert!(probe.play_block_hit_sound);
        assert!(!probe.live_projectile_collision_mutated);
        assert_eq!(
            shell.fireball_projectiles,
            vec![progressed],
            "map target probes report Lua tile-hit metadata without applying fireball collision responses",
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
            ],
            "map target probes now feed report-only fireball tile collision audio intents",
        );
        assert_eq!(frame.fireball_collision_probes.reports.len(), 1);
        let collision_probe = frame.fireball_collision_probes.reports[0];
        assert_eq!(collision_probe.projectile_index, 0);
        assert_eq!(
            collision_probe.source,
            LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe {
                coord: LegacyMapTileCoord::new(3, 4),
                tile_id: TileId(5),
            },
        );
        assert_eq!(
            collision_probe.axis,
            LegacyRuntimeFireballCollisionAxis::Right
        );
        assert_eq!(collision_probe.target, LegacyFireballCollisionTarget::Tile);
        assert_eq!(collision_probe.state_before, progressed);
        assert_eq!(
            collision_probe.outcome,
            LegacyFireballCollisionOutcome {
                suppress_default: true,
                released_thrower: true,
                play_block_hit_sound: true,
                shoot_target: None,
                points: None,
            },
        );
        assert_eq!(frame.fireball_collision_probes.release_summaries.len(), 1);
        assert_eq!(frame.fireball_count_projections.len(), 1);
        assert_eq!(
            frame.fireball_count_projections[0].source,
            LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
                projectile_index: 0,
                collision_source: LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe {
                    coord: LegacyMapTileCoord::new(3, 4),
                    tile_id: TileId(5),
                },
                axis: LegacyRuntimeFireballCollisionAxis::Right,
                target: LegacyFireballCollisionTarget::Tile,
            },
        );
        assert!(
            frame.score_counter_intents.is_empty(),
            "tile collision outcomes do not synthesize score events",
        );
    }

    #[test]
    fn shell_threads_fireball_map_floor_target_into_bounce_outcome_without_release() {
        let mut cells = flat_level_cells(3);
        cells[(5 - 1) * 3 + (2 - 1)] = "2";
        let mut shell = loaded_test_shell(&cells);
        let mut fireball = iw2wth_core::LegacyFireballState::spawn(
            1.1,
            2.05,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        fireball.speed_x = 0.0;
        fireball.speed_y = 120.0;
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 8.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.fireball_map_target_probes.reports.len(), 1);
        let target_probe = frame.fireball_map_target_probes.reports[0];
        assert_eq!(target_probe.coord, LegacyMapTileCoord::new(2, 5));
        assert_eq!(target_probe.axis, LegacyRuntimeFireballCollisionAxis::Floor);
        assert!(!target_probe.play_block_hit_sound);

        assert_eq!(frame.fireball_collision_probes.reports.len(), 1);
        let collision_probe = frame.fireball_collision_probes.reports[0];
        assert_eq!(
            collision_probe.axis,
            LegacyRuntimeFireballCollisionAxis::Floor
        );
        assert_eq!(
            collision_probe.outcome,
            LegacyFireballCollisionOutcome {
                suppress_default: true,
                released_thrower: false,
                play_block_hit_sound: false,
                shoot_target: None,
                points: None,
            },
        );
        assert!(frame.fireball_collision_probes.release_summaries.is_empty());
        assert!(frame.fireball_count_projections.is_empty());
        assert!(frame.frame.audio_commands.is_empty());
        assert_eq!(
            shell.fireball_projectiles,
            vec![frame.fireball_projectile_progress.reports[0].state_after],
            "map-derived floor collision outcome is a report-only bounce and does not mutate the live projectile",
        );
    }

    #[test]
    fn shell_fireball_map_target_probe_skips_invisible_tiles_like_lua_non_player_tile_scan() {
        let mut cells = flat_level_cells(4);
        cells[(4 - 1) * 4 + (3 - 1)] = "4";
        let mut shell = loaded_test_shell(&cells);
        let fireball = iw2wth_core::LegacyFireballState::spawn(
            1.0,
            3.5,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 8.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(
            frame.fireball_map_target_probes.reports.is_empty(),
            "Lua skips invisible tile objects for non-player tile scans before fireball hitstuff can run",
        );
        assert!(
            frame.collisions.block_hits.is_empty(),
            "fireball target probes are separate from live player block-hit reports",
        );
    }

    #[test]
    fn shell_fireball_map_target_probe_reads_projected_tile_changes_without_mutating_level() {
        let mut cells = flat_level_cells(3);
        cells[(4 - 1) * 3 + (3 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell
            .projected_tile_changes
            .apply_projection(LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(3, 4),
                },
                tile_change: LegacyTileChange {
                    coord: TileCoord::new(3, 4),
                    tile: TileId(1),
                },
            });
        let fireball = iw2wth_core::LegacyFireballState::spawn(
            1.0,
            3.5,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 8.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(
            frame.fireball_map_target_probes.reports.is_empty(),
            "projected reward tile changes feed fireball map probes before live fireball collision is migrated",
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(3, 4)),
            Some(TileId(5)),
            "projected tile changes steer fireball map probes without mutating the parsed level",
        );
        assert!(
            frame.collisions.block_hits.is_empty(),
            "projected fireball target probes remain separate from live player block-hit reports",
        );
    }

    #[test]
    fn shell_reports_fireball_collision_probe_without_live_collision_mutation() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let fireball = iw2wth_core::LegacyFireballState::spawn(
            3.0,
            4.0,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Left,
                    LegacyFireballCollisionTarget::Tile,
                ),
            ),
            &test_tiles(),
        );

        assert_eq!(frame.fireball_collision_probes.reports.len(), 1);
        let probe = frame.fireball_collision_probes.reports[0];
        let progressed = frame.fireball_projectile_progress.reports[0].state_after;
        assert_eq!(probe.projectile_index, 0);
        assert_eq!(probe.axis, LegacyRuntimeFireballCollisionAxis::Left);
        assert_eq!(probe.target, LegacyFireballCollisionTarget::Tile);
        assert_eq!(probe.state_before, progressed);
        assert_close(probe.state_after.x, progressed.x - 0.5);
        assert_close(probe.state_after.speed_x, 15.0);
        assert_eq!(probe.state_after.frame, LegacyFireballFrame::ExplosionOne);
        assert!(!probe.state_after.active);
        assert!(probe.state_after.destroy_soon);
        assert_eq!(
            probe.outcome,
            LegacyFireballCollisionOutcome {
                suppress_default: true,
                released_thrower: true,
                play_block_hit_sound: true,
                shoot_target: None,
                points: None,
            },
        );
        assert_eq!(
            frame.fireball_collision_probes.release_summaries,
            vec![LegacyRuntimeFireballProjectileReleaseSummary {
                projectile_index: 0,
                source: LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
                    source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    axis: LegacyRuntimeFireballCollisionAxis::Left,
                    target: LegacyFireballCollisionTarget::Tile,
                },
                callback: LegacyRuntimeFireballCallbackMetadata {
                    callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
                    fireball_count_delta: -1,
                },
                live_projectile_queue_mutated: false,
                live_fireball_counter_mutated: false,
            }],
            "collision probes preserve Lua fireballcallback metadata without mutating live projectile queues or counters",
        );
        assert_eq!(
            shell.fireball_projectiles,
            vec![progressed],
            "collision probes clone projectile state and do not mutate the live queue",
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
            ],
            "tile-like fireball probes preserve block-hit sound intent without live audio execution",
        );
        assert!(frame.score_counter_intents.is_empty());
        assert_eq!(frame.fireball_count_projections.len(), 1);
        let count_projection = frame.fireball_count_projections[0];
        assert_eq!(
            count_projection.source,
            LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
                projectile_index: 0,
                collision_source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Left,
                target: LegacyFireballCollisionTarget::Tile,
            },
        );
        assert_eq!(count_projection.active_fireball_count_before, 1);
        assert_eq!(count_projection.fireball_count_delta, -1);
        assert_eq!(count_projection.active_fireball_count_after, 0);
        assert!(!count_projection.live_fireball_counter_mutated);
        assert_eq!(
            frame.projected_fireball_count_state.active_fireball_count(),
            Some(0),
        );
    }

    #[test]
    fn shell_threads_fireball_collision_enemy_points_into_report_only_score_summary() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        shell.score_count = 400;
        let fireball = iw2wth_core::LegacyFireballState::spawn(
            3.0,
            4.0,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        shell.fireball_projectiles.push(fireball);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.25, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Passive,
                    LegacyFireballCollisionTarget::Goomba,
                ),
            ),
            &test_tiles(),
        );

        let probe = frame.fireball_collision_probes.reports[0];
        assert_eq!(probe.outcome.points, Some(100));
        assert_eq!(
            probe.outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert!(frame.frame.audio_commands.is_empty());
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::FireballCollisionProbe {
                    projectile_index: 0,
                    source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    axis: LegacyRuntimeFireballCollisionAxis::Passive,
                    target: LegacyFireballCollisionTarget::Goomba,
                },
                score_count_before: 400,
                score_delta: 100,
                score_count_after: 500,
                scrolling_score: Some(LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(100),
                    probe.state_after.x,
                    probe.state_after.y,
                    1.25,
                )),
            }],
            "enemy fireball probes preserve Lua addpoints metadata without mutating score",
        );
        assert_eq!(
            shell.scrolling_score_animations,
            vec![LegacyRuntimeScrollingScoreAnimationState {
                source: LegacyRuntimeScoreSource::FireballCollisionProbe {
                    projectile_index: 0,
                    source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    axis: LegacyRuntimeFireballCollisionAxis::Passive,
                    target: LegacyFireballCollisionTarget::Goomba,
                },
                score: LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(100),
                    probe.state_after.x,
                    probe.state_after.y,
                    1.25,
                ),
            }],
            "fireball collision score provenance feeds the report-only scrolling-score queue",
        );
        assert_eq!(
            shell.fireball_projectiles,
            vec![frame.fireball_projectile_progress.reports[0].state_after],
            "enemy fireball probes keep live projectile physics report-only",
        );

        let second_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.25, 1.0),
                None,
            ),
            &test_tiles(),
        );
        assert_eq!(
            second_frame
                .scrolling_score_animation_progress
                .reports
                .len(),
            1
        );
        let scrolling_score_report = second_frame.scrolling_score_animation_progress.reports[0];
        assert_eq!(
            scrolling_score_report.source,
            LegacyRuntimeScoreSource::FireballCollisionProbe {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
            "scrolling-score animation progress preserves explicit fireball collision provenance",
        );
        assert_eq!(
            scrolling_score_report.state.label,
            LegacyScrollingScoreLabel::Points(100),
        );
        assert!(!scrolling_score_report.remove);
        assert_eq!(shell.score_count, 400);
    }

    #[test]
    fn shell_threads_explicit_fireball_enemy_collision_into_report_only_hit_intent() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        shell.score_count = 400;
        shell.fireball_enemies = vec![LegacyRuntimeFireballEnemySnapshot::new(
            LegacyFireballCollisionTarget::Goomba,
            7,
            3.5,
            4.0,
            1.0,
            1.0,
            true,
        )];
        shell
            .fireball_projectiles
            .push(iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            ));
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.25, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Passive,
                    LegacyFireballCollisionTarget::Goomba,
                ),
            ),
            &test_tiles(),
        );

        let probe = frame.fireball_collision_probes.reports[0];
        assert_eq!(
            frame.fireball_enemy_hit_intents,
            vec![LegacyRuntimeFireballEnemyHitIntent {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
                enemy: shell.fireball_enemies[0],
                shot_direction: LegacyEnemyDirection::Right,
                score_delta: Some(100),
                score_x: probe.state_after.x,
                score_y: probe.state_after.y,
                live_enemy_mutated: false,
            }],
            "explicit fireball enemy probes preserve b:shotted(\"right\") metadata without live enemy mutation",
        );
        assert_eq!(
            frame.score_counter_intents[0].source,
            LegacyRuntimeScoreSource::FireballCollisionProbe {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
        );
        assert_eq!(shell.fireball_enemies.len(), 1);
        assert!(!frame.fireball_enemy_hit_intents[0].live_enemy_mutated);
        assert_eq!(frame.projected_fireball_enemy_hit_snapshots.len(), 1);
        assert_eq!(
            frame.projected_fireball_enemy_hit_snapshots[0],
            LegacyRuntimeProjectedFireballEnemyHitSnapshot {
                intent: frame.fireball_enemy_hit_intents[0],
                enemy: shell.fireball_enemies[0],
                active_after: false,
                shot_after: true,
                removed_from_future_queries: true,
                live_enemy_mutated: false,
            },
            "Lua shotted() disables the enemy for future active enemy queries without mutating the live adapter snapshot",
        );
        assert_eq!(shell.projected_fireball_enemy_hits.len(), 1);

        let later_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.25, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Passive,
                    LegacyFireballCollisionTarget::Goomba,
                ),
            ),
            &test_tiles(),
        );
        assert!(
            later_frame.fireball_enemy_hit_intents.is_empty(),
            "projected enemy-hit state suppresses later report-only enemy queries for the same removed enemy",
        );
        assert!(
            later_frame.score_counter_intents.is_empty(),
            "projected enemy-hit state also suppresses repeated score reports for the removed enemy",
        );
        assert_eq!(
            later_frame.projected_fireball_enemy_hit_state.len(),
            1,
            "the adapter keeps the projected hit/removal snapshot without live enemy-list mutation",
        );
        assert_eq!(
            shell.scrolling_score_animations.len(),
            1,
            "repeated explicit probes against a projected removed enemy do not enqueue another scrolling score",
        );

        let mut missing_snapshot_shell = loaded_test_shell(&cells);
        missing_snapshot_shell
            .fireball_projectiles
            .push(iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            ));
        let missing_snapshot_frame = missing_snapshot_shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Passive,
                    LegacyFireballCollisionTarget::Goomba,
                ),
            ),
            &test_tiles(),
        );
        assert!(
            missing_snapshot_frame.fireball_enemy_hit_intents.is_empty(),
            "adapter-side enemy-hit summaries require a provided enemy snapshot",
        );
    }

    #[derive(Debug)]
    struct FireballEnemyOverlapFixture {
        name: String,
        projectile: iw2wth_core::LegacyFireballState,
        dt: f32,
        enemies: Vec<LegacyRuntimeFireballEnemySnapshot>,
        expected_probe_count: usize,
        expected_enemy_index: Option<usize>,
        expected_axis: Option<LegacyRuntimeFireballCollisionAxis>,
        expected_target: Option<LegacyFireballCollisionTarget>,
        expected_shoot_target: Option<LegacyEnemyDirection>,
        expected_points: Option<u32>,
        expected_released_thrower: bool,
        expected_after_active: bool,
        expected_after_destroy_soon: bool,
        expected_after_speed_x: f32,
        expected_after_speed_y: f32,
    }

    #[test]
    fn shell_fireball_enemy_overlap_reports_match_lua_generated_fixtures() {
        let fixtures = parse_fireball_enemy_overlap_fixtures();
        assert_eq!(
            fixtures.len(),
            5,
            "fixture table should cover passive, horizontal, vertical, handler-skip, and beetle overlap cases",
        );

        for fixture in fixtures {
            let report = probe_legacy_runtime_fireball_collisions(
                &[fixture.projectile],
                None,
                &LegacyRuntimeFireballMapTargetProbeReport::default(),
                &fixture.enemies,
                &LegacyRuntimeProjectedFireballEnemyHitState::default(),
                fixture.dt,
            );

            assert_eq!(
                report.reports.len(),
                fixture.expected_probe_count,
                "fixture `{}` probe count should match the Lua ordering baseline",
                fixture.name,
            );

            if fixture.expected_probe_count == 0 {
                continue;
            }

            let probe = report.reports[0];
            assert_eq!(probe.projectile_index, 0, "fixture `{}`", fixture.name);
            assert_eq!(
                probe.source,
                LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe {
                    enemy_index: fixture
                        .expected_enemy_index
                        .expect("nonzero fixture probe should name an enemy"),
                },
                "fixture `{}` should retain the selected enemy index",
                fixture.name,
            );
            assert_eq!(
                Some(probe.axis),
                fixture.expected_axis,
                "fixture `{}` selected collision axis",
                fixture.name,
            );
            assert_eq!(
                Some(probe.target),
                fixture.expected_target,
                "fixture `{}` selected target",
                fixture.name,
            );
            assert_eq!(
                probe.outcome.shoot_target, fixture.expected_shoot_target,
                "fixture `{}` shot direction",
                fixture.name,
            );
            assert_eq!(
                probe.outcome.points, fixture.expected_points,
                "fixture `{}` score payload",
                fixture.name,
            );
            assert_eq!(
                probe.outcome.released_thrower, fixture.expected_released_thrower,
                "fixture `{}` fireball callback release",
                fixture.name,
            );
            assert_eq!(
                probe.state_after.active, fixture.expected_after_active,
                "fixture `{}` active-after state",
                fixture.name,
            );
            assert_eq!(
                probe.state_after.destroy_soon, fixture.expected_after_destroy_soon,
                "fixture `{}` destroysoon-after state",
                fixture.name,
            );
            assert_close(probe.state_after.speed_x, fixture.expected_after_speed_x);
            assert_close(probe.state_after.speed_y, fixture.expected_after_speed_y);
        }
    }

    fn parse_fireball_enemy_overlap_fixtures() -> Vec<FireballEnemyOverlapFixture> {
        let mut fixtures = Vec::new();
        let mut current: Option<FireballEnemyOverlapFixture> = None;

        for (line_number, line) in FIREBALL_ENEMY_OVERLAP_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let columns: Vec<_> = line.split_whitespace().collect();
            assert_eq!(
                columns.len(),
                28,
                "fixture line {} should have 28 columns",
                line_number + 1,
            );

            let name = columns[0].to_owned();
            let projectile = iw2wth_core::LegacyFireballState {
                x: parse_f32(columns[1], line_number),
                y: parse_f32(columns[2], line_number),
                width: parse_f32(columns[3], line_number),
                height: parse_f32(columns[4], line_number),
                speed_x: parse_f32(columns[5], line_number),
                speed_y: parse_f32(columns[6], line_number),
                active: parse_bool(columns[7], line_number),
                destroy: false,
                destroy_soon: false,
                rotation: 0.0,
                timer: 0.0,
                frame: LegacyFireballFrame::FlyingOne,
            };
            let dt = parse_f32(columns[8], line_number);
            let enemy = LegacyRuntimeFireballEnemySnapshot::new(
                parse_fireball_target(columns[10], line_number),
                parse_usize(columns[11], line_number),
                parse_f32(columns[12], line_number),
                parse_f32(columns[13], line_number),
                parse_f32(columns[14], line_number),
                parse_f32(columns[15], line_number),
                parse_bool(columns[16], line_number),
            );
            let expected_probe_count = parse_usize(columns[17], line_number);
            let expected_enemy_index = parse_optional_usize(columns[18], line_number);
            let expected_axis = parse_optional_fireball_axis(columns[19], line_number);
            let expected_target = parse_optional_fireball_target(columns[20], line_number);
            let expected_shoot_target = parse_optional_enemy_direction(columns[21], line_number);
            let expected_points = parse_optional_u32(columns[22], line_number);
            let expected_released_thrower = parse_bool(columns[23], line_number);
            let expected_after_active = parse_bool(columns[24], line_number);
            let expected_after_destroy_soon = parse_bool(columns[25], line_number);
            let expected_after_speed_x = parse_f32(columns[26], line_number);
            let expected_after_speed_y = parse_f32(columns[27], line_number);

            match current.as_mut() {
                Some(fixture) if fixture.name == name => {
                    assert_eq!(fixture.projectile, projectile);
                    assert_close(fixture.dt, dt);
                    assert_eq!(fixture.expected_probe_count, expected_probe_count);
                    assert_eq!(fixture.expected_enemy_index, expected_enemy_index);
                    assert_eq!(fixture.expected_axis, expected_axis);
                    assert_eq!(fixture.expected_target, expected_target);
                    assert_eq!(fixture.expected_shoot_target, expected_shoot_target);
                    assert_eq!(fixture.expected_points, expected_points);
                    assert_eq!(fixture.expected_released_thrower, expected_released_thrower);
                    assert_eq!(fixture.expected_after_active, expected_after_active);
                    assert_eq!(
                        fixture.expected_after_destroy_soon,
                        expected_after_destroy_soon
                    );
                    assert_close(fixture.expected_after_speed_x, expected_after_speed_x);
                    assert_close(fixture.expected_after_speed_y, expected_after_speed_y);
                    fixture.enemies.push(enemy);
                }
                _ => {
                    if let Some(fixture) = current.take() {
                        fixtures.push(fixture);
                    }
                    current = Some(FireballEnemyOverlapFixture {
                        name,
                        projectile,
                        dt,
                        enemies: vec![enemy],
                        expected_probe_count,
                        expected_enemy_index,
                        expected_axis,
                        expected_target,
                        expected_shoot_target,
                        expected_points,
                        expected_released_thrower,
                        expected_after_active,
                        expected_after_destroy_soon,
                        expected_after_speed_x,
                        expected_after_speed_y,
                    });
                }
            }
        }

        if let Some(fixture) = current {
            fixtures.push(fixture);
        }

        fixtures
    }

    fn parse_fireball_target(value: &str, line_number: usize) -> LegacyFireballCollisionTarget {
        match value {
            "goomba" => LegacyFireballCollisionTarget::Goomba,
            "koopa" => LegacyFireballCollisionTarget::Koopa { beetle: false },
            "koopa_beetle" => LegacyFireballCollisionTarget::Koopa { beetle: true },
            "hammerbro" => LegacyFireballCollisionTarget::HammerBro,
            "plant" => LegacyFireballCollisionTarget::Plant,
            "cheep" => LegacyFireballCollisionTarget::Cheep,
            "bowser" => LegacyFireballCollisionTarget::Bowser,
            "squid" => LegacyFireballCollisionTarget::Squid,
            "flyingfish" => LegacyFireballCollisionTarget::FlyingFish,
            "lakito" => LegacyFireballCollisionTarget::Lakito,
            _ => panic!("invalid fireball target `{value}` on fixture line {line_number}"),
        }
    }

    fn parse_optional_fireball_target(
        value: &str,
        line_number: usize,
    ) -> Option<LegacyFireballCollisionTarget> {
        if value == "none" {
            None
        } else {
            Some(parse_fireball_target(value, line_number))
        }
    }

    fn parse_optional_fireball_axis(
        value: &str,
        line_number: usize,
    ) -> Option<LegacyRuntimeFireballCollisionAxis> {
        match value {
            "none" => None,
            "left" => Some(LegacyRuntimeFireballCollisionAxis::Left),
            "right" => Some(LegacyRuntimeFireballCollisionAxis::Right),
            "floor" => Some(LegacyRuntimeFireballCollisionAxis::Floor),
            "ceiling" => Some(LegacyRuntimeFireballCollisionAxis::Ceiling),
            "passive" => Some(LegacyRuntimeFireballCollisionAxis::Passive),
            _ => panic!("invalid fireball axis `{value}` on fixture line {line_number}"),
        }
    }

    fn parse_optional_enemy_direction(
        value: &str,
        line_number: usize,
    ) -> Option<LegacyEnemyDirection> {
        match value {
            "none" => None,
            "left" => Some(LegacyEnemyDirection::Left),
            "right" => Some(LegacyEnemyDirection::Right),
            _ => panic!("invalid enemy direction `{value}` on fixture line {line_number}"),
        }
    }

    fn parse_optional_usize(value: &str, line_number: usize) -> Option<usize> {
        if value == "none" {
            None
        } else {
            Some(parse_usize(value, line_number))
        }
    }

    fn parse_optional_u32(value: &str, line_number: usize) -> Option<u32> {
        if value == "none" {
            None
        } else {
            Some(value.parse().unwrap_or_else(|error| {
                panic!("invalid u32 `{value}` on fixture line {line_number}: {error}")
            }))
        }
    }

    fn parse_usize(value: &str, line_number: usize) -> usize {
        value.parse().unwrap_or_else(|error| {
            panic!("invalid usize `{value}` on fixture line {line_number}: {error}")
        })
    }

    fn parse_bool(value: &str, line_number: usize) -> bool {
        match value {
            "true" => true,
            "false" => false,
            _ => panic!("invalid bool `{value}` on fixture line {line_number}"),
        }
    }

    fn parse_f32(value: &str, line_number: usize) -> f32 {
        value.parse().unwrap_or_else(|error| {
            panic!("invalid f32 `{value}` on fixture line {line_number}: {error}")
        })
    }

    #[test]
    fn shell_threads_fireball_enemy_snapshots_into_report_only_overlap_probe() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        shell.score_count = 250;
        let enemy = LegacyRuntimeFireballEnemySnapshot::new(
            LegacyFireballCollisionTarget::Goomba,
            7,
            3.5,
            4.0,
            1.0,
            1.0,
            true,
        );
        let later_enemy = LegacyRuntimeFireballEnemySnapshot::new(
            LegacyFireballCollisionTarget::Goomba,
            8,
            3.5,
            4.0,
            1.0,
            1.0,
            true,
        );
        shell.fireball_enemies = vec![enemy, later_enemy];
        shell
            .fireball_projectiles
            .push(iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            ));
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.fireball_collision_probes.reports.len(), 1);
        let probe = frame.fireball_collision_probes.reports[0];
        assert_eq!(probe.projectile_index, 0);
        assert_eq!(
            probe.source,
            LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { enemy_index: 7 },
        );
        assert_eq!(probe.axis, LegacyRuntimeFireballCollisionAxis::Passive);
        assert_eq!(probe.target, LegacyFireballCollisionTarget::Goomba);
        assert_eq!(
            probe.outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(probe.outcome.points, Some(100));
        assert_eq!(
            frame.fireball_enemy_hit_intents,
            vec![LegacyRuntimeFireballEnemyHitIntent {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe {
                    enemy_index: 7,
                },
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
                enemy,
                shot_direction: LegacyEnemyDirection::Right,
                score_delta: Some(100),
                score_x: probe.state_after.x,
                score_y: probe.state_after.y,
                live_enemy_mutated: false,
            }],
            "adapter enemy snapshots can now drive report-only fireball/enemy overlap hits",
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::FireballCollisionProbe {
                    projectile_index: 0,
                    source: LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe {
                        enemy_index: 7,
                    },
                    axis: LegacyRuntimeFireballCollisionAxis::Passive,
                    target: LegacyFireballCollisionTarget::Goomba,
                },
                score_count_before: 250,
                score_delta: 100,
                score_count_after: 350,
                scrolling_score: Some(LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(100),
                    probe.state_after.x,
                    probe.state_after.y,
                    1.0,
                )),
            }],
            "automatic overlap probes preserve Lua firepoints scoring without mutating counters",
        );
        assert_eq!(
            shell.fireball_projectiles,
            vec![frame.fireball_projectile_progress.reports[0].state_after],
            "enemy overlap probes clone projectile state and do not mutate the live projectile queue",
        );
        assert_eq!(
            frame
                .projected_fireball_projectile_collision_snapshots
                .len(),
            1,
            "the first overlap now produces an adapter-side projectile collision projection",
        );
        let projectile_snapshot = frame.projected_fireball_projectile_collision_snapshots[0];
        assert_eq!(projectile_snapshot.projectile_index, 0);
        assert_eq!(
            projectile_snapshot.source,
            LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { enemy_index: 7 },
        );
        assert!(!projectile_snapshot.state_after.active);
        assert!(projectile_snapshot.state_after.destroy_soon);
        assert!(projectile_snapshot.removed_from_future_collision_queries);
        assert!(!projectile_snapshot.live_projectile_queue_mutated);
        assert_eq!(
            shell.fireball_enemies,
            vec![enemy, later_enemy],
            "enemy overlap probes keep adapter enemy snapshots report-only",
        );

        let later_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 1.0),
                None,
            ),
            &test_tiles(),
        );
        assert!(
            later_frame.fireball_collision_probes.reports.is_empty(),
            "projected projectile collision state suppresses later automatic overlap probes before the live projectile queue is migrated",
        );
        assert!(later_frame.fireball_enemy_hit_intents.is_empty());
        assert_eq!(shell.projected_fireball_enemy_hits.len(), 1);
        assert_eq!(shell.projected_fireball_projectile_collisions.len(), 1);
    }

    #[test]
    fn shell_threads_projected_fireball_collision_state_into_progress_previews() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        shell.fireball_enemies = vec![LegacyRuntimeFireballEnemySnapshot::new(
            LegacyFireballCollisionTarget::Goomba,
            7,
            3.5,
            4.0,
            1.0,
            1.0,
            true,
        )];
        shell
            .fireball_projectiles
            .push(iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            ));
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let request = || {
            LegacyRuntimeFrameRequest::new(
                0.05,
                0.2,
                LegacyRuntimeRenderContext::new(1.0, 1.0),
                None,
            )
        };

        let collision_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let projected_collision =
            collision_frame.projected_fireball_projectile_collision_snapshots[0];
        assert_eq!(
            projected_collision.state_after.frame,
            LegacyFireballFrame::ExplosionOne
        );
        assert!(projected_collision.state_after.destroy_soon);
        assert_eq!(collision_frame.fireball_render_previews.previews.len(), 1);
        let collision_preview = collision_frame.fireball_render_previews.previews[0];
        assert_eq!(collision_preview.projectile_index, 0);
        assert_eq!(
            collision_preview.source,
            LegacyRuntimeFireballRenderSource::ProjectedProjectileCollision,
        );
        assert_eq!(collision_preview.state, projected_collision.state_after);
        assert_eq!(collision_preview.frame, LegacyFireballFrame::ExplosionOne);
        assert_eq!(
            collision_preview.frame_kind,
            LegacyRuntimeFireballRenderFrameKind::Explosion,
        );
        assert_eq!(
            collision_preview.quad,
            LegacyRuntimeFireballRenderQuad {
                x_px: 32,
                y_px: 0,
                width_px: 16,
                height_px: 16,
            },
        );
        assert!(!collision_preview.live_rendering_executed);
        assert!(!collision_preview.live_projectile_queue_mutated);
        assert_eq!(
            shell.fireball_projectiles,
            vec![collision_frame.fireball_projectile_progress.reports[0].state_after],
            "the live projectile queue remains at the pre-collision progression state",
        );

        let still_explosion_one_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let still_explosion_one_progress = still_explosion_one_frame
            .fireball_projectile_progress
            .reports[0];
        assert_eq!(
            still_explosion_one_progress.state_before,
            projected_collision.state_after,
        );
        assert_eq!(
            still_explosion_one_progress.state_after.frame,
            LegacyFireballFrame::ExplosionOne,
        );
        assert!(!still_explosion_one_progress.update.remove);
        assert!(
            still_explosion_one_frame
                .fireball_collision_probes
                .reports
                .is_empty(),
            "projected inactive explosion state remains collision-query only",
        );

        let explosion_two_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let explosion_two_progress = explosion_two_frame.fireball_projectile_progress.reports[0];
        assert_eq!(
            explosion_two_progress.state_before.frame,
            LegacyFireballFrame::ExplosionOne,
        );
        assert_eq!(
            explosion_two_progress.state_after.frame,
            LegacyFireballFrame::ExplosionTwo,
        );
        assert!(!explosion_two_progress.update.remove);
        assert!(
            explosion_two_frame
                .fireball_collision_probes
                .reports
                .is_empty(),
            "projected inactive explosion state remains collision-query only",
        );

        let still_explosion_two_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let still_explosion_two_progress = still_explosion_two_frame
            .fireball_projectile_progress
            .reports[0];
        assert_eq!(
            still_explosion_two_progress.state_after.frame,
            LegacyFireballFrame::ExplosionTwo,
        );

        let explosion_three_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let explosion_three_progress =
            explosion_three_frame.fireball_projectile_progress.reports[0];
        assert_eq!(
            explosion_three_progress.state_after.frame,
            LegacyFireballFrame::ExplosionThree,
        );
        assert!(!explosion_three_progress.update.remove);

        let still_explosion_three_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let still_explosion_three_progress = still_explosion_three_frame
            .fireball_projectile_progress
            .reports[0];
        assert_eq!(
            still_explosion_three_progress.state_after.frame,
            LegacyFireballFrame::ExplosionThree,
        );
        assert!(!still_explosion_three_progress.update.remove);

        let final_explosion_three_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let final_explosion_three_progress = final_explosion_three_frame
            .fireball_projectile_progress
            .reports[0];
        assert_eq!(
            final_explosion_three_progress.state_after.frame,
            LegacyFireballFrame::ExplosionThree,
        );
        assert!(!final_explosion_three_progress.update.remove);

        let removal_frame = shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        let removal_progress = removal_frame.fireball_projectile_progress.reports[0];
        assert_eq!(
            removal_progress.state_before.frame,
            LegacyFireballFrame::ExplosionThree
        );
        assert!(removal_progress.update.remove);
        assert!(!removal_progress.update.released_thrower);
        assert!(removal_progress.state_after.destroy);
        assert!(
            removal_frame
                .fireball_projectile_progress
                .release_summaries
                .is_empty()
        );
        assert_eq!(
            removal_frame
                .fireball_projectile_progress
                .queue_len_after_prune,
            0
        );
        assert!(
            removal_frame.fireball_render_previews.previews.is_empty(),
            "projected removal suppresses fireball draw previews before live render migration",
        );
        assert_eq!(
            removal_frame
                .fireball_render_previews
                .suppressed_projected_removal_indices,
            vec![0],
        );
        assert_eq!(
            shell.fireball_projectiles,
            vec![collision_frame.fireball_projectile_progress.reports[0].state_after],
            "projected explosion removal is preview-only and does not prune the live queue",
        );

        let after_removal_frame =
            shell.step_player_frame(&mut player, &input, request(), &test_tiles());
        assert!(
            after_removal_frame
                .fireball_projectile_progress
                .reports
                .is_empty(),
            "projected removal suppresses later progress previews for the same projectile",
        );
    }

    #[test]
    fn shell_projects_fireball_count_into_later_fire_guards_without_live_counter_mutation() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let input = BufferedLegacyInputSnapshot::new();
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let first_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(-0.1)
                    .with_flower_power(true)
                    .with_active_fireball_count(1),
            ),
            &test_tiles(),
        );

        let launch = first_frame
            .fireball_launch_intent
            .expect("first fire request should launch below the Lua maxfireballs guard");
        assert_eq!(launch.fireball_count_before, 1);
        assert_eq!(launch.fireball_count_after, 2);
        assert_eq!(first_frame.fireball_count_projections.len(), 1);
        assert_eq!(
            first_frame.fireball_count_projections[0].source,
            LegacyRuntimeProjectedFireballCountSource::LaunchIntent,
        );
        assert_eq!(
            shell.projected_fireball_count.active_fireball_count(),
            Some(2),
        );

        let blocked_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(-0.1)
                    .with_flower_power(true)
                    .with_active_fireball_count(1),
            ),
            &test_tiles(),
        );
        assert!(
            blocked_frame.fireball_launch_intent.is_none(),
            "later fire guards use the projected active count instead of the stale adapter snapshot",
        );
        assert_eq!(
            shell.projected_fireball_count.active_fireball_count(),
            Some(2),
        );

        let release_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_collision_probe(
                LegacyRuntimeFireballCollisionProbeRequest::new(
                    0,
                    LegacyRuntimeFireballCollisionAxis::Passive,
                    LegacyFireballCollisionTarget::Goomba,
                ),
            ),
            &test_tiles(),
        );
        assert_eq!(release_frame.fireball_count_projections.len(), 1);
        let release_projection = release_frame.fireball_count_projections[0];
        assert_eq!(release_projection.active_fireball_count_before, 2);
        assert_eq!(release_projection.fireball_count_delta, -1);
        assert_eq!(release_projection.active_fireball_count_after, 1);
        assert!(!release_projection.live_fireball_counter_mutated);
        assert_eq!(
            shell.projected_fireball_count.active_fireball_count(),
            Some(1),
        );

        let allowed_frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                LEGACY_MAX_UPDATE_DT,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(-0.1)
                    .with_flower_power(true)
                    .with_active_fireball_count(2),
            ),
            &test_tiles(),
        );
        let allowed_launch = allowed_frame
            .fireball_launch_intent
            .expect("fireballcallback projection should reopen one maxfireballs slot");
        assert_eq!(allowed_launch.fireball_count_before, 1);
        assert_eq!(allowed_launch.fireball_count_after, 2);
        assert_eq!(
            shell.projected_fireball_count.active_fireball_count(),
            Some(2),
        );
    }

    #[test]
    fn fireball_collision_probe_preserves_floor_ceiling_and_passive_contracts() {
        let constants = iw2wth_core::LegacyFireballConstants::default();
        let projectile = iw2wth_core::LegacyFireballState::spawn(
            3.0,
            4.0,
            LegacyEnemyDirection::Right,
            constants,
        );
        let projected_enemy_hits = LegacyRuntimeProjectedFireballEnemyHitState::default();

        let floor = probe_legacy_runtime_fireball_collisions(
            &[projectile],
            Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Floor,
                LegacyFireballCollisionTarget::Tile,
            )),
            &LegacyRuntimeFireballMapTargetProbeReport::default(),
            &[],
            &projected_enemy_hits,
            LEGACY_MAX_UPDATE_DT,
        )
        .reports[0];
        assert!(floor.outcome.suppress_default);
        assert!(!floor.outcome.play_block_hit_sound);
        assert!(floor.state_after.active);
        assert_close(floor.state_after.speed_y, -constants.jump_force);
        let floor_report = probe_legacy_runtime_fireball_collisions(
            &[projectile],
            Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Floor,
                LegacyFireballCollisionTarget::Tile,
            )),
            &LegacyRuntimeFireballMapTargetProbeReport::default(),
            &[],
            &projected_enemy_hits,
            LEGACY_MAX_UPDATE_DT,
        );
        assert!(
            floor_report.release_summaries.is_empty(),
            "floor tile bounces do not call fireballcallback in Lua",
        );

        let ceiling_report = probe_legacy_runtime_fireball_collisions(
            &[projectile],
            Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Ceiling,
                LegacyFireballCollisionTarget::BulletBill,
            )),
            &LegacyRuntimeFireballMapTargetProbeReport::default(),
            &[],
            &projected_enemy_hits,
            LEGACY_MAX_UPDATE_DT,
        );
        let ceiling = ceiling_report.reports[0];
        assert!(!ceiling.outcome.suppress_default);
        assert!(ceiling.outcome.play_block_hit_sound);
        assert!(ceiling.outcome.released_thrower);
        assert!(ceiling.state_after.destroy_soon);
        assert_eq!(ceiling_report.release_summaries.len(), 1);

        let passive = probe_legacy_runtime_fireball_collisions(
            &[projectile],
            Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Passive,
                LegacyFireballCollisionTarget::Plant,
            )),
            &LegacyRuntimeFireballMapTargetProbeReport::default(),
            &[],
            &projected_enemy_hits,
            LEGACY_MAX_UPDATE_DT,
        )
        .reports[0];
        assert!(passive.outcome.suppress_default);
        assert_eq!(
            passive.outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(passive.outcome.points, Some(200));
        assert!(passive.state_after.destroy_soon);
    }

    #[test]
    fn shell_reports_portal_target_probe_without_creating_live_portals() {
        let mut cells = flat_level_cells(6);
        cells[(3 - 1) * 6 + (4 - 1)] = "2";
        cells[(4 - 1) * 6 + (4 - 1)] = "2";
        let tiles = LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                portalable: true,
                ..LegacyTileMetadata::empty()
            },
        ]);
        let mut shell = loaded_test_shell(&cells);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.125, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_portal_aim(LegacyRuntimePortalAimSnapshot::new(-1.5).with_portal_1_down(true)),
            &tiles,
        );

        let probe = frame
            .portal_target_probe
            .expect("active portal aim snapshot should report a target probe");
        assert_eq!(probe.requested_slot, Some(LegacyRuntimePortalSlot::Portal1));
        let hit = probe.trace_hit.expect("ray should hit the portalable wall");
        assert_eq!(hit.coord, LegacyMapTileCoord::new(4, 4));
        assert_eq!(hit.side, Facing::Left);
        assert_eq!(hit.tendency, -1);
        assert_close(hit.impact_x, 3.0);
        assert_close(hit.impact_y, 3.455_678);
        let aim_preview = frame
            .portal_aim_render_preview
            .as_ref()
            .expect("active portal aim should expose draw-loop metadata");
        assert_eq!(
            aim_preview.player_source,
            LegacyRuntimePortalTargetPlayerSource::LivePlayer
        );
        assert_eq!(
            aim_preview.requested_slot,
            Some(LegacyRuntimePortalSlot::Portal1)
        );
        assert!(aim_preview.portal_possible);
        assert_close(aim_preview.source_x, 2.375);
        assert_close(aim_preview.source_y, 3.5);
        assert_close(aim_preview.target_x, 3.0);
        assert_close(aim_preview.target_y, 3.455_678);
        assert_eq!(aim_preview.dots_timer, 0.0);
        assert_eq!(aim_preview.dots_time, LEGACY_RUNTIME_PORTAL_DOT_TIME);
        assert_eq!(
            aim_preview.dots_distance_tiles,
            LEGACY_RUNTIME_PORTAL_DOT_DISTANCE_TILES,
        );
        assert_eq!(aim_preview.dot_draws.len(), 1);
        let dot = aim_preview.dot_draws[0];
        assert_eq!(dot.sequence_index, 1);
        assert_eq!(dot.phase, 0.0);
        assert_eq!(dot.draw_x_px, 37.0);
        assert_eq!(dot.draw_y_px, 47.0);
        assert_eq!(dot.alpha, 0.0);
        assert_eq!(
            dot.color,
            LegacyColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 0.0,
            },
        );
        assert_eq!(dot.image_path, LEGACY_RUNTIME_PORTAL_DOT_IMAGE_PATH);
        assert!(!dot.live_rendering_executed);
        let crosshair = aim_preview
            .crosshair
            .expect("trace hit should expose crosshair metadata");
        assert_eq!(crosshair.draw_x_px, 48.0);
        assert_eq!(crosshair.draw_y_px, 47.0);
        assert_eq!(crosshair.rotation, consts::FRAC_PI_2 * 3.0);
        assert_eq!(crosshair.origin_x_px, 4);
        assert_eq!(crosshair.origin_y_px, 8);
        assert_eq!(
            crosshair.color,
            LegacyColor {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        );
        assert_eq!(
            crosshair.image_path,
            LEGACY_RUNTIME_PORTAL_CROSSHAIR_IMAGE_PATH,
        );
        assert!(!crosshair.live_rendering_executed);
        assert!(aim_preview.color_reset_after_dots);
        assert!(aim_preview.color_reset_after_crosshair);
        assert!(!aim_preview.live_rendering_executed);
        assert!(!aim_preview.live_portal_mutated);
        let placement = probe
            .placement
            .expect("two-tile-high portalable wall should allow a placement");
        assert_eq!(placement.coord, LegacyMapTileCoord::new(4, 4));
        assert_eq!(placement.side, Facing::Left);
        assert!(probe.portal_possible());
        let outcome = frame
            .portal_outcome_intent
            .expect("requested valid portal target should report an open intent");
        assert_eq!(outcome.requested_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(outcome.kind, LegacyRuntimePortalOutcomeKind::Open);
        assert_eq!(outcome.placement, Some(placement));
        assert_eq!(outcome.sound, LegacySoundEffect::Portal1Open);
        let expected_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement,
            tile_reservations: [LegacyMapTileCoord::new(4, 4), LegacyMapTileCoord::new(4, 3)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(4, 2, 0, 2),
                LegacyRuntimePortalWallReservation::new(3, 2, 1, 0),
                LegacyRuntimePortalWallReservation::new(3, 4, 1, 0),
            ],
        };
        assert_eq!(
            frame.portal_reservation_projections,
            vec![expected_projection],
        );
        assert_eq!(
            frame.portal_replacement_summaries,
            vec![LegacyRuntimePortalReplacementSummary {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                previous_slot: None,
                replacement_slot: LegacyRuntimeProjectedPortal {
                    requested_slot: LegacyRuntimePortalSlot::Portal1,
                    placement,
                    tile_reservations: expected_projection.tile_reservations,
                    wall_reservations: expected_projection.wall_reservations,
                    block_reservation: LegacyBlockPortalReservation::new(
                        TileCoord::new(4, 4),
                        Facing::Up,
                    ),
                },
                preserved_other_slot: None,
            }],
        );
        assert_eq!(
            frame.projected_portal_state.portal_1,
            Some(LegacyRuntimeProjectedPortal {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement,
                tile_reservations: expected_projection.tile_reservations,
                wall_reservations: expected_projection.wall_reservations,
                block_reservation: LegacyBlockPortalReservation::new(
                    TileCoord::new(4, 4),
                    Facing::Up,
                ),
            }),
        );
        assert_eq!(frame.projected_portal_state.portal_2, None);
        assert_eq!(
            frame.portal_pair_readiness_summary,
            Some(LegacyRuntimePortalPairReadinessSummary {
                portal_1: frame.projected_portal_state.portal_1,
                portal_2: None,
                ready: false,
                portal_1_to_2: None,
                portal_2_to_1: None,
            }),
        );
        assert_eq!(
            frame.projected_portal_state.block_portal_reservations(),
            vec![LegacyBlockPortalReservation::new(
                TileCoord::new(4, 4),
                Facing::Up,
            )],
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Portal1Open),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Portal1Open),
            ],
        );
        assert!(shell.block_portal_reservations.is_empty());
        assert_eq!(shell.projected_portal_state, frame.projected_portal_state);
    }

    #[test]
    fn portal_reservation_projection_preserves_lua_tile_and_wall_shapes_for_each_facing() {
        let cases = [
            (
                Facing::Up,
                [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
                [
                    LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                    LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                    LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
                ],
            ),
            (
                Facing::Down,
                [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(4, 6)],
                [
                    LegacyRuntimePortalWallReservation::new(3, 5, 2, 0),
                    LegacyRuntimePortalWallReservation::new(3, 5, 0, 1),
                    LegacyRuntimePortalWallReservation::new(5, 5, 0, 1),
                ],
            ),
            (
                Facing::Left,
                [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(5, 5)],
                [
                    LegacyRuntimePortalWallReservation::new(5, 4, 0, 2),
                    LegacyRuntimePortalWallReservation::new(4, 4, 1, 0),
                    LegacyRuntimePortalWallReservation::new(4, 6, 1, 0),
                ],
            ),
            (
                Facing::Right,
                [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(5, 7)],
                [
                    LegacyRuntimePortalWallReservation::new(4, 5, 0, 2),
                    LegacyRuntimePortalWallReservation::new(4, 5, 1, 0),
                    LegacyRuntimePortalWallReservation::new(4, 7, 1, 0),
                ],
            ),
        ];

        for (facing, tile_reservations, wall_reservations) in cases {
            let placement = LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(5, 6),
                side: facing,
            };
            let outcome = LegacyRuntimePortalOutcomeIntent {
                requested_slot: LegacyRuntimePortalSlot::Portal2,
                kind: LegacyRuntimePortalOutcomeKind::Open,
                placement: Some(placement),
                sound: LegacySoundEffect::Portal2Open,
            };

            assert_eq!(
                super::legacy_runtime_portal_reservation_projection(outcome),
                Some(LegacyRuntimePortalReservationProjection {
                    requested_slot: LegacyRuntimePortalSlot::Portal2,
                    placement,
                    tile_reservations,
                    wall_reservations,
                }),
            );
        }

        let fizzle = LegacyRuntimePortalOutcomeIntent {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            kind: LegacyRuntimePortalOutcomeKind::Fizzle,
            placement: None,
            sound: LegacySoundEffect::PortalFizzle,
        };
        assert_eq!(
            super::legacy_runtime_portal_reservation_projection(fizzle),
            None,
        );
    }

    #[test]
    fn projected_portal_state_snapshots_open_slots_for_future_portal_guards() {
        let portal_1_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(5, 6),
                side: Facing::Up,
            },
            tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
            ],
        };
        let portal_2_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal2,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(9, 4),
                side: Facing::Right,
            },
            tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
            ],
        };
        let replacement_portal_1_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(2, 8),
                side: Facing::Down,
            },
            tile_reservations: [LegacyMapTileCoord::new(2, 8), LegacyMapTileCoord::new(1, 8)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(0, 7, 2, 0),
                LegacyRuntimePortalWallReservation::new(0, 7, 0, 1),
                LegacyRuntimePortalWallReservation::new(2, 7, 0, 1),
            ],
        };
        let mut state = LegacyRuntimeProjectedPortalState::default();

        assert_eq!(state.portal_pair_readiness_summary(), None);
        assert_eq!(
            state.replacement_summary_for_projection(portal_1_projection),
            Some(LegacyRuntimePortalReplacementSummary {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                previous_slot: None,
                replacement_slot: LegacyRuntimeProjectedPortal::from_projection(
                    portal_1_projection
                ),
                preserved_other_slot: None,
            }),
        );
        state.apply_projection(portal_1_projection);
        assert_eq!(state.projected_slot_count(), 1);
        assert_eq!(
            state.portal_pair_readiness_summary(),
            Some(LegacyRuntimePortalPairReadinessSummary {
                portal_1: Some(LegacyRuntimeProjectedPortal::from_projection(
                    portal_1_projection
                )),
                portal_2: None,
                ready: false,
                portal_1_to_2: None,
                portal_2_to_1: None,
            }),
        );
        assert_eq!(
            state.slot(LegacyRuntimePortalSlot::Portal1),
            Some(LegacyRuntimeProjectedPortal {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement: portal_1_projection.placement,
                tile_reservations: portal_1_projection.tile_reservations,
                wall_reservations: portal_1_projection.wall_reservations,
                block_reservation: LegacyBlockPortalReservation::new(
                    TileCoord::new(5, 6),
                    Facing::Right,
                ),
            }),
        );
        assert_eq!(
            state.replacement_summary_for_projection(portal_1_projection),
            None,
        );

        assert_eq!(
            state.replacement_summary_for_projection(portal_2_projection),
            Some(LegacyRuntimePortalReplacementSummary {
                requested_slot: LegacyRuntimePortalSlot::Portal2,
                previous_slot: None,
                replacement_slot: LegacyRuntimeProjectedPortal::from_projection(
                    portal_2_projection
                ),
                preserved_other_slot: Some(LegacyRuntimeProjectedPortal::from_projection(
                    portal_1_projection
                )),
            }),
        );
        state.apply_projection(portal_2_projection);
        assert_eq!(state.projected_slot_count(), 2);
        let portal_1 = LegacyRuntimeProjectedPortal::from_projection(portal_1_projection);
        let portal_2 = LegacyRuntimeProjectedPortal::from_projection(portal_2_projection);
        assert_eq!(
            state.portal_pair_readiness_summary(),
            Some(LegacyRuntimePortalPairReadinessSummary {
                portal_1: Some(portal_1),
                portal_2: Some(portal_2),
                ready: true,
                portal_1_to_2: Some(LegacyRuntimePortalPairing {
                    entry_slot: LegacyRuntimePortalSlot::Portal1,
                    exit_slot: LegacyRuntimePortalSlot::Portal2,
                    entry: portal_1,
                    exit: portal_2,
                }),
                portal_2_to_1: Some(LegacyRuntimePortalPairing {
                    entry_slot: LegacyRuntimePortalSlot::Portal2,
                    exit_slot: LegacyRuntimePortalSlot::Portal1,
                    entry: portal_2,
                    exit: portal_1,
                }),
            }),
        );
        assert_eq!(
            state.block_portal_reservations(),
            vec![
                LegacyBlockPortalReservation::new(TileCoord::new(5, 6), Facing::Right),
                LegacyBlockPortalReservation::new(TileCoord::new(9, 4), Facing::Down),
            ],
        );

        assert_eq!(
            state.replacement_summary_for_projection(replacement_portal_1_projection),
            Some(LegacyRuntimePortalReplacementSummary {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                previous_slot: Some(LegacyRuntimeProjectedPortal::from_projection(
                    portal_1_projection
                )),
                replacement_slot: LegacyRuntimeProjectedPortal::from_projection(
                    replacement_portal_1_projection
                ),
                preserved_other_slot: Some(LegacyRuntimeProjectedPortal::from_projection(
                    portal_2_projection
                )),
            }),
        );
        state.apply_projection(replacement_portal_1_projection);
        assert_eq!(state.projected_slot_count(), 2);
        assert_eq!(
            state.block_portal_reservations(),
            vec![
                LegacyBlockPortalReservation::new(TileCoord::new(2, 8), Facing::Left),
                LegacyBlockPortalReservation::new(TileCoord::new(9, 4), Facing::Down),
            ],
        );
        assert_eq!(
            state.slot(LegacyRuntimePortalSlot::Portal2),
            Some(LegacyRuntimeProjectedPortal::from_projection(
                portal_2_projection
            )),
        );
    }

    #[test]
    fn shell_report_only_block_portal_reservations_preserve_explicit_and_projected_guards() {
        let cells = flat_level_cells(2);
        let mut shell = loaded_test_shell(&cells);
        let explicit = LegacyBlockPortalReservation::new(TileCoord::new(1, 2), Facing::Down);
        shell.block_portal_reservations.push(explicit);
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(5, 6),
                    side: Facing::Up,
                },
                tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                    LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                    LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
                ],
            });

        assert_eq!(
            shell.report_only_block_portal_reservations(),
            vec![
                explicit,
                LegacyBlockPortalReservation::new(TileCoord::new(5, 6), Facing::Right),
            ],
        );
        assert_eq!(shell.block_portal_reservations, vec![explicit]);
    }

    #[test]
    fn shell_reports_portal_transit_candidate_from_ready_pair_without_mutating_player() {
        let cells = flat_level_cells(10);
        let mut shell = loaded_test_shell(&cells);
        let portal_1_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(5, 6),
                side: Facing::Up,
            },
            tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
            ],
        };
        let portal_2_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal2,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(9, 4),
                side: Facing::Right,
            },
            tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
            ],
        };
        shell
            .projected_portal_state
            .apply_projection(portal_1_projection);
        shell
            .projected_portal_state
            .apply_projection(portal_2_projection);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(3.875, 4.875, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let original_body = player.body;

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        let portal_1 = LegacyRuntimeProjectedPortal::from_projection(portal_1_projection);
        let portal_2 = LegacyRuntimeProjectedPortal::from_projection(portal_2_projection);
        assert_eq!(
            frame.portal_transit_candidate_probe,
            Some(LegacyRuntimePortalTransitCandidateProbe {
                center_x: 4.25,
                center_y: 5.25,
                center_coord: LegacyMapTileCoord::new(5, 6),
                candidate_entry_tile: Some(LegacyMapTileCoord::new(5, 6)),
                candidate_pairing: Some(LegacyRuntimePortalPairing {
                    entry_slot: LegacyRuntimePortalSlot::Portal1,
                    exit_slot: LegacyRuntimePortalSlot::Portal2,
                    entry: portal_1,
                    exit: portal_2,
                }),
            }),
        );
        assert_eq!(
            frame.portalcoords_preview,
            Some(LegacyRuntimePortalCoordsPreviewReport {
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                entry_facing: Facing::Up,
                exit_facing: Facing::Right,
                input_body: original_body,
                input_speed_x: 0.0,
                input_speed_y: 0.0,
                input_rotation: 0.0,
                output_body: PlayerBodyBounds::new(8.875, 4.375, 12.0 / 16.0, 12.0 / 16.0),
                output_speed_x: 0.0,
                output_speed_y: 0.0,
                output_rotation: -std::f32::consts::FRAC_PI_2,
                output_animation_direction: HorizontalDirection::Right,
                exit_blocked: false,
                blocked_exit_probe: None,
            }),
        );
        assert_eq!(
            frame.portal_transit_outcome_summary,
            Some(LegacyRuntimePortalTransitOutcomeSummary {
                kind: LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                entry_facing: Facing::Up,
                exit_facing: Facing::Right,
                input_body: original_body,
                output_body: PlayerBodyBounds::new(8.875, 4.375, 12.0 / 16.0, 12.0 / 16.0),
                output_speed_x: 0.0,
                output_speed_y: 0.0,
                blocked_exit_probe: None,
            }),
        );
        assert_eq!(
            frame.portal_transit_audio_intent,
            Some(LegacyRuntimePortalTransitAudioIntent {
                outcome_kind: LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                sound: LegacySoundEffect::PortalEnter,
            }),
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::PortalEnter),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::PortalEnter),
            ],
        );
        let expected_snapshot = LegacyRuntimeProjectedPlayerStateSnapshot {
            source: LegacyRuntimeProjectedPlayerStateSource::PortalTransitTeleportPreview,
            entry_slot: LegacyRuntimePortalSlot::Portal1,
            exit_slot: LegacyRuntimePortalSlot::Portal2,
            entry_facing: Facing::Up,
            exit_facing: Facing::Right,
            body: PlayerBodyBounds::new(8.875, 4.375, 12.0 / 16.0, 12.0 / 16.0),
            speed_x: 0.0,
            speed_y: 0.0,
            animation_direction: HorizontalDirection::Right,
        };
        assert_eq!(
            frame.portal_transit_projected_player_snapshot,
            Some(expected_snapshot),
        );
        assert_eq!(
            frame.projected_player_state.latest_portal_transit(),
            Some(expected_snapshot),
        );
        assert_eq!(
            shell.projected_player_state.latest_portal_transit(),
            Some(expected_snapshot),
        );
        assert_eq!(player.body, original_body);
    }

    #[test]
    fn shell_portalcoords_preview_reports_blocked_exit_probe_without_mutating_player() {
        let mut cells = flat_level_cells(10);
        cells[(5 - 1) * 10 + (10 - 1)] = "2";
        let mut shell = loaded_test_shell(&cells);
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(5, 6),
                    side: Facing::Up,
                },
                tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                    LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                    LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
                ],
            });
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal2,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(9, 4),
                    side: Facing::Right,
                },
                tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                    LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                    LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
                ],
            });
        let mut movement = PlayerMovementState {
            speed_y: 1.0,
            ..PlayerMovementState::default()
        };
        movement.animation_direction = HorizontalDirection::Right;
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(3.875, 4.875, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );
        let original_player = player;

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        let preview = frame
            .portalcoords_preview
            .expect("ready portal candidate should still expose a report-only preview");
        assert!(preview.exit_blocked);
        assert_eq!(
            preview.blocked_exit_probe,
            Some(LegacyRuntimePortalBlockedExitProbe {
                blocking_coord: LegacyMapTileCoord::new(10, 5),
                bounce_axis: LegacyRuntimePortalBlockedExitBounceAxis::Vertical,
                bounced_speed_x: 0.0,
                bounced_speed_y: -2.0,
            }),
        );
        assert_eq!(
            preview.output_body,
            PlayerBodyBounds::new(8.875, 4.375, 12.0 / 16.0, 12.0 / 16.0),
        );
        assert_eq!(
            frame.portal_transit_outcome_summary,
            Some(LegacyRuntimePortalTransitOutcomeSummary {
                kind: LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview,
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                entry_facing: Facing::Up,
                exit_facing: Facing::Right,
                input_body: original_player.body,
                output_body: PlayerBodyBounds::new(8.875, 4.375, 12.0 / 16.0, 12.0 / 16.0),
                output_speed_x: 1.0,
                output_speed_y: 0.0,
                blocked_exit_probe: Some(LegacyRuntimePortalBlockedExitProbe {
                    blocking_coord: LegacyMapTileCoord::new(10, 5),
                    bounce_axis: LegacyRuntimePortalBlockedExitBounceAxis::Vertical,
                    bounced_speed_x: 0.0,
                    bounced_speed_y: -2.0,
                }),
            }),
        );
        assert_eq!(
            frame.portal_transit_audio_intent,
            Some(LegacyRuntimePortalTransitAudioIntent {
                outcome_kind: LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview,
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                sound: LegacySoundEffect::PortalEnter,
            }),
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::PortalEnter),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::PortalEnter),
            ],
        );
        let expected_snapshot = LegacyRuntimeProjectedPlayerStateSnapshot {
            source: LegacyRuntimeProjectedPlayerStateSource::PortalTransitBlockedExitBouncePreview,
            entry_slot: LegacyRuntimePortalSlot::Portal1,
            exit_slot: LegacyRuntimePortalSlot::Portal2,
            entry_facing: Facing::Up,
            exit_facing: Facing::Right,
            body: original_player.body,
            speed_x: 0.0,
            speed_y: -2.0,
            animation_direction: HorizontalDirection::Right,
        };
        assert_eq!(
            frame.portal_transit_projected_player_snapshot,
            Some(expected_snapshot),
        );
        assert_eq!(
            frame.projected_player_state.latest_portal_transit(),
            Some(expected_snapshot),
        );
        assert_eq!(
            shell.projected_player_state.latest_portal_transit(),
            Some(expected_snapshot),
        );
        assert_eq!(player, original_player);
    }

    #[test]
    fn shell_threads_projected_player_snapshot_into_next_portal_transit_candidate_probe() {
        let cells = flat_level_cells(10);
        let mut shell = loaded_test_shell(&cells);
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(5, 6),
                    side: Facing::Up,
                },
                tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                    LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                    LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
                ],
            });
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal2,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(9, 4),
                    side: Facing::Right,
                },
                tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                    LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                    LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
                ],
            });
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(3.875, 4.875, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let original_body = player.body;

        let first_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );
        let snapshot = first_frame
            .portal_transit_projected_player_snapshot
            .expect("first frame should report the projected teleport output");

        let second_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        let candidate = second_frame
            .portal_transit_candidate_probe
            .expect("ready portal pair should still report a candidate probe");
        assert_eq!(
            candidate.center_x,
            snapshot.body.x + snapshot.body.width / 2.0
        );
        assert_eq!(
            candidate.center_y,
            snapshot.body.y + snapshot.body.height / 2.0
        );
        assert_eq!(candidate.center_coord, LegacyMapTileCoord::new(10, 5));
        assert_eq!(candidate.candidate_entry_tile, None);
        assert_eq!(candidate.candidate_pairing, None);
        assert_eq!(second_frame.portalcoords_preview, None);
        assert_eq!(
            second_frame.projected_player_state.latest_portal_transit(),
            Some(snapshot),
        );
        assert_eq!(player.body, original_body);
    }

    #[test]
    fn shell_threads_projected_player_snapshot_into_next_portal_target_probe() {
        let mut cells = flat_level_cells(6);
        cells[(3 - 1) * 6 + (4 - 1)] = "2";
        cells[(4 - 1) * 6 + (4 - 1)] = "2";
        let tiles = LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                portalable: true,
                ..LegacyTileMetadata::empty()
            },
        ]);
        let mut shell = loaded_test_shell(&cells);
        let projected_body = PlayerBodyBounds::new(2.0, 3.125, 12.0 / 16.0, 12.0 / 16.0);
        let snapshot = LegacyRuntimeProjectedPlayerStateSnapshot {
            source: LegacyRuntimeProjectedPlayerStateSource::PortalTransitTeleportPreview,
            entry_slot: LegacyRuntimePortalSlot::Portal1,
            exit_slot: LegacyRuntimePortalSlot::Portal2,
            entry_facing: Facing::Up,
            exit_facing: Facing::Right,
            body: projected_body,
            speed_x: 4.0,
            speed_y: -2.0,
            animation_direction: HorizontalDirection::Right,
        };
        shell.projected_player_state.apply_snapshot(snapshot);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(0.0, 0.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let original_body = player.body;

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_portal_aim(LegacyRuntimePortalAimSnapshot::new(-1.5).with_portal_1_down(true)),
            &tiles,
        );

        let probe = frame
            .portal_target_probe
            .expect("projected player should still allow an active portal target probe");
        assert_close(probe.source_x, projected_body.x + 6.0 / 16.0);
        assert_close(probe.source_y, projected_body.y + 6.0 / 16.0);
        assert_eq!(probe.requested_slot, Some(LegacyRuntimePortalSlot::Portal1));
        let hit = probe
            .trace_hit
            .expect("projected player source should trace to the portalable wall");
        assert_eq!(hit.coord, LegacyMapTileCoord::new(4, 4));
        assert_eq!(
            probe.placement,
            Some(LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(4, 4),
                side: Facing::Left,
            }),
        );
        assert_eq!(
            frame.projected_player_state.latest_portal_transit(),
            Some(snapshot),
        );
        assert_eq!(player.body, original_body);
    }

    #[test]
    fn portal_transit_candidate_probe_reports_ready_pair_even_when_center_is_outside_apertures() {
        let portal_1_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal1,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(5, 6),
                side: Facing::Up,
            },
            tile_reservations: [LegacyMapTileCoord::new(5, 6), LegacyMapTileCoord::new(6, 6)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(4, 6, 2, 0),
                LegacyRuntimePortalWallReservation::new(4, 5, 0, 1),
                LegacyRuntimePortalWallReservation::new(6, 5, 0, 1),
            ],
        };
        let portal_2_projection = LegacyRuntimePortalReservationProjection {
            requested_slot: LegacyRuntimePortalSlot::Portal2,
            placement: LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(9, 4),
                side: Facing::Right,
            },
            tile_reservations: [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
            wall_reservations: [
                LegacyRuntimePortalWallReservation::new(8, 3, 0, 2),
                LegacyRuntimePortalWallReservation::new(8, 3, 1, 0),
                LegacyRuntimePortalWallReservation::new(8, 5, 1, 0),
            ],
        };
        let mut state = LegacyRuntimeProjectedPortalState::default();
        state.apply_projection(portal_1_projection);
        state.apply_projection(portal_2_projection);
        let player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 2.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let cells = flat_level_cells(10);
        let shell = loaded_test_shell(&cells);
        let tiles = test_tiles();

        assert_eq!(
            super::legacy_runtime_portal_transit_candidate_probe(
                player,
                state.portal_pair_readiness_summary(),
            ),
            Some(LegacyRuntimePortalTransitCandidateProbe {
                center_x: 1.375,
                center_y: 2.375,
                center_coord: LegacyMapTileCoord::new(2, 3),
                candidate_entry_tile: None,
                candidate_pairing: None,
            }),
        );
        assert_eq!(
            super::legacy_runtime_portalcoords_preview(
                player,
                super::legacy_runtime_portal_transit_candidate_probe(
                    player,
                    state.portal_pair_readiness_summary(),
                ),
                0.0,
                shell.projected_metadata_map_query(&tiles),
            ),
            None,
        );
    }

    #[test]
    fn shell_portal_target_probe_rejects_non_portalable_or_blocked_surfaces() {
        let mut cells = flat_level_cells(6);
        cells[(3 - 1) * 6 + (4 - 1)] = "3";
        cells[(4 - 1) * 6 + (4 - 1)] = "3";
        let tiles = LegacyTileMetadataTable::from_metadata_for_tests(vec![
            LegacyTileMetadata::empty(),
            LegacyTileMetadata {
                collision: true,
                portalable: true,
                ..LegacyTileMetadata::empty()
            },
            LegacyTileMetadata {
                collision: true,
                portalable: false,
                ..LegacyTileMetadata::empty()
            },
        ]);
        let mut shell = loaded_test_shell(&cells);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.125, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let blocked = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_portal_aim(LegacyRuntimePortalAimSnapshot::new(-1.5).with_portal_2_down(true)),
            &tiles,
        );

        let blocked_probe = blocked
            .portal_target_probe
            .expect("active portal aim snapshot should still report the blocked target");
        assert_eq!(
            blocked_probe.requested_slot,
            Some(LegacyRuntimePortalSlot::Portal2)
        );
        assert_eq!(
            blocked_probe.trace_hit.map(|hit| hit.coord),
            Some(LegacyMapTileCoord::new(4, 4)),
        );
        assert_eq!(blocked_probe.placement, None);
        assert!(!blocked_probe.portal_possible());
        let blocked_outcome = blocked
            .portal_outcome_intent
            .expect("requested rejected portal target should report a fizzle intent");
        assert_eq!(
            blocked_outcome.requested_slot,
            LegacyRuntimePortalSlot::Portal2
        );
        assert_eq!(blocked_outcome.kind, LegacyRuntimePortalOutcomeKind::Fizzle);
        assert_eq!(blocked_outcome.placement, None);
        assert_eq!(blocked_outcome.sound, LegacySoundEffect::PortalFizzle);
        assert!(blocked.portal_reservation_projections.is_empty());
        assert_eq!(
            blocked.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::PortalFizzle),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::PortalFizzle),
            ],
        );

        let inactive = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            )
            .with_portal_aim(LegacyRuntimePortalAimSnapshot {
                controls_enabled: false,
                ..LegacyRuntimePortalAimSnapshot::new(-1.5)
            }),
            &tiles,
        );
        assert_eq!(inactive.portal_target_probe, None);
        assert_eq!(inactive.portal_outcome_intent, None);
        assert!(inactive.portal_reservation_projections.is_empty());
    }

    #[test]
    fn shell_player_frame_applies_legacy_movement_before_map_collision() {
        let cells = flat_level_cells(3);
        let mut shell = loaded_test_shell(&cells);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );
        let input = BufferedLegacyInputSnapshot::new().with_keyboard_key("right");

        let frame = shell.step_player_frame(
            &mut player,
            &input,
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.frame.movement_input,
            PlayerMovementInput::new(false, true, false),
        );
        assert_eq!(frame.collisions.horizontal, None);
        assert_eq!(frame.collisions.vertical, None);
        assert_close(player.movement.speed_x, 8.0 / 60.0);
        assert_close(player.movement.speed_y, 80.0 / 60.0);
        assert_close(player.body.x, 1.0 + player.movement.speed_x / 60.0);
        assert_close(player.body.y, 3.0 + player.movement.speed_y / 60.0);
        assert!(player.movement.falling);
        assert_eq!(
            player.movement.animation_state,
            PlayerAnimationState::Falling,
        );
    }

    #[test]
    fn shell_player_frame_lands_on_solid_tiles_from_parsed_map_query() {
        let mut cells = flat_level_cells(3);
        cells[(5 - 1) * 3 + (2 - 1)] = "2";
        let mut shell = loaded_test_shell(&cells);
        let mut movement = PlayerMovementState {
            speed_y: 20.0,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        movement.gravity = 80.0;
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.vertical.map(|collision| collision.coord),
            Some(LegacyMapTileCoord::new(2, 5)),
        );
        assert_eq!(
            frame.collisions.vertical.map(|collision| collision.axis),
            Some(LegacyRuntimePlayerCollisionAxis::Vertical),
        );
        assert_eq!(frame.collisions.horizontal, None);
        assert_close(player.body.y, 4.0 - 12.0 / 16.0);
        assert_close(player.movement.speed_y, 0.0);
        assert!(!player.movement.falling);
        assert_eq!(player.movement.animation_state, PlayerAnimationState::Idle);
    }

    #[test]
    fn shell_player_frame_snaps_against_solid_side_tiles_from_parsed_map_query() {
        let mut cells = flat_level_cells(4);
        cells[(4 - 1) * 4 + (3 - 1)] = "2";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_x: 60.0,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.1, 3.25, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.horizontal.map(|collision| collision.coord),
            Some(LegacyMapTileCoord::new(3, 4)),
        );
        assert_eq!(
            frame.collisions.horizontal.map(|collision| collision.axis),
            Some(LegacyRuntimePlayerCollisionAxis::Horizontal),
        );
        assert_close(player.body.x, 2.0 - 12.0 / 16.0);
        assert_close(player.movement.speed_x, 0.0);
    }

    #[test]
    fn shell_player_frame_collects_coin_tiles_before_movement_using_legacy_metadata() {
        let mut cells = flat_level_cells(3);
        cells[(4 - 1) * 3 + (2 - 1)] = "3";
        let mut shell = loaded_test_shell(&cells);
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.coin_pickups.len(), 1);
        assert_eq!(frame.coin_pickups[0].coord, LegacyMapTileCoord::new(2, 4));
        assert_eq!(frame.coin_pickups[0].tile_id, TileId(3));
        assert_eq!(frame.coin_pickups[0].clear_tile_id, TileId(1));
        assert_eq!(frame.coin_pickups[0].score_delta, 200);
        assert_eq!(frame.coin_pickups[0].sound, LegacySoundEffect::Coin);
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 4)),
            Some(TileId(1)),
        );

        let second_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(second_frame.coin_pickups.is_empty());
        assert!(second_frame.frame.audio_commands.is_empty());
    }

    #[test]
    fn shell_coin_pickup_reports_counter_intent_from_snapshot_without_mutating_coin_count() {
        let mut cells = flat_level_cells(3);
        cells[(4 - 1) * 3 + (2 - 1)] = "3";
        let mut shell = loaded_test_shell(&cells);
        shell.coin_count = 99;
        shell.score_count = 500;
        shell.life_count_enabled = false;
        shell.player_count = 2;
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.coin_pickups.len(), 1);
        assert_eq!(
            frame.coin_counter_intents,
            vec![LegacyRuntimeCoinCounterIntent {
                source: LegacyRuntimeCoinCounterSource::PlayerCoinPickup {
                    coord: LegacyMapTileCoord::new(2, 4),
                },
                coin_count_before: 99,
                coin_count_after: 0,
                life_reward: Some(LegacyCoinLifeReward {
                    grant_lives_to_players: 0,
                    respawn_players: false,
                    play_sound: true,
                }),
                score_delta: 200,
            }],
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::PlayerCoinPickup {
                    coord: LegacyMapTileCoord::new(2, 4),
                },
                score_count_before: 500,
                score_delta: 200,
                score_count_after: 700,
                scrolling_score: None,
            }],
        );
        assert_eq!(shell.coin_count, 99);
        assert_eq!(shell.score_count, 500);
    }

    #[test]
    fn shell_player_frame_suppresses_floor_collision_for_invisible_solid_tiles() {
        let mut cells = flat_level_cells(3);
        cells[(5 - 1) * 3 + (2 - 1)] = "4";
        let mut shell = loaded_test_shell(&cells);
        let mut movement = PlayerMovementState {
            speed_y: 20.0,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        movement.gravity = 80.0;
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.vertical, None);
        assert_eq!(frame.collisions.horizontal, None);
        assert_close(player.body.y, 3.0 + player.movement.speed_y / 60.0);
        assert_close(player.movement.speed_y, 20.0 + 80.0 / 60.0);
        assert!(player.movement.falling);
        assert_eq!(
            player.movement.animation_state,
            PlayerAnimationState::Falling,
        );
    }

    #[test]
    fn shell_player_frame_suppresses_side_collision_for_invisible_solid_tiles() {
        let mut cells = flat_level_cells(4);
        cells[(4 - 1) * 4 + (3 - 1)] = "4";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_x: 60.0,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.1, 3.25, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.horizontal, None);
        assert_eq!(frame.collisions.vertical, None);
        assert_close(player.body.x, 1.1 + player.movement.speed_x / 60.0);
        assert!(player.movement.speed_x > 0.0);
    }

    #[test]
    fn shell_player_frame_emits_ceiling_block_hit_intent_from_legacy_metadata() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-2";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.vertical.map(|collision| collision.coord),
            Some(LegacyMapTileCoord::new(2, 3)),
        );
        assert_eq!(frame.collisions.block_hits.len(), 1);
        assert_eq!(
            frame.collisions.block_hits[0].coord,
            LegacyMapTileCoord::new(2, 3),
        );
        assert_eq!(frame.collisions.block_hits[0].tile_id, TileId(5));
        assert!(frame.collisions.block_hits[0].breakable);
        assert!(frame.collisions.block_hits[0].coin_block);
        assert!(frame.collisions.block_hits[0].play_hit_sound);
        assert_eq!(frame.collisions.block_bounce_schedules.len(), 1);
        assert_eq!(
            frame.collisions.block_bounce_schedules[0].coord,
            LegacyMapTileCoord::new(2, 3),
        );
        assert_eq!(
            frame.collisions.block_bounce_schedules[0].schedule,
            LegacyBlockBounceSchedule {
                timer: LEGACY_BLOCK_BOUNCE_TIMER_START,
                coord: TileCoord::new(2, 3),
                spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
        );
        assert_eq!(
            frame.collisions.contained_reward_reveals,
            vec![LegacyRuntimeBlockContainedRewardRevealIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                content: LegacyBlockBounceContentKind::Mushroom,
                outcome: LegacyBlockContainedRewardRevealOutcome {
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(113),
                    },
                    sound: LegacyBlockRevealSound::MushroomAppear,
                },
            }],
        );
        assert!(frame.collisions.coin_block_rewards.is_empty());
        assert!(frame.block_bounce_progress.completions.is_empty());
        assert_eq!(shell.block_bounce_queue.len(), 1);
        assert_eq!(
            shell.block_bounce_queue[0],
            frame.collisions.block_bounce_schedules[0].schedule,
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::MushroomAppear),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::MushroomAppear),
            ],
        );
        assert_close(player.body.y, 3.0);
        assert_close(
            player.movement.speed_y,
            iw2wth_core::PhysicsConstants::default().head_force,
        );
        assert!(!player.movement.jumping);
        assert!(player.movement.falling);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_coin_block_animation_progress_emits_deferred_score_without_live_mutation() {
        let cells = flat_level_cells(3);
        let mut shell = loaded_test_shell(&cells);
        shell.score_count = 900;
        shell
            .coin_block_animations
            .push(LegacyCoinBlockAnimationState {
                x: 1.5,
                y: 2.0,
                timer: 0.0,
                frame: 31,
            });
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 6.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.25, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.coin_block_animation_progress.reports.len(), 1);
        assert_eq!(frame.coin_block_animation_progress.queue_len_after_prune, 0);
        assert_eq!(
            frame.coin_block_animation_progress.reports[0].score,
            Some(LegacyCoinBlockAnimationScore {
                score_delta: 0,
                floating_score: 200,
                x: 1.5,
                y: 2.0,
            }),
        );
        assert_eq!(
            frame.coin_block_animation_progress.reports[0].scrolling_score,
            Some(LegacyScrollingScoreState::spawn(
                LegacyScrollingScoreLabel::Points(200),
                1.5,
                2.0,
                0.25,
            )),
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::CoinBlockAnimation { source_index: 0 },
                score_count_before: 900,
                score_delta: 0,
                score_count_after: 900,
                scrolling_score: Some(LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(200),
                    1.5,
                    2.0,
                    0.25,
                )),
            }],
        );
        assert!(shell.coin_block_animations.is_empty());
        assert_eq!(shell.score_count, 900);
    }

    #[test]
    fn shell_queues_deferred_scrolling_score_and_progresses_it_on_following_frame() {
        let cells = flat_level_cells(3);
        let mut shell = loaded_test_shell(&cells);
        shell
            .coin_block_animations
            .push(LegacyCoinBlockAnimationState {
                x: 1.5,
                y: 2.0,
                timer: 0.0,
                frame: 31,
            });
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 6.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let first_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.25, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(
            first_frame
                .scrolling_score_animation_progress
                .reports
                .is_empty()
        );
        assert_eq!(shell.scrolling_score_animations.len(), 1);
        assert_eq!(
            shell.scrolling_score_animations[0],
            LegacyRuntimeScrollingScoreAnimationState {
                source: LegacyRuntimeScoreSource::CoinBlockAnimation { source_index: 0 },
                score: LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(200),
                    1.5,
                    2.0,
                    0.25,
                ),
            },
        );

        let second_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.25, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            second_frame
                .scrolling_score_animation_progress
                .reports
                .len(),
            1,
        );
        let report = second_frame.scrolling_score_animation_progress.reports[0];
        assert_eq!(
            report.source,
            LegacyRuntimeScoreSource::CoinBlockAnimation { source_index: 0 },
        );
        assert!(!report.remove);
        assert_close(report.state.timer, LEGACY_MAX_UPDATE_DT);
        assert_eq!(report.state.label, LegacyScrollingScoreLabel::Points(200));
        assert_close(report.presentation.x, 0.85);
        assert_close(
            report.presentation.y,
            2.0 - 1.5 - 2.5 * (LEGACY_MAX_UPDATE_DT / 0.8),
        );
        assert_eq!(
            second_frame
                .scrolling_score_animation_progress
                .queue_len_after_prune,
            1,
        );
    }

    #[test]
    fn shell_report_only_scrolling_score_progress_preserves_lua_offsets_and_pruning() {
        let mut queue = vec![
            LegacyRuntimeScrollingScoreAnimationState {
                source: LegacyRuntimeScoreSource::EnemyShotRequest {
                    block_coord: LegacyMapTileCoord::new(2, 3),
                    enemy_index: 7,
                },
                score: LegacyScrollingScoreState::spawn(
                    LegacyScrollingScoreLabel::Points(100),
                    1.5,
                    2.0,
                    0.25,
                ),
            },
            LegacyRuntimeScrollingScoreAnimationState {
                source: LegacyRuntimeScoreSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(4, 5),
                },
                score: LegacyScrollingScoreState {
                    x: 7.0,
                    y: 4.0,
                    label: LegacyScrollingScoreLabel::OneUp,
                    timer: 0.8,
                },
            },
        ];

        let report = super::progress_legacy_runtime_scrolling_score_animations(&mut queue, 0.001);

        assert_eq!(report.reports.len(), 2);
        assert_eq!(report.queue_len_after_prune, 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(
            queue[0].source,
            LegacyRuntimeScoreSource::EnemyShotRequest {
                block_coord: LegacyMapTileCoord::new(2, 3),
                enemy_index: 7,
            },
        );
        assert!(!report.reports[0].remove);
        assert_close(report.reports[0].state.timer, 0.001);
        assert_close(report.reports[0].presentation.x, 0.85);
        assert_close(report.reports[0].presentation.y, 0.5 - 2.5 * (0.001 / 0.8));
        assert_eq!(
            report.reports[0].presentation.label,
            LegacyScrollingScoreLabel::Points(100),
        );
        assert!(report.reports[1].remove);
        assert_close(report.reports[1].state.timer, 0.801);
        assert_close(report.reports[1].presentation.x, 7.0);
        assert_close(report.reports[1].presentation.y, 2.5 - 2.5 * (0.801 / 0.8));
        assert_eq!(
            report.reports[1].presentation.label,
            LegacyScrollingScoreLabel::OneUp,
        );
    }

    #[test]
    fn shell_plain_coin_block_hit_reports_reward_intent_without_mutating_counters_or_map() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.block_bounce_schedules.len(), 1);
        assert_eq!(
            frame.collisions.coin_block_rewards,
            vec![LegacyRuntimeCoinBlockRewardIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyCoinBlockRewardOutcome {
                    play_coin_sound: true,
                    animation: LegacyCoinBlockAnimationState::spawn(1.5, 2.0),
                    score_delta: 200,
                    coin_count: 1,
                    life_reward: None,
                    tile_change: Some(LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(113),
                    }),
                    start_many_coins_timer: None,
                },
            }],
        );
        assert!(frame.collisions.contained_reward_reveals.is_empty());
        assert_eq!(
            frame.tile_change_projections,
            vec![LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                tile_change: LegacyTileChange {
                    coord: TileCoord::new(2, 3),
                    tile: TileId(113),
                },
            }],
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
        assert_eq!(
            shell.coin_block_animations,
            vec![LegacyCoinBlockAnimationState::spawn(1.5, 2.0)],
        );
        assert!(frame.coin_block_animation_progress.reports.is_empty());
    }

    #[test]
    fn shell_coin_block_reward_reports_counter_intent_from_snapshot_without_mutating_coin_count() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.coin_count = 99;
        shell.player_count = 2;
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.coin_block_rewards[0].outcome.coin_count, 0);
        assert_eq!(
            frame.collisions.coin_block_rewards[0].outcome.life_reward,
            Some(LegacyCoinLifeReward {
                grant_lives_to_players: 2,
                respawn_players: true,
                play_sound: true,
            }),
        );
        assert_eq!(
            frame.coin_counter_intents,
            vec![LegacyRuntimeCoinCounterIntent {
                source: LegacyRuntimeCoinCounterSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                coin_count_before: 99,
                coin_count_after: 0,
                life_reward: Some(LegacyCoinLifeReward {
                    grant_lives_to_players: 2,
                    respawn_players: true,
                    play_sound: true,
                }),
                score_delta: 200,
            }],
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                score_count_before: 0,
                score_delta: 200,
                score_count_after: 200,
                scrolling_score: None,
            }],
        );
        assert_eq!(shell.coin_count, 99);
        assert_eq!(shell.score_count, 0);
    }

    #[test]
    fn shell_block_hit_reports_top_coin_collection_without_mutating_coin_state_or_map() {
        let mut cells = flat_level_cells(3);
        cells[4] = "3";
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.coin_count = 99;
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.top_coin_collections,
            vec![LegacyRuntimeBlockTopCoinCollectionIntent {
                block_coord: LegacyMapTileCoord::new(2, 3),
                coin_coord: LegacyMapTileCoord::new(2, 2),
                outcome: LegacyBlockTopCoinCollectionOutcome {
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 2),
                        tile: TileId(1),
                    },
                    play_coin_sound: true,
                    animation: LegacyCoinBlockAnimationState::spawn(1.5, 2.0),
                    score_delta: 200,
                    coin_count: 0,
                    life_reward: Some(LegacyCoinLifeReward {
                        grant_lives_to_players: 1,
                        respawn_players: true,
                        play_sound: true,
                    }),
                },
            }],
        );
        assert_eq!(
            frame.coin_counter_intents,
            vec![
                LegacyRuntimeCoinCounterIntent {
                    source: LegacyRuntimeCoinCounterSource::CoinBlockReward {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    coin_count_before: 99,
                    coin_count_after: 0,
                    life_reward: Some(LegacyCoinLifeReward {
                        grant_lives_to_players: 1,
                        respawn_players: true,
                        play_sound: true,
                    }),
                    score_delta: 200,
                },
                LegacyRuntimeCoinCounterIntent {
                    source: LegacyRuntimeCoinCounterSource::TopCoinCollection {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        coin_coord: LegacyMapTileCoord::new(2, 2),
                    },
                    coin_count_before: 99,
                    coin_count_after: 0,
                    life_reward: Some(LegacyCoinLifeReward {
                        grant_lives_to_players: 1,
                        respawn_players: true,
                        play_sound: true,
                    }),
                    score_delta: 200,
                }
            ],
        );
        assert_eq!(
            frame.tile_change_projections,
            vec![
                LegacyRuntimeTileChangeProjection {
                    source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(113),
                    },
                },
                LegacyRuntimeTileChangeProjection {
                    source: LegacyRuntimeTileChangeSource::TopCoinCollection {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        coin_coord: LegacyMapTileCoord::new(2, 2),
                    },
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 2),
                        tile: TileId(1),
                    },
                },
            ],
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::CoinBlockReward {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    score_count_before: 0,
                    score_delta: 200,
                    score_count_after: 200,
                    scrolling_score: None,
                },
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::TopCoinCollection {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        coin_coord: LegacyMapTileCoord::new(2, 2),
                    },
                    score_count_before: 200,
                    score_delta: 200,
                    score_count_after: 400,
                    scrolling_score: None,
                }
            ],
        );
        assert_eq!(shell.coin_count, 99);
        assert_eq!(shell.score_count, 0);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 2)),
            Some(TileId(3)),
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
    }

    #[test]
    fn shell_reward_tile_change_projection_feeds_future_map_queries_without_live_map_mutation() {
        let mut cells = flat_level_cells(3);
        cells[4] = "3";
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        let tiles = test_tiles_with_used_block();
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let request = LegacyRuntimeFrameRequest::new(
            1.0,
            0.2,
            LegacyRuntimeRenderContext::new(0.0, 1.0),
            None,
        );

        let mut first_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );
        let first_frame = shell.step_player_frame(
            &mut first_player,
            &BufferedLegacyInputSnapshot::new(),
            request,
            &tiles,
        );

        assert_eq!(
            first_frame.tile_change_projections,
            vec![
                LegacyRuntimeTileChangeProjection {
                    source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(113),
                    },
                },
                LegacyRuntimeTileChangeProjection {
                    source: LegacyRuntimeTileChangeSource::TopCoinCollection {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        coin_coord: LegacyMapTileCoord::new(2, 2),
                    },
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 2),
                        tile: TileId(1),
                    },
                },
            ],
        );
        assert_eq!(
            first_frame.projected_tile_change_state.projections,
            first_frame.tile_change_projections,
        );
        assert_eq!(
            shell.projected_tile_changes.projections,
            first_frame.tile_change_projections,
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 2)),
            Some(TileId(3)),
        );

        let mut second_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );
        let second_frame = shell.step_player_frame(
            &mut second_player,
            &BufferedLegacyInputSnapshot::new(),
            request,
            &tiles,
        );

        assert_eq!(second_frame.collisions.block_hits.len(), 1);
        assert_eq!(second_frame.collisions.block_hits[0].tile_id, TileId(113));
        assert!(!second_frame.collisions.block_hits[0].breakable);
        assert!(!second_frame.collisions.block_hits[0].coin_block);
        assert!(second_frame.collisions.coin_block_rewards.is_empty());
        assert!(second_frame.collisions.top_coin_collections.is_empty());
        assert!(second_frame.tile_change_projections.is_empty());
        assert_eq!(
            second_frame.projected_tile_change_state.projections,
            first_frame.tile_change_projections,
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 2)),
            Some(TileId(3)),
        );
    }

    #[test]
    fn shell_block_hit_reports_item_jump_requests_without_mutating_live_items() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.jump_items = vec![
            LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::Mushroom,
                7,
                0.5,
                1.5,
                1.0,
                0.5,
                true,
            ),
            LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::OneUp,
                8,
                1.5,
                1.5,
                1.0,
                0.5,
                true,
            ),
        ];
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.item_jump_requests,
            vec![
                LegacyRuntimeBlockItemJumpIntent {
                    coord: LegacyMapTileCoord::new(2, 3),
                    request: LegacyBlockItemJumpRequest {
                        kind: LegacyBlockJumpItemKind::Mushroom,
                        index: 7,
                        source_x: 2.0,
                    },
                },
                LegacyRuntimeBlockItemJumpIntent {
                    coord: LegacyMapTileCoord::new(2, 3),
                    request: LegacyBlockItemJumpRequest {
                        kind: LegacyBlockJumpItemKind::OneUp,
                        index: 8,
                        source_x: 2.0,
                    },
                },
            ],
        );
        assert_eq!(shell.jump_items.len(), 2);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_block_hit_item_jump_requests_require_handler_range_and_portal_guard() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.jump_items = vec![
            LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::Mushroom,
                1,
                0.499,
                1.5,
                1.0,
                0.5,
                true,
            ),
            LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::OneUp,
                2,
                0.5,
                1.499,
                1.0,
                0.5,
                true,
            ),
            LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::Mushroom,
                3,
                0.5,
                1.5,
                1.0,
                0.5,
                false,
            ),
        ];
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(frame.collisions.item_jump_requests.is_empty());

        let mut protected_shell = loaded_test_shell(&cells);
        protected_shell.jump_items = vec![LegacyRuntimeBlockJumpItemSnapshot::new(
            LegacyBlockJumpItemKind::Mushroom,
            4,
            0.5,
            1.5,
            1.0,
            0.5,
            true,
        )];
        protected_shell
            .block_portal_reservations
            .push(LegacyBlockPortalReservation::new(
                TileCoord::new(2, 3),
                Facing::Right,
            ));
        let mut protected_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let protected_frame = protected_shell.step_player_frame(
            &mut protected_player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(protected_frame.collisions.item_jump_requests.is_empty());
    }

    #[test]
    fn shell_block_hit_reports_enemy_shot_requests_without_mutating_live_enemies() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.top_enemies = vec![
            LegacyRuntimeBlockTopEnemySnapshot::new(7, 0.5, 1.5, 1.0, 0.5, true),
            LegacyRuntimeBlockTopEnemySnapshot::new(8, 1.0, 1.5, 1.0, 0.5, true),
            LegacyRuntimeBlockTopEnemySnapshot::new(7, 0.5, 1.5, 1.0, 0.5, true),
        ];
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.25, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.enemy_shot_requests,
            vec![
                LegacyRuntimeBlockEnemyShotIntent {
                    coord: LegacyMapTileCoord::new(2, 3),
                    request: LegacyBlockEnemyShotRequest {
                        index: 7,
                        direction: LegacyEnemyDirection::Left,
                        score_delta: 100,
                        score_x: 1.0,
                        score_y: 1.5,
                    },
                },
                LegacyRuntimeBlockEnemyShotIntent {
                    coord: LegacyMapTileCoord::new(2, 3),
                    request: LegacyBlockEnemyShotRequest {
                        index: 8,
                        direction: LegacyEnemyDirection::Right,
                        score_delta: 100,
                        score_x: 1.5,
                        score_y: 1.5,
                    },
                },
                LegacyRuntimeBlockEnemyShotIntent {
                    coord: LegacyMapTileCoord::new(2, 3),
                    request: LegacyBlockEnemyShotRequest {
                        index: 7,
                        direction: LegacyEnemyDirection::Left,
                        score_delta: 100,
                        score_x: 1.0,
                        score_y: 1.5,
                    },
                },
            ],
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::CoinBlockReward {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    score_count_before: 0,
                    score_delta: 200,
                    score_count_after: 200,
                    scrolling_score: None,
                },
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::EnemyShotRequest {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        enemy_index: 7,
                    },
                    score_count_before: 200,
                    score_delta: 100,
                    score_count_after: 300,
                    scrolling_score: Some(LegacyScrollingScoreState::spawn(
                        LegacyScrollingScoreLabel::Points(100),
                        1.0,
                        1.5,
                        0.25,
                    )),
                },
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::EnemyShotRequest {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        enemy_index: 8,
                    },
                    score_count_before: 300,
                    score_delta: 100,
                    score_count_after: 400,
                    scrolling_score: Some(LegacyScrollingScoreState::spawn(
                        LegacyScrollingScoreLabel::Points(100),
                        1.5,
                        1.5,
                        0.25,
                    )),
                },
                LegacyRuntimeScoreCounterIntent {
                    source: LegacyRuntimeScoreSource::EnemyShotRequest {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        enemy_index: 7,
                    },
                    score_count_before: 400,
                    score_delta: 100,
                    score_count_after: 500,
                    scrolling_score: Some(LegacyScrollingScoreState::spawn(
                        LegacyScrollingScoreLabel::Points(100),
                        1.0,
                        1.5,
                        0.25,
                    )),
                },
            ],
        );
        assert_eq!(shell.top_enemies.len(), 3);
        assert_eq!(shell.score_count, 0);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_block_hit_enemy_shot_requests_require_handler_range_bottom_and_portal_guard() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5";
        let mut shell = loaded_test_shell(&cells);
        shell.top_enemies = vec![
            LegacyRuntimeBlockTopEnemySnapshot::new(1, 0.499, 1.5, 1.0, 0.5, true),
            LegacyRuntimeBlockTopEnemySnapshot::new(2, 0.5, 1.499, 1.0, 0.5, true),
            LegacyRuntimeBlockTopEnemySnapshot::new(3, 0.5, 1.5, 1.0, 0.5, false),
        ];
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(frame.collisions.enemy_shot_requests.is_empty());

        let mut protected_shell = loaded_test_shell(&cells);
        protected_shell.top_enemies = vec![LegacyRuntimeBlockTopEnemySnapshot::new(
            4, 0.5, 1.5, 1.0, 0.5, true,
        )];
        protected_shell
            .block_portal_reservations
            .push(LegacyBlockPortalReservation::new(
                TileCoord::new(2, 3),
                Facing::Right,
            ));
        let mut protected_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let protected_frame = protected_shell.step_player_frame(
            &mut protected_player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(protected_frame.collisions.enemy_shot_requests.is_empty());
    }

    #[test]
    fn shell_big_player_hit_reports_empty_breakable_block_destroy_without_map_mutation() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "6";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        )
        .with_big_mario(true);

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.block_bounce_schedules.len(), 1);
        assert_eq!(
            frame.collisions.block_bounce_schedules[0]
                .schedule
                .hitter_size,
            2
        );
        assert_eq!(
            frame.collisions.empty_breakable_block_destroys,
            vec![LegacyRuntimeEmptyBreakableBlockDestroyIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyBreakableBlockOutcome::Broken(LegacyBrokenBlockEffects {
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(1),
                    },
                    remove_tile_collision_object: true,
                    clear_gels: true,
                    play_break_sound: true,
                    score_delta: 50,
                    debris: [
                        LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -23.0),
                        LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -23.0),
                        LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -14.0),
                        LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -14.0),
                    ],
                    regenerate_sprite_batch: true,
                }),
            }],
        );
        assert_eq!(
            frame.score_counter_intents,
            vec![LegacyRuntimeScoreCounterIntent {
                source: LegacyRuntimeScoreSource::EmptyBreakableBlockDestroy {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                score_count_before: 0,
                score_delta: 50,
                score_count_after: 50,
                scrolling_score: None,
            }],
        );
        assert_eq!(
            frame.tile_change_projections,
            vec![LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::EmptyBreakableBlockDestroy {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                tile_change: LegacyTileChange {
                    coord: TileCoord::new(2, 3),
                    tile: TileId(1),
                },
            }],
        );
        assert_eq!(
            frame.breakable_block_cleanup_projections,
            vec![
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::RemoveTileCollisionObject,
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::ClearGels,
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 0,
                        debris: LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -23.0),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 1,
                        debris: LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -23.0),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 2,
                        debris: LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -14.0),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 3,
                        debris: LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -14.0),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::RegenerateSpriteBatch,
                },
            ],
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockBreak),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockBreak),
            ],
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(6)),
        );
    }

    #[test]
    fn shell_progresses_report_only_block_debris_spawned_by_cleanup_on_following_frame() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "6";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        )
        .with_big_mario(true);

        let first_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(
            first_frame
                .block_debris_animation_progress
                .reports
                .is_empty()
        );
        assert_eq!(shell.block_debris_animations.len(), 4);
        player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 12.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        )
        .with_big_mario(true);

        let second_frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            second_frame.block_debris_animation_progress.reports.len(),
            4
        );
        assert_eq!(
            second_frame
                .block_debris_animation_progress
                .queue_len_after_prune,
            4,
        );
        let report = second_frame.block_debris_animation_progress.reports[0];
        assert_eq!(
            report.source,
            LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                coord: LegacyMapTileCoord::new(2, 3),
            },
        );
        assert_eq!(report.debris_index, 0);
        assert!(!report.remove);
        assert_eq!(report.state.frame, 1);
        assert_close(report.state.timer, LEGACY_MAX_UPDATE_DT);
        assert_close(report.state.x, 1.5 + 3.5 * LEGACY_MAX_UPDATE_DT);
        assert_close(report.state.speed_y, -23.0 + 60.0 * LEGACY_MAX_UPDATE_DT);
        assert_close(
            report.state.y,
            2.5 + report.state.speed_y * LEGACY_MAX_UPDATE_DT,
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(6)),
        );
    }

    #[test]
    fn shell_report_only_block_debris_progress_preserves_lua_update_and_prune_order() {
        let source = LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
            coord: LegacyMapTileCoord::new(2, 3),
        };
        let mut queue = vec![
            LegacyRuntimeBlockDebrisAnimationState {
                source,
                debris_index: 0,
                debris: LegacyBlockDebrisState::spawn(1.0, 15.0, 3.5, 0.0),
            },
            LegacyRuntimeBlockDebrisAnimationState {
                source,
                debris_index: 3,
                debris: LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -14.0),
            },
        ];

        let report = super::progress_legacy_runtime_block_debris_animations(&mut queue, 0.101);

        assert_eq!(report.reports.len(), 2);
        assert_eq!(report.queue_len_after_prune, 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].debris_index, 3);
        assert_eq!(report.reports[0].debris_index, 0);
        assert!(report.reports[0].remove);
        assert_eq!(report.reports[0].state.frame, 2);
        assert_close(report.reports[0].state.timer, 0.001);
        assert_close(report.reports[0].state.speed_y, 6.06);
        assert_close(report.reports[0].state.x, 1.3535);
        assert_close(report.reports[0].state.y, 15.61206);
        assert_eq!(report.reports[1].debris_index, 3);
        assert!(!report.reports[1].remove);
        assert_eq!(report.reports[1].state.frame, 2);
        assert_close(report.reports[1].state.timer, 0.001);
        assert_close(report.reports[1].state.speed_y, -7.94);
        assert_close(report.reports[1].state.x, 1.1465);
        assert_close(report.reports[1].state.y, 1.69806);
    }

    #[test]
    fn shell_report_only_scrolling_score_progress_preserves_lua_timer_and_presentation() {
        let source = LegacyRuntimeScoreSource::EnemyShotRequest {
            block_coord: LegacyMapTileCoord::new(2, 3),
            enemy_index: 7,
        };
        let mut queue = vec![LegacyRuntimeScrollingScoreAnimationState {
            source,
            score: LegacyScrollingScoreState::spawn(
                LegacyScrollingScoreLabel::Points(100),
                5.0,
                6.0,
                1.0,
            ),
        }];

        let report = super::progress_legacy_runtime_scrolling_score_animations(&mut queue, 0.4);

        assert_eq!(report.reports.len(), 1);
        assert_eq!(report.queue_len_after_prune, 1);
        assert_eq!(report.reports[0].source, source);
        assert!(!report.reports[0].remove);
        assert_close(report.reports[0].state.timer, 0.4);
        assert_close(report.reports[0].state.x, 4.0);
        assert_close(report.reports[0].presentation.x, 3.6);
        assert_close(report.reports[0].presentation.y, 3.25);
        assert_eq!(
            report.reports[0].presentation.label,
            LegacyScrollingScoreLabel::Points(100),
        );

        let report = super::progress_legacy_runtime_scrolling_score_animations(&mut queue, 0.401);

        assert_eq!(report.reports.len(), 1);
        assert!(report.reports[0].remove);
        assert_eq!(report.queue_len_after_prune, 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn shell_empty_breakable_block_destroy_reports_portal_protected_no_op() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "6";
        let mut shell = loaded_test_shell(&cells);
        shell
            .block_portal_reservations
            .push(LegacyBlockPortalReservation::new(
                TileCoord::new(2, 3),
                Facing::Right,
            ));
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        )
        .with_big_mario(true);

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.block_hits.len(), 1);
        assert!(!frame.collisions.block_hits[0].play_hit_sound);
        assert!(frame.collisions.block_bounce_schedules.is_empty());
        assert!(frame.collisions.coin_block_rewards.is_empty());
        assert!(frame.collisions.top_coin_collections.is_empty());
        assert!(frame.collisions.contained_reward_reveals.is_empty());
        assert_eq!(
            frame.collisions.empty_breakable_block_destroys,
            vec![LegacyRuntimeEmptyBreakableBlockDestroyIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyBreakableBlockOutcome::ProtectedByPortal,
            }],
        );
        assert!(frame.breakable_block_cleanup_projections.is_empty());
        assert!(frame.frame.audio_commands.is_empty());
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(6)),
        );
    }

    #[test]
    fn shell_block_hit_portal_guard_uses_projected_portal_state_without_mutating_explicit_guards() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "6";
        let mut shell = loaded_test_shell(&cells);
        shell
            .projected_portal_state
            .apply_projection(LegacyRuntimePortalReservationProjection {
                requested_slot: LegacyRuntimePortalSlot::Portal1,
                placement: LegacyRuntimePortalPlacement {
                    coord: LegacyMapTileCoord::new(2, 3),
                    side: Facing::Up,
                },
                tile_reservations: [LegacyMapTileCoord::new(2, 3), LegacyMapTileCoord::new(3, 3)],
                wall_reservations: [
                    LegacyRuntimePortalWallReservation::new(1, 3, 2, 0),
                    LegacyRuntimePortalWallReservation::new(1, 2, 0, 1),
                    LegacyRuntimePortalWallReservation::new(3, 2, 0, 1),
                ],
            });
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        )
        .with_big_mario(true);

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(shell.block_portal_reservations.is_empty());
        assert_eq!(frame.collisions.block_hits.len(), 1);
        assert!(!frame.collisions.block_hits[0].play_hit_sound);
        assert!(frame.collisions.block_bounce_schedules.is_empty());
        assert_eq!(
            frame.collisions.empty_breakable_block_destroys,
            vec![LegacyRuntimeEmptyBreakableBlockDestroyIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyBreakableBlockOutcome::ProtectedByPortal,
            }],
        );
        assert!(frame.frame.audio_commands.is_empty());
    }

    #[test]
    fn shell_contained_reward_reveal_reports_vine_sound_and_spriteset_tile_without_mutation() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-14";
        let mut shell = loaded_test_shell_with_properties(&cells, "background=1;spriteset=2");
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.contained_reward_reveals,
            vec![LegacyRuntimeBlockContainedRewardRevealIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                content: LegacyBlockBounceContentKind::Vine,
                outcome: LegacyBlockContainedRewardRevealOutcome {
                    tile_change: LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(114),
                    },
                    sound: LegacyBlockRevealSound::Vine,
                },
            }],
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::Vine),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Vine),
            ],
        );
        assert_eq!(
            frame.tile_change_projections,
            vec![LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::ContainedRewardReveal {
                    coord: LegacyMapTileCoord::new(2, 3),
                    content: LegacyBlockBounceContentKind::Vine,
                },
                tile_change: LegacyTileChange {
                    coord: TileCoord::new(2, 3),
                    tile: TileId(114),
                },
            }],
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_block_bounce_progress_uses_strict_lua_threshold_before_prune() {
        let cells = flat_level_cells(3);
        let mut shell = loaded_test_shell(&cells);
        shell.block_bounce_queue.push(LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(2, 3),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Star),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        });
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.01,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.block_bounce_progress.completions.len(), 1);
        assert_eq!(
            frame.block_bounce_progress.completions[0].coord,
            TileCoord::new(2, 3)
        );
        assert_close(
            frame.block_bounce_progress.completions[0].timer,
            LEGACY_BLOCK_BOUNCE_DURATION,
        );
        assert!(!frame.block_bounce_progress.completions[0].remove);
        assert_eq!(
            frame.block_bounce_progress.completions[0].suppressed_replay_spawn,
            None,
        );
        assert_eq!(
            frame.block_bounce_progress.completions[0].item_spawn_intent,
            None,
        );
        assert!(frame.block_bounce_progress.item_spawn_intents.is_empty());
        assert!(!frame.block_bounce_progress.regenerate_sprite_batch);
        assert_eq!(frame.block_bounce_progress.queue_len_after_prune, 1);
        assert_eq!(shell.block_bounce_queue.len(), 1);
    }

    #[test]
    fn shell_block_bounce_progress_reports_item_spawn_intent_without_mutating_map() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-2";
        let mut shell = loaded_test_shell(&cells);
        shell.block_bounce_queue.push(LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(2, 3),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
            hitter_size: 2,
            regenerate_sprite_batch: true,
        });
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.02,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.block_bounce_progress.completions.len(), 1);
        assert!(frame.block_bounce_progress.completions[0].remove);
        assert_eq!(
            frame.block_bounce_progress.completions[0].suppressed_replay_spawn,
            Some(LegacyBlockBounceReplaySpawn {
                kind: LegacyBlockBounceReplayKind::Flower,
                x: 1.5,
                y: 2.875,
            }),
        );
        assert_eq!(
            frame.block_bounce_progress.completions[0].item_spawn_intent,
            Some(LegacyRuntimeBlockBounceItemSpawnIntent {
                source_index: 0,
                source_coord: TileCoord::new(2, 3),
                spawn: LegacyBlockBounceReplaySpawn {
                    kind: LegacyBlockBounceReplayKind::Flower,
                    x: 1.5,
                    y: 2.875,
                },
            }),
        );
        assert_eq!(
            frame.block_bounce_progress.item_spawn_intents,
            vec![LegacyRuntimeBlockBounceItemSpawnIntent {
                source_index: 0,
                source_coord: TileCoord::new(2, 3),
                spawn: LegacyBlockBounceReplaySpawn {
                    kind: LegacyBlockBounceReplayKind::Flower,
                    x: 1.5,
                    y: 2.875,
                },
            }],
        );
        assert!(frame.block_bounce_progress.regenerate_sprite_batch);
        assert_eq!(frame.block_bounce_progress.queue_len_after_prune, 0);
        assert!(shell.block_bounce_queue.is_empty());
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_block_bounce_report_suppresses_many_coins_replay_and_keeps_map_unchanged() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-5";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(frame.collisions.block_bounce_schedules.len(), 1);
        assert!(frame.collisions.contained_reward_reveals.is_empty());
        assert_eq!(
            frame.collisions.block_bounce_schedules[0].schedule,
            LegacyBlockBounceSchedule {
                timer: LEGACY_BLOCK_BOUNCE_TIMER_START,
                coord: TileCoord::new(2, 3),
                spawn_content: None,
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
        );
        assert_eq!(frame.collisions.coin_block_rewards.len(), 1);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_many_coin_block_hit_reports_timer_spawn_without_mutating_live_timer_state() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-5";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.coin_block_rewards,
            vec![LegacyRuntimeCoinBlockRewardIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyCoinBlockRewardOutcome {
                    play_coin_sound: true,
                    animation: LegacyCoinBlockAnimationState::spawn(1.5, 2.0),
                    score_delta: 200,
                    coin_count: 1,
                    life_reward: None,
                    tile_change: None,
                    start_many_coins_timer: Some(LegacyCoinBlockTimerSpawn {
                        coord: TileCoord::new(2, 3),
                        duration: LegacyCoinBlockRewardConstants::default()
                            .many_coins_timer_duration,
                    }),
                },
            }],
        );
        assert!(shell.many_coins_timers.is_empty());
        assert!(frame.many_coins_timer_progress.reports.is_empty());
        assert_eq!(
            frame.many_coins_timer_progress.starts,
            vec![LegacyRuntimeManyCoinsTimerStartReport {
                reward_index: 0,
                coord: TileCoord::new(2, 3),
                duration: LegacyCoinBlockRewardConstants::default().many_coins_timer_duration,
            }],
        );
        assert_eq!(
            frame.many_coins_timer_progress.projected_timers,
            vec![LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(2, 3),
                remaining: LegacyCoinBlockRewardConstants::default().many_coins_timer_duration,
            }],
        );
        assert_eq!(
            frame.frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_many_coin_block_reward_reports_life_counter_intent_from_snapshot() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-5";
        let mut shell = loaded_test_shell(&cells);
        shell.coin_count = 99;
        shell.player_count = 3;
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.coin_block_rewards[0].outcome.life_reward,
            Some(LegacyCoinLifeReward {
                grant_lives_to_players: 3,
                respawn_players: true,
                play_sound: true,
            }),
        );
        assert_eq!(
            frame.coin_counter_intents,
            vec![LegacyRuntimeCoinCounterIntent {
                source: LegacyRuntimeCoinCounterSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                coin_count_before: 99,
                coin_count_after: 0,
                life_reward: Some(LegacyCoinLifeReward {
                    grant_lives_to_players: 3,
                    respawn_players: true,
                    play_sound: true,
                }),
                score_delta: 200,
            }],
        );
        assert_eq!(shell.coin_count, 99);
        assert!(shell.many_coins_timers.is_empty());
    }

    #[test]
    fn shell_many_coin_block_hit_reports_expired_timer_tile_change_without_map_mutation() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-5";
        let mut shell = loaded_test_shell_with_properties(&cells, "background=1;spriteset=2");
        shell.many_coins_timers.push(LegacyManyCoinsTimerEntry {
            coord: TileCoord::new(2, 3),
            remaining: 0.0,
        });
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.collisions.coin_block_rewards,
            vec![LegacyRuntimeCoinBlockRewardIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                outcome: LegacyCoinBlockRewardOutcome {
                    play_coin_sound: true,
                    animation: LegacyCoinBlockAnimationState::spawn(1.5, 2.0),
                    score_delta: 200,
                    coin_count: 1,
                    life_reward: None,
                    tile_change: Some(LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: TileId(114),
                    }),
                    start_many_coins_timer: None,
                },
            }],
        );
        assert_eq!(shell.many_coins_timers[0].remaining, 0.0);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_many_coins_timer_progress_reports_positive_countdown_without_live_mutation() {
        let cells = flat_level_cells(3);
        let mut shell = loaded_test_shell(&cells);
        shell.many_coins_timers = vec![
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(2, 3),
                remaining: 4.0,
            },
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(4, 5),
                remaining: 0.0,
            },
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(6, 7),
                remaining: -0.25,
            },
        ];
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.01,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.many_coins_timer_progress.reports,
            vec![
                LegacyRuntimeManyCoinsTimerUpdateReport {
                    index: 0,
                    coord: TileCoord::new(2, 3),
                    remaining_before: 4.0,
                    remaining_after: 3.99,
                },
                LegacyRuntimeManyCoinsTimerUpdateReport {
                    index: 1,
                    coord: TileCoord::new(4, 5),
                    remaining_before: 0.0,
                    remaining_after: 0.0,
                },
                LegacyRuntimeManyCoinsTimerUpdateReport {
                    index: 2,
                    coord: TileCoord::new(6, 7),
                    remaining_before: -0.25,
                    remaining_after: -0.25,
                },
            ],
        );
        assert!(frame.many_coins_timer_progress.starts.is_empty());
        assert_eq!(
            frame.many_coins_timer_progress.projected_timers,
            vec![
                LegacyManyCoinsTimerEntry {
                    coord: TileCoord::new(2, 3),
                    remaining: 3.99,
                },
                LegacyManyCoinsTimerEntry {
                    coord: TileCoord::new(4, 5),
                    remaining: 0.0,
                },
                LegacyManyCoinsTimerEntry {
                    coord: TileCoord::new(6, 7),
                    remaining: -0.25,
                },
            ],
        );
        assert_eq!(shell.many_coins_timers[0].remaining, 4.0);
        assert_eq!(shell.many_coins_timers[1].remaining, 0.0);
        assert_eq!(shell.many_coins_timers[2].remaining, -0.25);
    }

    #[test]
    fn shell_many_coins_timer_progress_preserves_negative_overshoot_for_same_frame_hit_lookup() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "5-5";
        let mut shell = loaded_test_shell_with_properties(&cells, "background=1;spriteset=2");
        shell.many_coins_timers.push(LegacyManyCoinsTimerEntry {
            coord: TileCoord::new(2, 3),
            remaining: 0.005,
        });
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                0.01,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert_eq!(
            frame.many_coins_timer_progress.reports,
            vec![LegacyRuntimeManyCoinsTimerUpdateReport {
                index: 0,
                coord: TileCoord::new(2, 3),
                remaining_before: 0.005,
                remaining_after: -0.005,
            }],
        );
        assert_eq!(
            frame.collisions.coin_block_rewards[0].outcome.tile_change,
            Some(LegacyTileChange {
                coord: TileCoord::new(2, 3),
                tile: TileId(114),
            }),
        );
        assert!(frame.many_coins_timer_progress.starts.is_empty());
        assert_eq!(
            frame.many_coins_timer_progress.projected_timers,
            vec![LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(2, 3),
                remaining: -0.005,
            }],
        );
        assert_eq!(shell.many_coins_timers[0].remaining, 0.005);
        assert_eq!(
            shell.map_query().tile_id_at(LegacyMapTileCoord::new(2, 3)),
            Some(TileId(5)),
        );
    }

    #[test]
    fn shell_player_frame_does_not_emit_block_hit_for_invisible_ceiling_tiles() {
        let mut cells = flat_level_cells(3);
        cells[(3 - 1) * 3 + (2 - 1)] = "4";
        let mut shell = loaded_test_shell(&cells);
        let movement = PlayerMovementState {
            speed_y: -80.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            movement,
        );

        let frame = shell.step_player_frame(
            &mut player,
            &BufferedLegacyInputSnapshot::new(),
            LegacyRuntimeFrameRequest::new(
                1.0,
                0.2,
                LegacyRuntimeRenderContext::new(0.0, 1.0),
                None,
            ),
            &test_tiles(),
        );

        assert!(frame.collisions.block_hits.is_empty());
        assert!(frame.frame.audio_commands.is_empty());
    }
}

//! Runnable local harness for the current one-level runtime shell.
//!
//! This is intentionally adapter-side code. It wires the filesystem/source,
//! input snapshot, frame clock, map query, and one-player frame integration
//! without claiming ownership of the remaining gameplay loop.

use std::{error::Error, fmt};

use iw2wth_core::{
    LegacyBlockRevealSound, LegacyBreakableBlockOutcome, LegacyEntityKind, LegacyFireballState,
    LegacyManyCoinsTimerEntry, LegacyMapTileCoord, Mari0Level, PlayerBodyBounds,
    PlayerMovementInput, PlayerMovementState, TileId,
};

use crate::{
    assets::LegacyAssetSource,
    audio::{LegacyAudioCommand, LegacySoundEffect, legacy_play_sound_commands},
    input::{BufferedLegacyInputSnapshot, LegacyControlBinding, LegacyPlayerControls},
    map::LegacyMapQuery,
    render::LegacyColor,
    shell::{
        LegacyRuntimeBlockBounceCompletionReport, LegacyRuntimeBlockBounceItemSpawnIntent,
        LegacyRuntimeBlockContainedRewardRevealIntent,
        LegacyRuntimeBlockDebrisAnimationUpdateReport, LegacyRuntimeBlockEnemyShotIntent,
        LegacyRuntimeBlockItemJumpIntent, LegacyRuntimeBlockJumpItemSnapshot,
        LegacyRuntimeBlockTopCoinCollectionIntent, LegacyRuntimeBlockTopEnemySnapshot,
        LegacyRuntimeBreakableBlockCleanupProjection, LegacyRuntimeCoinBlockAnimationUpdateReport,
        LegacyRuntimeCoinBlockRewardIntent, LegacyRuntimeCoinCounterIntent,
        LegacyRuntimeEmptyBreakableBlockDestroyIntent, LegacyRuntimeFireballCollisionProbe,
        LegacyRuntimeFireballCollisionProbeRequest, LegacyRuntimeFireballCollisionProbeSource,
        LegacyRuntimeFireballEnemyHitIntent, LegacyRuntimeFireballEnemySnapshot,
        LegacyRuntimeFireballLaunchIntent, LegacyRuntimeFireballLaunchSnapshot,
        LegacyRuntimeFireballMapTargetProbe, LegacyRuntimeFireballProjectileReleaseSummary,
        LegacyRuntimeFireballProjectileUpdateReport, LegacyRuntimeFireballRenderIntentPreview,
        LegacyRuntimeFrameRequest, LegacyRuntimeLevelSelection, LegacyRuntimeLoadError,
        LegacyRuntimeManyCoinsTimerStartReport, LegacyRuntimeManyCoinsTimerUpdateReport,
        LegacyRuntimePlayer, LegacyRuntimePlayerBlockBounceSchedule,
        LegacyRuntimePlayerCeilingBlockHit, LegacyRuntimePlayerCoinPickup,
        LegacyRuntimePlayerCollisionReport, LegacyRuntimePlayerRenderIntentPreview,
        LegacyRuntimePlayerTileCollision, LegacyRuntimePortalAimSnapshot,
        LegacyRuntimePortalBlockGuard, LegacyRuntimePortalBlockGuardSource,
        LegacyRuntimePortalCoordsPreviewReport, LegacyRuntimePortalOutcomeIntent,
        LegacyRuntimePortalOutcomeKind, LegacyRuntimePortalPairReadinessSummary,
        LegacyRuntimePortalPlacement, LegacyRuntimePortalReplacementSummary,
        LegacyRuntimePortalReservationProjection, LegacyRuntimePortalSlot,
        LegacyRuntimePortalTargetPlayerSource, LegacyRuntimePortalTargetProbe,
        LegacyRuntimePortalTraceHit, LegacyRuntimePortalTransitAudioIntent,
        LegacyRuntimePortalTransitCandidateProbe, LegacyRuntimePortalTransitOutcomeKind,
        LegacyRuntimePortalTransitOutcomeSummary, LegacyRuntimeProjectedFireballCountSnapshot,
        LegacyRuntimeProjectedFireballCountState, LegacyRuntimeProjectedFireballEnemyHitSnapshot,
        LegacyRuntimeProjectedFireballEnemyHitState,
        LegacyRuntimeProjectedFireballProjectileCollisionSnapshot,
        LegacyRuntimeProjectedFireballProjectileCollisionState, LegacyRuntimeProjectedPlayerState,
        LegacyRuntimeProjectedPlayerStateSnapshot, LegacyRuntimeProjectedPortalState,
        LegacyRuntimeRenderContext, LegacyRuntimeScoreCounterIntent, LegacyRuntimeScoreSource,
        LegacyRuntimeScrollingScoreAnimationUpdateReport, LegacyRuntimeShell,
        LegacyRuntimeTileChangeProjection,
    },
    tiles::{LegacyTileMetadataLoadError, LegacyTileMetadataTable},
    time::LEGACY_MAX_UPDATE_DT,
};

pub const LEGACY_RUNTIME_HARNESS_PARITY_GAPS: &[&str] = &[
    "rendering and audio are emitted as intents only",
    "objects, entities, portals, block effect execution, live reward spawning, and broad gameplay systems remain Lua-owned",
];

pub const LEGACY_RUNTIME_HARNESS_GRAPHICS_PACK: &str = "SMB";

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeHarnessConfig {
    pub selection: LegacyRuntimeLevelSelection,
    pub frames: usize,
    pub raw_dt: f32,
    pub joystick_deadzone: f32,
    pub render: LegacyRuntimeRenderContext,
    pub input: LegacyRuntimeHarnessInput,
    pub initial_player: LegacyRuntimePlayer,
    pub initial_fireball_projectiles: Vec<LegacyFireballState>,
    pub fireball_collision_probe: Option<LegacyRuntimeFireballCollisionProbeRequest>,
    pub fireball_enemies: Vec<LegacyRuntimeFireballEnemySnapshot>,
    pub jump_items: Vec<LegacyRuntimeBlockJumpItemSnapshot>,
    pub top_enemies: Vec<LegacyRuntimeBlockTopEnemySnapshot>,
    pub many_coins_timers: Vec<LegacyManyCoinsTimerEntry>,
    pub coin_count: u32,
    pub score_count: u32,
    pub life_count_enabled: bool,
    pub player_count: usize,
    pub force_initial_player_seed: bool,
    pub initial_projected_portal_state: LegacyRuntimeProjectedPortalState,
    pub initial_projected_player_state: LegacyRuntimeProjectedPlayerState,
}

impl Default for LegacyRuntimeHarnessConfig {
    fn default() -> Self {
        Self {
            selection: LegacyRuntimeLevelSelection::new("smb", "1-1", 1, 1, 0),
            frames: 5,
            raw_dt: LEGACY_MAX_UPDATE_DT,
            joystick_deadzone: 0.2,
            render: LegacyRuntimeRenderContext::new(0.0, 1.0),
            input: LegacyRuntimeHarnessInput::default(),
            initial_player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            initial_fireball_projectiles: Vec::new(),
            fireball_collision_probe: None,
            fireball_enemies: Vec::new(),
            jump_items: Vec::new(),
            top_enemies: Vec::new(),
            many_coins_timers: Vec::new(),
            coin_count: 0,
            score_count: 0,
            life_count_enabled: true,
            player_count: 1,
            force_initial_player_seed: false,
            initial_projected_portal_state: LegacyRuntimeProjectedPortalState::default(),
            initial_projected_player_state: LegacyRuntimeProjectedPlayerState::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeHarnessInput {
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub fire: bool,
    pub fire_flower_power: bool,
    pub fire_ducking: bool,
    pub active_fireball_count: usize,
    pub portal_1: bool,
    pub portal_2: bool,
    pub pointing_angle: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeHarnessReport {
    pub selection: LegacyRuntimeLevelSelection,
    pub settings_name: Option<String>,
    pub level_width: usize,
    pub player_spawn: LegacyRuntimePlayerSpawn,
    pub frame_count: usize,
    pub custom_tiles: bool,
    pub background_count: usize,
    pub portalability: LegacyRuntimePortalabilitySummary,
    pub final_player: LegacyRuntimePlayer,
    pub player_render_preview_count: usize,
    pub player_render_preview_detail_summary: LegacyRuntimePlayerRenderPreviewDetailSummary,
    pub last_frame: Option<LegacyRuntimeHarnessFrame>,
    pub frame_audio_command_count: usize,
    pub frame_audio_command_detail_summary: LegacyRuntimeFrameAudioCommandDetailSummary,
    pub fireball_launch_intent_count: usize,
    pub fireball_launch_detail_summary: LegacyRuntimeFireballLaunchDetailSummary,
    pub fireball_projectile_progress_count: usize,
    pub fireball_projectile_prune_count: usize,
    pub fireball_projectile_detail_summary: LegacyRuntimeFireballProjectileDetailSummary,
    pub fireball_render_preview_count: usize,
    pub fireball_render_preview_suppressed_count: usize,
    pub fireball_render_preview_detail_summary: LegacyRuntimeFireballRenderPreviewDetailSummary,
    pub fireball_map_target_probe_count: usize,
    pub fireball_map_target_detail_summary: LegacyRuntimeFireballMapTargetDetailSummary,
    pub fireball_collision_probe_count: usize,
    pub fireball_collision_release_summary_count: usize,
    pub fireball_collision_detail_summary: LegacyRuntimeFireballCollisionDetailSummary,
    pub projected_fireball_projectile_collision_snapshot_count: usize,
    pub projected_fireball_projectile_collision_detail_summary:
        LegacyRuntimeProjectedFireballProjectileCollisionDetailSummary,
    pub fireball_enemy_hit_intent_count: usize,
    pub fireball_enemy_hit_detail_summary: LegacyRuntimeFireballEnemyHitDetailSummary,
    pub projected_fireball_enemy_hit_snapshot_count: usize,
    pub projected_fireball_enemy_hit_detail_summary:
        LegacyRuntimeProjectedFireballEnemyHitDetailSummary,
    pub projected_fireball_count_snapshot_count: usize,
    pub projected_fireball_count_detail_summary: LegacyRuntimeProjectedFireballCountDetailSummary,
    pub coin_pickup_count: usize,
    pub player_coin_pickup_detail_summary: LegacyRuntimePlayerCoinPickupDetailSummary,
    pub coin_counter_intent_count: usize,
    pub score_counter_intent_count: usize,
    pub scrolling_score_intent_count: usize,
    pub life_reward_counter_intent_count: usize,
    pub coin_counter_detail_summary: LegacyRuntimeCoinCounterDetailSummary,
    pub life_reward_counter_detail_summary: LegacyRuntimeLifeRewardCounterDetailSummary,
    pub score_counter_detail_summary: LegacyRuntimeScoreCounterDetailSummary,
    pub scrolling_score_detail_summary: LegacyRuntimeScrollingScoreDetailSummary,
    pub horizontal_collision_count: usize,
    pub vertical_collision_count: usize,
    pub ceiling_block_hit_count: usize,
    pub tile_collision_detail_summary: LegacyRuntimeTileCollisionDetailSummary,
    pub block_hit_portal_guard_summary: LegacyRuntimeBlockHitPortalGuardDetailSummary,
    pub block_hit_portal_guard_suppression_count: usize,
    pub block_hit_projected_portal_guard_suppression_count: usize,
    pub block_bounce_schedule_count: usize,
    pub block_bounce_detail_summary: LegacyRuntimeBlockBounceDetailSummary,
    pub coin_block_reward_intent_count: usize,
    pub top_coin_collection_intent_count: usize,
    pub coin_block_reward_detail_summary: LegacyRuntimeCoinBlockRewardDetailSummary,
    pub tile_change_projection_count: usize,
    pub projected_tile_change_snapshot_count: usize,
    pub tile_change_projection_detail_summary: LegacyRuntimeTileChangeProjectionDetailSummary,
    pub breakable_block_cleanup_projection_count: usize,
    pub breakable_block_cleanup_projection_detail_summary:
        LegacyRuntimeBreakableBlockCleanupProjectionDetailSummary,
    pub coin_block_animation_progress_count: usize,
    pub coin_block_animation_prune_count: usize,
    pub block_debris_animation_progress_count: usize,
    pub block_debris_animation_prune_count: usize,
    pub scrolling_score_animation_progress_count: usize,
    pub scrolling_score_animation_prune_count: usize,
    pub effect_animation_detail_summary: LegacyRuntimeEffectAnimationDetailSummary,
    pub item_jump_request_intent_count: usize,
    pub enemy_shot_request_intent_count: usize,
    pub item_jump_request_detail_summary: LegacyRuntimeItemJumpRequestDetailSummary,
    pub enemy_shot_request_detail_summary: LegacyRuntimeEnemyShotRequestDetailSummary,
    pub empty_breakable_block_destroy_intent_count: usize,
    pub block_bounce_progress_count: usize,
    pub block_bounce_prune_count: usize,
    pub block_bounce_item_spawn_intent_count: usize,
    pub many_coins_timer_progress_count: usize,
    pub many_coins_timer_start_count: usize,
    pub many_coins_timer_detail_summary: LegacyRuntimeManyCoinsTimerDetailSummary,
    pub contained_reward_reveal_intent_count: usize,
    pub contained_reward_reveal_detail_summary: LegacyRuntimeContainedRewardRevealDetailSummary,
    pub portal_target_source_selection: LegacyRuntimePortalTargetSourceSelectionSummary,
    pub portal_target_placement_summary: LegacyRuntimePortalTargetPlacementSummary,
    pub portal_open_outcome_summary: LegacyRuntimePortalOpenOutcomeSummary,
    pub portal_reservation_projection_summary: LegacyRuntimePortalReservationProjectionSummary,
    pub portal_replacement_detail_summary: LegacyRuntimePortalReplacementDetailSummary,
    pub portal_pair_readiness_detail_summary: LegacyRuntimePortalPairReadinessDetailSummary,
    pub portal_transit_candidate_detail_summary: LegacyRuntimePortalTransitCandidateDetailSummary,
    pub portalcoords_preview_detail_summary: LegacyRuntimePortalCoordsPreviewDetailSummary,
    pub portal_transit_outcome_detail_summary: LegacyRuntimePortalTransitOutcomeDetailSummary,
    pub portal_transit_audio_detail_summary: LegacyRuntimePortalTransitAudioDetailSummary,
    pub projected_player_state_detail_summary: LegacyRuntimeProjectedPlayerStateDetailSummary,
    pub portal_target_probe_count: usize,
    pub portal_target_projected_player_source_count: usize,
    pub portal_target_possible_count: usize,
    pub portal_open_intent_count: usize,
    pub portal_fizzle_intent_count: usize,
    pub portal_reservation_projection_count: usize,
    pub portal_replacement_summary_count: usize,
    pub projected_portal_state_snapshot_count: usize,
    pub portal_pair_readiness_summary_count: usize,
    pub portal_pair_ready_count: usize,
    pub portal_transit_candidate_probe_count: usize,
    pub portal_transit_candidate_ready_count: usize,
    pub portalcoords_preview_count: usize,
    pub portal_transit_outcome_summary_count: usize,
    pub portal_transit_audio_intent_count: usize,
    pub portal_transit_success_preview_count: usize,
    pub portal_transit_blocked_exit_bounce_preview_count: usize,
    pub portal_transit_projected_player_snapshot_count: usize,
    pub projected_player_state_snapshot_count: usize,
    pub parity_gaps: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalTargetSourceSelectionSummary {
    pub live_player_count: usize,
    pub projected_portal_transit_count: usize,
    pub last_selection: Option<LegacyRuntimePortalTargetSourceSelection>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTargetSourceSelection {
    pub frame_index: usize,
    pub player_source: LegacyRuntimePortalTargetPlayerSource,
    pub source_x: f32,
    pub source_y: f32,
    pub pointing_angle: f32,
    pub requested_slot: Option<crate::shell::LegacyRuntimePortalSlot>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalTargetPlacementSummary {
    pub possible_count: usize,
    pub impossible_count: usize,
    pub last_summary: Option<LegacyRuntimePortalTargetPlacementDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTargetPlacementDetail {
    pub frame_index: usize,
    pub requested_slot: Option<LegacyRuntimePortalSlot>,
    pub trace_hit: Option<LegacyRuntimePortalTraceHit>,
    pub placement: Option<LegacyRuntimePortalPlacement>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalOpenOutcomeSummary {
    pub open_count: usize,
    pub fizzle_count: usize,
    pub last_summary: Option<LegacyRuntimePortalOpenOutcomeDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalOpenOutcomeDetail {
    pub frame_index: usize,
    pub requested_slot: LegacyRuntimePortalSlot,
    pub kind: LegacyRuntimePortalOutcomeKind,
    pub placement: Option<LegacyRuntimePortalPlacement>,
    pub sound: LegacySoundEffect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalReservationProjectionSummary {
    pub projection_count: usize,
    pub last_projection: Option<LegacyRuntimePortalReservationProjectionDetail>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalReservationProjectionDetail {
    pub frame_index: usize,
    pub projection: LegacyRuntimePortalReservationProjection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalReplacementDetailSummary {
    pub replacement_count: usize,
    pub last_replacement: Option<LegacyRuntimePortalReplacementDetail>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalReplacementDetail {
    pub frame_index: usize,
    pub summary: LegacyRuntimePortalReplacementSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalPairReadinessDetailSummary {
    pub last_summary: Option<LegacyRuntimePortalPairReadinessDetail>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimePortalPairReadinessDetail {
    pub frame_index: usize,
    pub summary: LegacyRuntimePortalPairReadinessSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalTransitCandidateDetailSummary {
    pub last_probe: Option<LegacyRuntimePortalTransitCandidateDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitCandidateDetail {
    pub frame_index: usize,
    pub probe: LegacyRuntimePortalTransitCandidateProbe,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalCoordsPreviewDetailSummary {
    pub last_preview: Option<LegacyRuntimePortalCoordsPreviewDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalCoordsPreviewDetail {
    pub frame_index: usize,
    pub preview: LegacyRuntimePortalCoordsPreviewReport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalTransitOutcomeDetailSummary {
    pub last_summary: Option<LegacyRuntimePortalTransitOutcomeDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitOutcomeDetail {
    pub frame_index: usize,
    pub summary: LegacyRuntimePortalTransitOutcomeSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePortalTransitAudioDetailSummary {
    pub last_intent: Option<LegacyRuntimePortalTransitAudioDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePortalTransitAudioDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimePortalTransitAudioIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedPlayerStateDetailSummary {
    pub last_snapshot: Option<LegacyRuntimeProjectedPlayerStateDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedPlayerStateDetail {
    pub frame_index: usize,
    pub snapshot: LegacyRuntimeProjectedPlayerStateSnapshot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeTileCollisionDetailSummary {
    pub last_horizontal: Option<LegacyRuntimeTileCollisionDetail>,
    pub last_vertical: Option<LegacyRuntimeTileCollisionDetail>,
    pub last_block_hit: Option<LegacyRuntimeCeilingBlockHitDetail>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePlayerCoinPickupDetailSummary {
    pub last_pickup: Option<LegacyRuntimePlayerCoinPickupDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerCoinPickupDetail {
    pub frame_index: usize,
    pub pickup: LegacyRuntimePlayerCoinPickup,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballLaunchDetailSummary {
    pub last_intent: Option<LegacyRuntimeFireballLaunchDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballLaunchDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeFireballLaunchIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimePlayerRenderPreviewDetailSummary {
    pub last_preview: Option<LegacyRuntimePlayerRenderPreviewDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerRenderPreviewDetail {
    pub frame_index: usize,
    pub preview: LegacyRuntimePlayerRenderIntentPreview,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballProjectileDetailSummary {
    pub last_progress: Option<LegacyRuntimeFireballProjectileDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballProjectileDetail {
    pub frame_index: usize,
    pub progress: LegacyRuntimeFireballProjectileUpdateReport,
    pub queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballRenderPreviewDetailSummary {
    pub last_preview: Option<LegacyRuntimeFireballRenderPreviewDetail>,
    pub last_suppressed_projected_removal_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballRenderPreviewDetail {
    pub frame_index: usize,
    pub preview: LegacyRuntimeFireballRenderIntentPreview,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballMapTargetDetailSummary {
    pub last_probe: Option<LegacyRuntimeFireballMapTargetDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballMapTargetDetail {
    pub frame_index: usize,
    pub probe: LegacyRuntimeFireballMapTargetProbe,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballCollisionDetailSummary {
    pub last_probe: Option<LegacyRuntimeFireballCollisionDetail>,
    pub last_explicit_probe: Option<LegacyRuntimeFireballCollisionDetail>,
    pub last_map_derived_probe: Option<LegacyRuntimeFireballCollisionDetail>,
    pub last_enemy_overlap_probe: Option<LegacyRuntimeFireballCollisionDetail>,
    pub explicit_probe_count: usize,
    pub map_derived_probe_count: usize,
    pub enemy_overlap_probe_count: usize,
    pub last_release_summary: Option<LegacyRuntimeFireballReleaseDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballCollisionDetail {
    pub frame_index: usize,
    pub probe: LegacyRuntimeFireballCollisionProbe,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballReleaseDetail {
    pub frame_index: usize,
    pub summary: LegacyRuntimeFireballProjectileReleaseSummary,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedFireballProjectileCollisionDetailSummary {
    pub last_snapshot: Option<LegacyRuntimeProjectedFireballProjectileCollisionDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedFireballProjectileCollisionDetail {
    pub frame_index: usize,
    pub snapshot: LegacyRuntimeProjectedFireballProjectileCollisionSnapshot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFireballEnemyHitDetailSummary {
    pub last_intent: Option<LegacyRuntimeFireballEnemyHitDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeFireballEnemyHitDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeFireballEnemyHitIntent,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedFireballEnemyHitDetailSummary {
    pub last_snapshot: Option<LegacyRuntimeProjectedFireballEnemyHitDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedFireballEnemyHitDetail {
    pub frame_index: usize,
    pub snapshot: LegacyRuntimeProjectedFireballEnemyHitSnapshot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeProjectedFireballCountDetailSummary {
    pub last_snapshot: Option<LegacyRuntimeProjectedFireballCountDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeProjectedFireballCountDetail {
    pub frame_index: usize,
    pub snapshot: LegacyRuntimeProjectedFireballCountSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeFrameAudioCommandDetailSummary {
    pub last_commands: Option<LegacyRuntimeFrameAudioCommandDetail>,
    pub fireball_launch_command_count: usize,
    pub block_hit_command_count: usize,
    pub reward_reveal_command_count: usize,
    pub coin_block_reward_command_count: usize,
    pub top_coin_collection_command_count: usize,
    pub block_break_command_count: usize,
    pub fireball_collision_command_count: usize,
    pub portal_outcome_command_count: usize,
    pub portal_transit_command_count: usize,
    pub last_block_hit: Option<LegacyRuntimeFrameAudioBlockHitCommandDetail>,
    pub last_reward_reveal: Option<LegacyRuntimeFrameAudioRewardRevealCommandDetail>,
    pub last_coin_block_reward: Option<LegacyRuntimeFrameAudioCoinBlockRewardCommandDetail>,
    pub last_top_coin_collection: Option<LegacyRuntimeFrameAudioTopCoinCollectionCommandDetail>,
    pub last_block_break: Option<LegacyRuntimeFrameAudioBlockBreakCommandDetail>,
    pub last_fireball_collision: Option<LegacyRuntimeFrameAudioFireballCollisionCommandDetail>,
    pub last_portal_outcome: Option<LegacyRuntimeFrameAudioPortalOutcomeCommandDetail>,
    pub last_portal_transit: Option<LegacyRuntimeFrameAudioPortalTransitCommandDetail>,
    pub last_fireball_launch: Option<LegacyRuntimeFrameAudioFireballLaunchCommandDetail>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioCommandDetail {
    pub frame_index: usize,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioFireballLaunchCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeFireballLaunchIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioBlockHitCommandDetail {
    pub frame_index: usize,
    pub block_hit: LegacyRuntimePlayerCeilingBlockHit,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioRewardRevealCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockContainedRewardRevealIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioCoinBlockRewardCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeCoinBlockRewardIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioTopCoinCollectionCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockTopCoinCollectionIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioBlockBreakCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeEmptyBreakableBlockDestroyIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioFireballCollisionCommandDetail {
    pub frame_index: usize,
    pub probe: LegacyRuntimeFireballCollisionProbe,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioPortalOutcomeCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimePortalOutcomeIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeFrameAudioPortalTransitCommandDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimePortalTransitAudioIntent,
    pub commands: Vec<LegacyAudioCommand>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeTileCollisionDetail {
    pub frame_index: usize,
    pub collision: LegacyRuntimePlayerTileCollision,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCeilingBlockHitDetail {
    pub frame_index: usize,
    pub block_hit: LegacyRuntimePlayerCeilingBlockHit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeBlockHitPortalGuardDetailSummary {
    pub explicit_reservation_count: usize,
    pub projected_portal_state_count: usize,
    pub last_guard: Option<LegacyRuntimeBlockHitPortalGuardDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockHitPortalGuardDetail {
    pub frame_index: usize,
    pub coord: LegacyMapTileCoord,
    pub tile_id: TileId,
    pub breakable: bool,
    pub coin_block: bool,
    pub play_hit_sound: bool,
    pub guard: LegacyRuntimePortalBlockGuard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeItemJumpRequestDetailSummary {
    pub last_intent: Option<LegacyRuntimeItemJumpRequestDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeItemJumpRequestDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockItemJumpIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeEnemyShotRequestDetailSummary {
    pub last_intent: Option<LegacyRuntimeEnemyShotRequestDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeEnemyShotRequestDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockEnemyShotIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeBlockBounceDetailSummary {
    pub last_schedule: Option<LegacyRuntimeBlockBounceScheduleDetail>,
    pub last_completion: Option<LegacyRuntimeBlockBounceCompletionDetail>,
    pub last_item_spawn: Option<LegacyRuntimeBlockBounceItemSpawnDetail>,
    pub queue_len_after_prune: usize,
    pub regenerate_sprite_batch_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockBounceScheduleDetail {
    pub frame_index: usize,
    pub schedule: LegacyRuntimePlayerBlockBounceSchedule,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockBounceCompletionDetail {
    pub frame_index: usize,
    pub completion: LegacyRuntimeBlockBounceCompletionReport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockBounceItemSpawnDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockBounceItemSpawnIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeContainedRewardRevealDetailSummary {
    pub last_intent: Option<LegacyRuntimeContainedRewardRevealDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeContainedRewardRevealDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockContainedRewardRevealIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerDetailSummary {
    pub last_progress: Option<LegacyRuntimeManyCoinsTimerProgressDetail>,
    pub last_start: Option<LegacyRuntimeManyCoinsTimerStartDetail>,
    pub projected_timer_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerProgressDetail {
    pub frame_index: usize,
    pub report: LegacyRuntimeManyCoinsTimerUpdateReport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeManyCoinsTimerStartDetail {
    pub frame_index: usize,
    pub report: LegacyRuntimeManyCoinsTimerStartReport,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeTileChangeProjectionDetailSummary {
    pub projection_count: usize,
    pub projected_snapshot_count: usize,
    pub last_projection: Option<LegacyRuntimeTileChangeProjectionDetail>,
    pub last_frame_projections: Vec<LegacyRuntimeTileChangeProjection>,
    pub projected_snapshot: Vec<LegacyRuntimeTileChangeProjection>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyRuntimeTileChangeProjectionDetail {
    pub frame_index: usize,
    pub projection: LegacyRuntimeTileChangeProjection,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyRuntimeBreakableBlockCleanupProjectionDetailSummary {
    pub projection_count: usize,
    pub last_projection: Option<LegacyRuntimeBreakableBlockCleanupProjectionDetail>,
    pub last_frame_projections: Vec<LegacyRuntimeBreakableBlockCleanupProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBreakableBlockCleanupProjectionDetail {
    pub frame_index: usize,
    pub projection: LegacyRuntimeBreakableBlockCleanupProjection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeCoinCounterDetailSummary {
    pub last_intent: Option<LegacyRuntimeCoinCounterDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCoinCounterDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeCoinCounterIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeLifeRewardCounterDetailSummary {
    pub last_intent: Option<LegacyRuntimeLifeRewardCounterDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeLifeRewardCounterDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeCoinCounterIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeScoreCounterDetailSummary {
    pub last_intent: Option<LegacyRuntimeScoreCounterDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScoreCounterDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeScoreCounterIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeScrollingScoreDetailSummary {
    pub last_intent: Option<LegacyRuntimeScrollingScoreDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScrollingScoreDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeScoreCounterIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeCoinBlockRewardDetailSummary {
    pub last_reward: Option<LegacyRuntimeCoinBlockRewardDetail>,
    pub last_top_coin_collection: Option<LegacyRuntimeTopCoinCollectionDetail>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCoinBlockRewardDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeCoinBlockRewardIntent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeTopCoinCollectionDetail {
    pub frame_index: usize,
    pub intent: LegacyRuntimeBlockTopCoinCollectionIntent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyRuntimeEffectAnimationDetailSummary {
    pub last_coin_block: Option<LegacyRuntimeCoinBlockAnimationDetail>,
    pub last_coin_block_prune: Option<LegacyRuntimeCoinBlockAnimationDetail>,
    pub coin_block_queue_len_after_prune: usize,
    pub last_block_debris: Option<LegacyRuntimeBlockDebrisAnimationDetail>,
    pub last_block_debris_prune: Option<LegacyRuntimeBlockDebrisAnimationDetail>,
    pub block_debris_queue_len_after_prune: usize,
    pub last_scrolling_score: Option<LegacyRuntimeScrollingScoreAnimationDetail>,
    pub last_scrolling_score_prune: Option<LegacyRuntimeScrollingScoreAnimationDetail>,
    pub explicit_fireball_collision_scrolling_score_progress_count: usize,
    pub explicit_fireball_collision_scrolling_score_prune_count: usize,
    pub last_explicit_fireball_collision_scrolling_score:
        Option<LegacyRuntimeScrollingScoreAnimationDetail>,
    pub last_explicit_fireball_collision_scrolling_score_prune:
        Option<LegacyRuntimeScrollingScoreAnimationDetail>,
    pub scrolling_score_queue_len_after_prune: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeCoinBlockAnimationDetail {
    pub frame_index: usize,
    pub report: LegacyRuntimeCoinBlockAnimationUpdateReport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeBlockDebrisAnimationDetail {
    pub frame_index: usize,
    pub report: LegacyRuntimeBlockDebrisAnimationUpdateReport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimeScrollingScoreAnimationDetail {
    pub frame_index: usize,
    pub report: LegacyRuntimeScrollingScoreAnimationUpdateReport,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LegacyRuntimePortalabilitySummary {
    pub queried_tile_count: usize,
    pub portalable_tile_count: usize,
    pub solid_portalable_tile_count: usize,
    pub solid_non_portalable_tile_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyRuntimeHarnessFrame {
    pub index: usize,
    pub should_update: bool,
    pub update_dt: f32,
    pub movement_input: PlayerMovementInput,
    pub player: LegacyRuntimePlayer,
    pub player_render_preview: LegacyRuntimePlayerRenderIntentPreview,
    pub player_render_preview_count: usize,
    pub fireball_launch_intent: Option<LegacyRuntimeFireballLaunchIntent>,
    pub fireball_projectile_progress_count: usize,
    pub fireball_projectile_prune_count: usize,
    pub fireball_render_preview_count: usize,
    pub fireball_render_preview_suppressed_count: usize,
    pub fireball_map_target_probe_count: usize,
    pub fireball_collision_probe_count: usize,
    pub fireball_collision_release_summary_count: usize,
    pub projected_fireball_projectile_collision_snapshot_count: usize,
    pub projected_fireball_projectile_collision_state:
        LegacyRuntimeProjectedFireballProjectileCollisionState,
    pub fireball_enemy_hit_intent_count: usize,
    pub projected_fireball_enemy_hit_snapshot_count: usize,
    pub projected_fireball_enemy_hit_state: LegacyRuntimeProjectedFireballEnemyHitState,
    pub projected_fireball_count_snapshot_count: usize,
    pub projected_fireball_count_state: LegacyRuntimeProjectedFireballCountState,
    pub audio_command_count: usize,
    pub audio_commands: Vec<LegacyAudioCommand>,
    pub coin_pickup_count: usize,
    pub coin_pickups: Vec<LegacyRuntimePlayerCoinPickup>,
    pub coin_counter_intent_count: usize,
    pub score_counter_intent_count: usize,
    pub scrolling_score_intent_count: usize,
    pub many_coins_timer_progress_count: usize,
    pub many_coins_timer_start_count: usize,
    pub projected_many_coins_timer_count: usize,
    pub tile_change_projection_count: usize,
    pub projected_tile_change_snapshot_count: usize,
    pub breakable_block_cleanup_projection_count: usize,
    pub coin_block_animation_progress_count: usize,
    pub coin_block_animation_prune_count: usize,
    pub block_debris_animation_progress_count: usize,
    pub block_debris_animation_prune_count: usize,
    pub scrolling_score_animation_progress_count: usize,
    pub scrolling_score_animation_prune_count: usize,
    pub explicit_fireball_collision_scrolling_score_animation_progress_count: usize,
    pub explicit_fireball_collision_scrolling_score_animation_prune_count: usize,
    pub collisions: LegacyRuntimePlayerCollisionReport,
    pub tile_change_projections: Vec<LegacyRuntimeTileChangeProjection>,
    pub projected_tile_change_snapshot: Vec<LegacyRuntimeTileChangeProjection>,
    pub portal_target_probe: Option<LegacyRuntimePortalTargetProbe>,
    pub portal_outcome_intent: Option<LegacyRuntimePortalOutcomeIntent>,
    pub portal_reservation_projections: Vec<LegacyRuntimePortalReservationProjection>,
    pub portal_replacement_summaries: Vec<LegacyRuntimePortalReplacementSummary>,
    pub projected_portal_state_snapshot_count: usize,
    pub projected_portal_state: LegacyRuntimeProjectedPortalState,
    pub portal_pair_readiness_summary: Option<LegacyRuntimePortalPairReadinessSummary>,
    pub portal_transit_candidate_probe: Option<LegacyRuntimePortalTransitCandidateProbe>,
    pub portalcoords_preview: Option<LegacyRuntimePortalCoordsPreviewReport>,
    pub portal_transit_outcome_summary: Option<LegacyRuntimePortalTransitOutcomeSummary>,
    pub portal_transit_audio_intent: Option<LegacyRuntimePortalTransitAudioIntent>,
    pub portal_transit_projected_player_snapshot: Option<LegacyRuntimeProjectedPlayerStateSnapshot>,
    pub projected_player_state_snapshot_count: usize,
    pub projected_player_state: LegacyRuntimeProjectedPlayerState,
    pub background_color: Option<LegacyColor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyRuntimePlayerSpawnSource {
    LegacyPlayerSpawnEntity,
    FixedSeedFallback,
    ConfiguredSeed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyRuntimePlayerSpawn {
    pub source: LegacyRuntimePlayerSpawnSource,
    pub coord: Option<LegacyMapTileCoord>,
    pub player: LegacyRuntimePlayer,
}

pub fn run_legacy_runtime_harness(
    source: &impl LegacyAssetSource,
    config: LegacyRuntimeHarnessConfig,
) -> Result<LegacyRuntimeHarnessReport, LegacyRuntimeHarnessError> {
    let controls = LegacyPlayerControls::new(
        LegacyControlBinding::keyboard("left"),
        LegacyControlBinding::keyboard("right"),
        LegacyControlBinding::keyboard("run"),
    );
    let mut shell = LegacyRuntimeShell::load(source, config.selection.clone(), controls)?;
    shell.jump_items = config.jump_items.clone();
    shell.top_enemies = config.top_enemies.clone();
    shell.many_coins_timers = config.many_coins_timers.clone();
    shell.fireball_projectiles = config.initial_fireball_projectiles.clone();
    shell.fireball_enemies = config.fireball_enemies.clone();
    shell.coin_count = config.coin_count;
    shell.score_count = config.score_count;
    shell.life_count_enabled = config.life_count_enabled;
    shell.player_count = config.player_count;
    shell.projected_portal_state = config.initial_projected_portal_state.clone();
    shell.projected_player_state = config.initial_projected_player_state.clone();
    let tile_metadata = LegacyTileMetadataTable::load(
        source,
        LEGACY_RUNTIME_HARNESS_GRAPHICS_PACK,
        &config.selection.mappack,
    )?;
    let settings_name = shell.settings.name.clone();
    let level_width = shell.level.width();
    let custom_tiles = shell.custom_tiles;
    let background_count = shell.background_paths.len();
    let portalability = legacy_runtime_level_portalability_summary(&shell, &tile_metadata);
    let player_spawn = if config.force_initial_player_seed {
        LegacyRuntimePlayerSpawn {
            source: LegacyRuntimePlayerSpawnSource::ConfiguredSeed,
            coord: None,
            player: config.initial_player,
        }
    } else {
        legacy_runtime_player_spawn(&shell.level, config.initial_player)
    };
    let input = input_snapshot(config.input);
    let mut player = player_spawn.player;
    let mut last_frame = None;
    let mut frame_audio_command_count = 0;
    let mut frame_audio_command_detail_summary =
        LegacyRuntimeFrameAudioCommandDetailSummary::default();
    let mut player_render_preview_count = 0;
    let mut player_render_preview_detail_summary =
        LegacyRuntimePlayerRenderPreviewDetailSummary::default();
    let mut fireball_launch_intent_count = 0;
    let mut fireball_launch_detail_summary = LegacyRuntimeFireballLaunchDetailSummary::default();
    let mut fireball_projectile_progress_count = 0;
    let mut fireball_projectile_prune_count = 0;
    let mut fireball_projectile_detail_summary =
        LegacyRuntimeFireballProjectileDetailSummary::default();
    let mut fireball_render_preview_count = 0;
    let mut fireball_render_preview_suppressed_count = 0;
    let mut fireball_render_preview_detail_summary =
        LegacyRuntimeFireballRenderPreviewDetailSummary::default();
    let mut fireball_map_target_probe_count = 0;
    let mut fireball_map_target_detail_summary =
        LegacyRuntimeFireballMapTargetDetailSummary::default();
    let mut fireball_collision_probe_count = 0;
    let mut fireball_collision_release_summary_count = 0;
    let mut fireball_collision_detail_summary =
        LegacyRuntimeFireballCollisionDetailSummary::default();
    let mut projected_fireball_projectile_collision_snapshot_count = 0;
    let mut projected_fireball_projectile_collision_detail_summary =
        LegacyRuntimeProjectedFireballProjectileCollisionDetailSummary::default();
    let mut fireball_enemy_hit_intent_count = 0;
    let mut fireball_enemy_hit_detail_summary =
        LegacyRuntimeFireballEnemyHitDetailSummary::default();
    let mut projected_fireball_enemy_hit_snapshot_count = 0;
    let mut projected_fireball_enemy_hit_detail_summary =
        LegacyRuntimeProjectedFireballEnemyHitDetailSummary::default();
    let mut projected_fireball_count_snapshot_count = 0;
    let mut projected_fireball_count_detail_summary =
        LegacyRuntimeProjectedFireballCountDetailSummary::default();
    let mut coin_pickup_count = 0;
    let mut player_coin_pickup_detail_summary =
        LegacyRuntimePlayerCoinPickupDetailSummary::default();
    let mut coin_counter_intent_count = 0;
    let mut score_counter_intent_count = 0;
    let mut scrolling_score_intent_count = 0;
    let mut life_reward_counter_intent_count = 0;
    let mut coin_counter_detail_summary = LegacyRuntimeCoinCounterDetailSummary::default();
    let mut life_reward_counter_detail_summary =
        LegacyRuntimeLifeRewardCounterDetailSummary::default();
    let mut score_counter_detail_summary = LegacyRuntimeScoreCounterDetailSummary::default();
    let mut scrolling_score_detail_summary = LegacyRuntimeScrollingScoreDetailSummary::default();
    let mut horizontal_collision_count = 0;
    let mut vertical_collision_count = 0;
    let mut ceiling_block_hit_count = 0;
    let mut tile_collision_detail_summary = LegacyRuntimeTileCollisionDetailSummary::default();
    let mut block_hit_portal_guard_summary =
        LegacyRuntimeBlockHitPortalGuardDetailSummary::default();
    let mut block_hit_portal_guard_suppression_count = 0;
    let mut block_hit_projected_portal_guard_suppression_count = 0;
    let mut block_bounce_schedule_count = 0;
    let mut block_bounce_detail_summary = LegacyRuntimeBlockBounceDetailSummary::default();
    let mut coin_block_reward_intent_count = 0;
    let mut top_coin_collection_intent_count = 0;
    let mut coin_block_reward_detail_summary = LegacyRuntimeCoinBlockRewardDetailSummary::default();
    let mut tile_change_projection_count = 0;
    let mut projected_tile_change_snapshot_count = 0;
    let mut tile_change_projection_detail_summary =
        LegacyRuntimeTileChangeProjectionDetailSummary::default();
    let mut breakable_block_cleanup_projection_count = 0;
    let mut breakable_block_cleanup_projection_detail_summary =
        LegacyRuntimeBreakableBlockCleanupProjectionDetailSummary::default();
    let mut coin_block_animation_progress_count = 0;
    let mut coin_block_animation_prune_count = 0;
    let mut block_debris_animation_progress_count = 0;
    let mut block_debris_animation_prune_count = 0;
    let mut scrolling_score_animation_progress_count = 0;
    let mut scrolling_score_animation_prune_count = 0;
    let mut effect_animation_detail_summary = LegacyRuntimeEffectAnimationDetailSummary::default();
    let mut item_jump_request_intent_count = 0;
    let mut enemy_shot_request_intent_count = 0;
    let mut item_jump_request_detail_summary = LegacyRuntimeItemJumpRequestDetailSummary::default();
    let mut enemy_shot_request_detail_summary =
        LegacyRuntimeEnemyShotRequestDetailSummary::default();
    let mut empty_breakable_block_destroy_intent_count = 0;
    let mut block_bounce_progress_count = 0;
    let mut block_bounce_prune_count = 0;
    let mut block_bounce_item_spawn_intent_count = 0;
    let mut many_coins_timer_progress_count = 0;
    let mut many_coins_timer_start_count = 0;
    let mut many_coins_timer_detail_summary = LegacyRuntimeManyCoinsTimerDetailSummary::default();
    let mut contained_reward_reveal_intent_count = 0;
    let mut contained_reward_reveal_detail_summary =
        LegacyRuntimeContainedRewardRevealDetailSummary::default();
    let mut portal_target_source_selection =
        LegacyRuntimePortalTargetSourceSelectionSummary::default();
    let mut portal_target_placement_summary = LegacyRuntimePortalTargetPlacementSummary::default();
    let mut portal_open_outcome_summary = LegacyRuntimePortalOpenOutcomeSummary::default();
    let mut portal_reservation_projection_summary =
        LegacyRuntimePortalReservationProjectionSummary::default();
    let mut portal_replacement_detail_summary =
        LegacyRuntimePortalReplacementDetailSummary::default();
    let mut portal_pair_readiness_detail_summary =
        LegacyRuntimePortalPairReadinessDetailSummary::default();
    let mut portal_transit_candidate_detail_summary =
        LegacyRuntimePortalTransitCandidateDetailSummary::default();
    let mut portalcoords_preview_detail_summary =
        LegacyRuntimePortalCoordsPreviewDetailSummary::default();
    let mut portal_transit_outcome_detail_summary =
        LegacyRuntimePortalTransitOutcomeDetailSummary::default();
    let mut portal_transit_audio_detail_summary =
        LegacyRuntimePortalTransitAudioDetailSummary::default();
    let mut projected_player_state_detail_summary =
        LegacyRuntimeProjectedPlayerStateDetailSummary::default();
    let mut portal_target_probe_count = 0;
    let mut portal_target_projected_player_source_count = 0;
    let mut portal_target_possible_count = 0;
    let mut portal_open_intent_count = 0;
    let mut portal_fizzle_intent_count = 0;
    let mut portal_reservation_projection_count = 0;
    let mut portal_replacement_summary_count = 0;
    let mut projected_portal_state_snapshot_count = 0;
    let mut portal_pair_readiness_summary_count = 0;
    let mut portal_pair_ready_count = 0;
    let mut portal_transit_candidate_probe_count = 0;
    let mut portal_transit_candidate_ready_count = 0;
    let mut portalcoords_preview_count = 0;
    let mut portal_transit_outcome_summary_count = 0;
    let mut portal_transit_audio_intent_count = 0;
    let mut portal_transit_success_preview_count = 0;
    let mut portal_transit_blocked_exit_bounce_preview_count = 0;
    let mut portal_transit_projected_player_snapshot_count = 0;
    let mut projected_player_state_snapshot_count = 0;

    for index in 0..config.frames {
        let mut request = LegacyRuntimeFrameRequest::new(
            config.raw_dt,
            config.joystick_deadzone,
            config.render,
            None,
        );
        if config.input.portal_1 || config.input.portal_2 {
            request = request.with_portal_aim(
                LegacyRuntimePortalAimSnapshot::new(config.input.pointing_angle)
                    .with_portal_1_down(config.input.portal_1)
                    .with_portal_2_down(config.input.portal_2),
            );
        }
        if config.input.fire {
            request = request.with_fireball_launch(
                LegacyRuntimeFireballLaunchSnapshot::new(config.input.pointing_angle)
                    .with_flower_power(config.input.fire_flower_power)
                    .with_ducking(config.input.fire_ducking)
                    .with_active_fireball_count(config.input.active_fireball_count),
            );
        }
        if let Some(probe) = config.fireball_collision_probe {
            request = request.with_fireball_collision_probe(probe);
        }
        let frame = shell.step_player_frame(&mut player, &input, request, &tile_metadata);

        player_render_preview_count += 1;
        player_render_preview_detail_summary.last_preview =
            Some(LegacyRuntimePlayerRenderPreviewDetail {
                frame_index: index,
                preview: frame.player_render_preview,
            });
        frame_audio_command_count += frame.frame.audio_commands.len();
        if !frame.frame.audio_commands.is_empty() {
            frame_audio_command_detail_summary.last_commands =
                Some(LegacyRuntimeFrameAudioCommandDetail {
                    frame_index: index,
                    commands: frame.frame.audio_commands.clone(),
                });
        }
        if let Some(intent) = frame.fireball_launch_intent {
            fireball_launch_intent_count += 1;
            fireball_launch_detail_summary.last_intent = Some(LegacyRuntimeFireballLaunchDetail {
                frame_index: index,
                intent,
            });
            let commands = legacy_play_sound_commands(true, intent.sound);
            frame_audio_command_detail_summary.fireball_launch_command_count += commands.len();
            frame_audio_command_detail_summary.last_fireball_launch =
                Some(LegacyRuntimeFrameAudioFireballLaunchCommandDetail {
                    frame_index: index,
                    intent,
                    commands,
                });
        }
        fireball_projectile_progress_count += frame.fireball_projectile_progress.reports.len();
        fireball_projectile_prune_count += frame
            .fireball_projectile_progress
            .reports
            .iter()
            .filter(|report| report.update.remove)
            .count();
        if let Some(progress) = frame.fireball_projectile_progress.reports.last().copied() {
            fireball_projectile_detail_summary.last_progress =
                Some(LegacyRuntimeFireballProjectileDetail {
                    frame_index: index,
                    progress,
                    queue_len_after_prune: frame.fireball_projectile_progress.queue_len_after_prune,
                });
        }
        fireball_render_preview_count += frame.fireball_render_previews.previews.len();
        fireball_render_preview_suppressed_count += frame
            .fireball_render_previews
            .suppressed_projected_removal_indices
            .len();
        if let Some(preview) = frame.fireball_render_previews.previews.last().copied() {
            fireball_render_preview_detail_summary.last_preview =
                Some(LegacyRuntimeFireballRenderPreviewDetail {
                    frame_index: index,
                    preview,
                });
        }
        if let Some(index) = frame
            .fireball_render_previews
            .suppressed_projected_removal_indices
            .last()
            .copied()
        {
            fireball_render_preview_detail_summary.last_suppressed_projected_removal_index =
                Some(index);
        }
        fireball_map_target_probe_count += frame.fireball_map_target_probes.reports.len();
        if let Some(probe) = frame.fireball_map_target_probes.reports.last().copied() {
            fireball_map_target_detail_summary.last_probe =
                Some(LegacyRuntimeFireballMapTargetDetail {
                    frame_index: index,
                    probe,
                });
        }
        fireball_collision_release_summary_count +=
            frame.fireball_projectile_progress.release_summaries.len();
        if let Some(summary) = frame
            .fireball_projectile_progress
            .release_summaries
            .last()
            .copied()
        {
            fireball_collision_detail_summary.last_release_summary =
                Some(LegacyRuntimeFireballReleaseDetail {
                    frame_index: index,
                    summary,
                });
        }
        fireball_collision_probe_count += frame.fireball_collision_probes.reports.len();
        fireball_collision_release_summary_count +=
            frame.fireball_collision_probes.release_summaries.len();
        if let Some(probe) = frame.fireball_collision_probes.reports.last().copied() {
            let detail = LegacyRuntimeFireballCollisionDetail {
                frame_index: index,
                probe,
            };
            fireball_collision_detail_summary.last_probe = Some(detail);
        }
        for probe in frame.fireball_collision_probes.reports.iter().copied() {
            let detail = LegacyRuntimeFireballCollisionDetail {
                frame_index: index,
                probe,
            };
            match probe.source {
                LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest => {
                    fireball_collision_detail_summary.explicit_probe_count += 1;
                    fireball_collision_detail_summary.last_explicit_probe = Some(detail);
                }
                LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe { .. } => {
                    fireball_collision_detail_summary.map_derived_probe_count += 1;
                    fireball_collision_detail_summary.last_map_derived_probe = Some(detail);
                }
                LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { .. } => {
                    fireball_collision_detail_summary.enemy_overlap_probe_count += 1;
                    fireball_collision_detail_summary.last_enemy_overlap_probe = Some(detail);
                }
            }
        }
        if let Some(summary) = frame
            .fireball_collision_probes
            .release_summaries
            .last()
            .copied()
        {
            fireball_collision_detail_summary.last_release_summary =
                Some(LegacyRuntimeFireballReleaseDetail {
                    frame_index: index,
                    summary,
                });
        }
        for snapshot in frame
            .projected_fireball_projectile_collision_snapshots
            .iter()
            .copied()
        {
            projected_fireball_projectile_collision_detail_summary.last_snapshot =
                Some(LegacyRuntimeProjectedFireballProjectileCollisionDetail {
                    frame_index: index,
                    snapshot,
                });
        }
        projected_fireball_projectile_collision_snapshot_count =
            frame.projected_fireball_projectile_collision_state.len();
        fireball_enemy_hit_intent_count += frame.fireball_enemy_hit_intents.len();
        for intent in frame.fireball_enemy_hit_intents.iter().copied() {
            fireball_enemy_hit_detail_summary.last_intent =
                Some(LegacyRuntimeFireballEnemyHitDetail {
                    frame_index: index,
                    intent,
                });
        }
        for snapshot in frame.projected_fireball_enemy_hit_snapshots.iter().copied() {
            projected_fireball_enemy_hit_detail_summary.last_snapshot =
                Some(LegacyRuntimeProjectedFireballEnemyHitDetail {
                    frame_index: index,
                    snapshot,
                });
        }
        projected_fireball_enemy_hit_snapshot_count =
            frame.projected_fireball_enemy_hit_state.len();
        for snapshot in frame.fireball_count_projections.iter().copied() {
            projected_fireball_count_detail_summary.last_snapshot =
                Some(LegacyRuntimeProjectedFireballCountDetail {
                    frame_index: index,
                    snapshot,
                });
        }
        projected_fireball_count_snapshot_count =
            frame.projected_fireball_count_state.snapshot_count();
        for probe in frame.fireball_collision_probes.reports.iter().copied() {
            if probe.outcome.play_block_hit_sound {
                let commands = legacy_play_sound_commands(true, LegacySoundEffect::BlockHit);
                frame_audio_command_detail_summary.fireball_collision_command_count +=
                    commands.len();
                frame_audio_command_detail_summary.last_fireball_collision =
                    Some(LegacyRuntimeFrameAudioFireballCollisionCommandDetail {
                        frame_index: index,
                        probe,
                        commands,
                    });
            }
        }
        for block_hit in frame
            .collisions
            .block_hits
            .iter()
            .copied()
            .filter(|block_hit| block_hit.play_hit_sound)
        {
            let commands = legacy_play_sound_commands(true, LegacySoundEffect::BlockHit);
            frame_audio_command_detail_summary.block_hit_command_count += commands.len();
            frame_audio_command_detail_summary.last_block_hit =
                Some(LegacyRuntimeFrameAudioBlockHitCommandDetail {
                    frame_index: index,
                    block_hit,
                    commands,
                });
        }
        for intent in frame.collisions.contained_reward_reveals.iter().copied() {
            let commands = legacy_play_sound_commands(
                true,
                legacy_runtime_reveal_sound_effect(intent.outcome.sound),
            );
            frame_audio_command_detail_summary.reward_reveal_command_count += commands.len();
            frame_audio_command_detail_summary.last_reward_reveal =
                Some(LegacyRuntimeFrameAudioRewardRevealCommandDetail {
                    frame_index: index,
                    intent,
                    commands,
                });
        }
        for intent in frame.collisions.coin_block_rewards.iter().copied() {
            if intent.outcome.play_coin_sound {
                let commands = legacy_play_sound_commands(true, LegacySoundEffect::Coin);
                frame_audio_command_detail_summary.coin_block_reward_command_count +=
                    commands.len();
                frame_audio_command_detail_summary.last_coin_block_reward =
                    Some(LegacyRuntimeFrameAudioCoinBlockRewardCommandDetail {
                        frame_index: index,
                        intent,
                        commands,
                    });
            }
        }
        for intent in frame.collisions.top_coin_collections.iter().copied() {
            if intent.outcome.play_coin_sound {
                let commands = legacy_play_sound_commands(true, LegacySoundEffect::Coin);
                frame_audio_command_detail_summary.top_coin_collection_command_count +=
                    commands.len();
                frame_audio_command_detail_summary.last_top_coin_collection =
                    Some(LegacyRuntimeFrameAudioTopCoinCollectionCommandDetail {
                        frame_index: index,
                        intent,
                        commands,
                    });
            }
        }
        for intent in &frame.collisions.empty_breakable_block_destroys {
            if matches!(
                &intent.outcome,
                LegacyBreakableBlockOutcome::Broken(effects) if effects.play_break_sound
            ) {
                let commands = legacy_play_sound_commands(true, LegacySoundEffect::BlockBreak);
                frame_audio_command_detail_summary.block_break_command_count += commands.len();
                frame_audio_command_detail_summary.last_block_break =
                    Some(LegacyRuntimeFrameAudioBlockBreakCommandDetail {
                        frame_index: index,
                        intent: intent.clone(),
                        commands,
                    });
            }
        }
        coin_pickup_count += frame.coin_pickups.len();
        for pickup in frame.coin_pickups.iter().copied() {
            player_coin_pickup_detail_summary.last_pickup =
                Some(LegacyRuntimePlayerCoinPickupDetail {
                    frame_index: index,
                    pickup,
                });
        }
        coin_counter_intent_count += frame.coin_counter_intents.len();
        for intent in frame.coin_counter_intents.iter().copied() {
            coin_counter_detail_summary.last_intent = Some(LegacyRuntimeCoinCounterDetail {
                frame_index: index,
                intent,
            });
            if intent.life_reward.is_some() {
                life_reward_counter_detail_summary.last_intent =
                    Some(LegacyRuntimeLifeRewardCounterDetail {
                        frame_index: index,
                        intent,
                    });
            }
        }
        score_counter_intent_count += frame.score_counter_intents.len();
        for intent in frame.score_counter_intents.iter().copied() {
            score_counter_detail_summary.last_intent = Some(LegacyRuntimeScoreCounterDetail {
                frame_index: index,
                intent,
            });
            if intent.scrolling_score.is_some() {
                scrolling_score_detail_summary.last_intent =
                    Some(LegacyRuntimeScrollingScoreDetail {
                        frame_index: index,
                        intent,
                    });
            }
        }
        scrolling_score_intent_count += frame
            .score_counter_intents
            .iter()
            .filter(|intent| intent.scrolling_score.is_some())
            .count();
        life_reward_counter_intent_count += frame
            .coin_counter_intents
            .iter()
            .filter(|intent| intent.life_reward.is_some())
            .count();
        if let Some(collision) = frame.collisions.horizontal {
            horizontal_collision_count += 1;
            tile_collision_detail_summary.last_horizontal =
                Some(LegacyRuntimeTileCollisionDetail {
                    frame_index: index,
                    collision,
                });
        }
        if let Some(collision) = frame.collisions.vertical {
            vertical_collision_count += 1;
            tile_collision_detail_summary.last_vertical = Some(LegacyRuntimeTileCollisionDetail {
                frame_index: index,
                collision,
            });
        }
        ceiling_block_hit_count += frame.collisions.block_hits.len();
        for block_hit in frame.collisions.block_hits.iter().copied() {
            tile_collision_detail_summary.last_block_hit =
                Some(LegacyRuntimeCeilingBlockHitDetail {
                    frame_index: index,
                    block_hit,
                });
            if let Some(guard) = block_hit.portal_guard {
                block_hit_portal_guard_suppression_count += 1;
                match guard.source {
                    LegacyRuntimePortalBlockGuardSource::ExplicitReservation => {
                        block_hit_portal_guard_summary.explicit_reservation_count += 1;
                    }
                    LegacyRuntimePortalBlockGuardSource::ProjectedPortalState => {
                        block_hit_projected_portal_guard_suppression_count += 1;
                        block_hit_portal_guard_summary.projected_portal_state_count += 1;
                    }
                }
                block_hit_portal_guard_summary.last_guard =
                    Some(LegacyRuntimeBlockHitPortalGuardDetail {
                        frame_index: index,
                        coord: block_hit.coord,
                        tile_id: block_hit.tile_id,
                        breakable: block_hit.breakable,
                        coin_block: block_hit.coin_block,
                        play_hit_sound: block_hit.play_hit_sound,
                        guard,
                    });
            }
        }
        block_bounce_schedule_count += frame.collisions.block_bounce_schedules.len();
        for schedule in frame.collisions.block_bounce_schedules.iter().copied() {
            block_bounce_detail_summary.last_schedule =
                Some(LegacyRuntimeBlockBounceScheduleDetail {
                    frame_index: index,
                    schedule,
                });
        }
        coin_block_reward_intent_count += frame.collisions.coin_block_rewards.len();
        for intent in frame.collisions.coin_block_rewards.iter().copied() {
            coin_block_reward_detail_summary.last_reward =
                Some(LegacyRuntimeCoinBlockRewardDetail {
                    frame_index: index,
                    intent,
                });
        }
        top_coin_collection_intent_count += frame.collisions.top_coin_collections.len();
        for intent in frame.collisions.top_coin_collections.iter().copied() {
            coin_block_reward_detail_summary.last_top_coin_collection =
                Some(LegacyRuntimeTopCoinCollectionDetail {
                    frame_index: index,
                    intent,
                });
        }
        tile_change_projection_count += frame.tile_change_projections.len();
        for projection in frame.tile_change_projections.iter().copied() {
            tile_change_projection_detail_summary.projection_count += 1;
            tile_change_projection_detail_summary.last_projection =
                Some(LegacyRuntimeTileChangeProjectionDetail {
                    frame_index: index,
                    projection,
                });
        }
        breakable_block_cleanup_projection_count += frame.breakable_block_cleanup_projections.len();
        for projection in frame.breakable_block_cleanup_projections.iter().copied() {
            breakable_block_cleanup_projection_detail_summary.projection_count += 1;
            breakable_block_cleanup_projection_detail_summary.last_projection =
                Some(LegacyRuntimeBreakableBlockCleanupProjectionDetail {
                    frame_index: index,
                    projection,
                });
        }
        coin_block_animation_progress_count += frame.coin_block_animation_progress.reports.len();
        coin_block_animation_prune_count += frame
            .coin_block_animation_progress
            .reports
            .iter()
            .filter(|report| report.remove)
            .count();
        for report in frame.coin_block_animation_progress.reports.iter().copied() {
            let detail = LegacyRuntimeCoinBlockAnimationDetail {
                frame_index: index,
                report,
            };
            effect_animation_detail_summary.last_coin_block = Some(detail);
            if report.remove {
                effect_animation_detail_summary.last_coin_block_prune = Some(detail);
            }
        }
        effect_animation_detail_summary.coin_block_queue_len_after_prune =
            frame.coin_block_animation_progress.queue_len_after_prune;
        block_debris_animation_progress_count +=
            frame.block_debris_animation_progress.reports.len();
        block_debris_animation_prune_count += frame
            .block_debris_animation_progress
            .reports
            .iter()
            .filter(|report| report.remove)
            .count();
        for report in frame
            .block_debris_animation_progress
            .reports
            .iter()
            .copied()
        {
            let detail = LegacyRuntimeBlockDebrisAnimationDetail {
                frame_index: index,
                report,
            };
            effect_animation_detail_summary.last_block_debris = Some(detail);
            if report.remove {
                effect_animation_detail_summary.last_block_debris_prune = Some(detail);
            }
        }
        effect_animation_detail_summary.block_debris_queue_len_after_prune =
            frame.block_debris_animation_progress.queue_len_after_prune;
        scrolling_score_animation_progress_count +=
            frame.scrolling_score_animation_progress.reports.len();
        scrolling_score_animation_prune_count += frame
            .scrolling_score_animation_progress
            .reports
            .iter()
            .filter(|report| report.remove)
            .count();
        for report in frame
            .scrolling_score_animation_progress
            .reports
            .iter()
            .copied()
        {
            let detail = LegacyRuntimeScrollingScoreAnimationDetail {
                frame_index: index,
                report,
            };
            effect_animation_detail_summary.last_scrolling_score = Some(detail);
            if matches!(
                report.source,
                LegacyRuntimeScoreSource::FireballCollisionProbe {
                    source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    ..
                }
            ) {
                effect_animation_detail_summary
                    .explicit_fireball_collision_scrolling_score_progress_count += 1;
                effect_animation_detail_summary.last_explicit_fireball_collision_scrolling_score =
                    Some(detail);
            }
            if report.remove {
                effect_animation_detail_summary.last_scrolling_score_prune = Some(detail);
                if matches!(
                    report.source,
                    LegacyRuntimeScoreSource::FireballCollisionProbe {
                        source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                        ..
                    }
                ) {
                    effect_animation_detail_summary
                        .explicit_fireball_collision_scrolling_score_prune_count += 1;
                    effect_animation_detail_summary
                        .last_explicit_fireball_collision_scrolling_score_prune = Some(detail);
                }
            }
        }
        effect_animation_detail_summary.scrolling_score_queue_len_after_prune = frame
            .scrolling_score_animation_progress
            .queue_len_after_prune;
        item_jump_request_intent_count += frame.collisions.item_jump_requests.len();
        for intent in frame.collisions.item_jump_requests.iter().copied() {
            item_jump_request_detail_summary.last_intent =
                Some(LegacyRuntimeItemJumpRequestDetail {
                    frame_index: index,
                    intent,
                });
        }
        enemy_shot_request_intent_count += frame.collisions.enemy_shot_requests.len();
        for intent in frame.collisions.enemy_shot_requests.iter().copied() {
            enemy_shot_request_detail_summary.last_intent =
                Some(LegacyRuntimeEnemyShotRequestDetail {
                    frame_index: index,
                    intent,
                });
        }
        empty_breakable_block_destroy_intent_count +=
            frame.collisions.empty_breakable_block_destroys.len();
        contained_reward_reveal_intent_count += frame.collisions.contained_reward_reveals.len();
        for intent in frame.collisions.contained_reward_reveals.iter().copied() {
            contained_reward_reveal_detail_summary.last_intent =
                Some(LegacyRuntimeContainedRewardRevealDetail {
                    frame_index: index,
                    intent,
                });
        }
        block_bounce_progress_count += frame.block_bounce_progress.completions.len();
        for completion in frame.block_bounce_progress.completions.iter().copied() {
            block_bounce_detail_summary.last_completion =
                Some(LegacyRuntimeBlockBounceCompletionDetail {
                    frame_index: index,
                    completion,
                });
        }
        if frame.block_bounce_progress.regenerate_sprite_batch {
            block_bounce_prune_count += 1;
            block_bounce_detail_summary.regenerate_sprite_batch_count += 1;
        }
        block_bounce_item_spawn_intent_count +=
            frame.block_bounce_progress.item_spawn_intents.len();
        for intent in frame
            .block_bounce_progress
            .item_spawn_intents
            .iter()
            .copied()
        {
            block_bounce_detail_summary.last_item_spawn =
                Some(LegacyRuntimeBlockBounceItemSpawnDetail {
                    frame_index: index,
                    intent,
                });
        }
        block_bounce_detail_summary.queue_len_after_prune =
            frame.block_bounce_progress.queue_len_after_prune;
        many_coins_timer_progress_count += frame.many_coins_timer_progress.reports.len();
        for report in frame.many_coins_timer_progress.reports.iter().copied() {
            many_coins_timer_detail_summary.last_progress =
                Some(LegacyRuntimeManyCoinsTimerProgressDetail {
                    frame_index: index,
                    report,
                });
        }
        many_coins_timer_start_count += frame.many_coins_timer_progress.starts.len();
        for report in frame.many_coins_timer_progress.starts.iter().copied() {
            many_coins_timer_detail_summary.last_start =
                Some(LegacyRuntimeManyCoinsTimerStartDetail {
                    frame_index: index,
                    report,
                });
        }
        many_coins_timer_detail_summary.projected_timer_count =
            frame.many_coins_timer_progress.projected_timers.len();
        let projected_tile_change_snapshot = frame.projected_tile_change_state.projections.clone();
        projected_tile_change_snapshot_count = projected_tile_change_snapshot.len();
        tile_change_projection_detail_summary.projected_snapshot_count =
            projected_tile_change_snapshot.len();
        tile_change_projection_detail_summary.projected_snapshot =
            projected_tile_change_snapshot.clone();
        tile_change_projection_detail_summary.last_frame_projections =
            frame.tile_change_projections.clone();
        breakable_block_cleanup_projection_detail_summary.last_frame_projections =
            frame.breakable_block_cleanup_projections.clone();
        if let Some(probe) = frame.portal_target_probe {
            portal_target_probe_count += 1;
            match probe.player_source {
                LegacyRuntimePortalTargetPlayerSource::LivePlayer => {
                    portal_target_source_selection.live_player_count += 1;
                }
                LegacyRuntimePortalTargetPlayerSource::ProjectedPortalTransit => {
                    portal_target_projected_player_source_count += 1;
                    portal_target_source_selection.projected_portal_transit_count += 1;
                }
            }
            portal_target_source_selection.last_selection =
                Some(LegacyRuntimePortalTargetSourceSelection {
                    frame_index: index,
                    player_source: probe.player_source,
                    source_x: probe.source_x,
                    source_y: probe.source_y,
                    pointing_angle: probe.pointing_angle,
                    requested_slot: probe.requested_slot,
                });
            if probe.portal_possible() {
                portal_target_possible_count += 1;
                portal_target_placement_summary.possible_count += 1;
            } else {
                portal_target_placement_summary.impossible_count += 1;
            }
            portal_target_placement_summary.last_summary =
                Some(LegacyRuntimePortalTargetPlacementDetail {
                    frame_index: index,
                    requested_slot: probe.requested_slot,
                    trace_hit: probe.trace_hit,
                    placement: probe.placement,
                });
        }
        if let Some(outcome) = frame.portal_outcome_intent {
            match outcome.kind {
                LegacyRuntimePortalOutcomeKind::Open => {
                    portal_open_intent_count += 1;
                    portal_open_outcome_summary.open_count += 1;
                }
                LegacyRuntimePortalOutcomeKind::Fizzle => {
                    portal_fizzle_intent_count += 1;
                    portal_open_outcome_summary.fizzle_count += 1;
                }
            }
            portal_open_outcome_summary.last_summary = Some(LegacyRuntimePortalOpenOutcomeDetail {
                frame_index: index,
                requested_slot: outcome.requested_slot,
                kind: outcome.kind,
                placement: outcome.placement,
                sound: outcome.sound,
            });
            let commands = legacy_play_sound_commands(true, outcome.sound);
            frame_audio_command_detail_summary.portal_outcome_command_count += commands.len();
            frame_audio_command_detail_summary.last_portal_outcome =
                Some(LegacyRuntimeFrameAudioPortalOutcomeCommandDetail {
                    frame_index: index,
                    intent: outcome,
                    commands,
                });
        }
        for projection in frame.portal_reservation_projections.iter().copied() {
            portal_reservation_projection_count += 1;
            portal_reservation_projection_summary.projection_count += 1;
            portal_reservation_projection_summary.last_projection =
                Some(LegacyRuntimePortalReservationProjectionDetail {
                    frame_index: index,
                    projection,
                });
        }
        for summary in frame.portal_replacement_summaries.iter().copied() {
            portal_replacement_summary_count += 1;
            portal_replacement_detail_summary.replacement_count += 1;
            portal_replacement_detail_summary.last_replacement =
                Some(LegacyRuntimePortalReplacementDetail {
                    frame_index: index,
                    summary,
                });
        }
        projected_portal_state_snapshot_count = frame.projected_portal_state.projected_slot_count();
        if let Some(summary) = frame.portal_pair_readiness_summary {
            portal_pair_readiness_summary_count += 1;
            if summary.ready {
                portal_pair_ready_count += 1;
            }
            portal_pair_readiness_detail_summary.last_summary =
                Some(LegacyRuntimePortalPairReadinessDetail {
                    frame_index: index,
                    summary,
                });
        }
        if let Some(probe) = frame.portal_transit_candidate_probe {
            portal_transit_candidate_probe_count += 1;
            if probe.candidate_pairing.is_some() {
                portal_transit_candidate_ready_count += 1;
            }
            portal_transit_candidate_detail_summary.last_probe =
                Some(LegacyRuntimePortalTransitCandidateDetail {
                    frame_index: index,
                    probe,
                });
        }
        if let Some(preview) = frame.portalcoords_preview {
            portalcoords_preview_count += 1;
            portalcoords_preview_detail_summary.last_preview =
                Some(LegacyRuntimePortalCoordsPreviewDetail {
                    frame_index: index,
                    preview,
                });
        }
        if let Some(summary) = frame.portal_transit_outcome_summary {
            portal_transit_outcome_summary_count += 1;
            match summary.kind {
                LegacyRuntimePortalTransitOutcomeKind::TeleportPreview => {
                    portal_transit_success_preview_count += 1;
                }
                LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview => {
                    portal_transit_blocked_exit_bounce_preview_count += 1;
                }
            }
            portal_transit_outcome_detail_summary.last_summary =
                Some(LegacyRuntimePortalTransitOutcomeDetail {
                    frame_index: index,
                    summary,
                });
        }
        if let Some(intent) = frame.portal_transit_audio_intent {
            portal_transit_audio_intent_count += 1;
            portal_transit_audio_detail_summary.last_intent =
                Some(LegacyRuntimePortalTransitAudioDetail {
                    frame_index: index,
                    intent,
                });
            let commands = legacy_play_sound_commands(true, intent.sound);
            frame_audio_command_detail_summary.portal_transit_command_count += commands.len();
            frame_audio_command_detail_summary.last_portal_transit =
                Some(LegacyRuntimeFrameAudioPortalTransitCommandDetail {
                    frame_index: index,
                    intent,
                    commands,
                });
        }
        if let Some(snapshot) = frame.portal_transit_projected_player_snapshot {
            portal_transit_projected_player_snapshot_count += 1;
            projected_player_state_detail_summary.last_snapshot =
                Some(LegacyRuntimeProjectedPlayerStateDetail {
                    frame_index: index,
                    snapshot,
                });
        }
        projected_player_state_snapshot_count = frame.projected_player_state.snapshot_count();

        last_frame = Some(LegacyRuntimeHarnessFrame {
            index,
            should_update: frame.frame.frame_step.should_update,
            update_dt: frame.frame.frame_step.update_dt,
            movement_input: frame.frame.movement_input,
            player: frame.player,
            player_render_preview: frame.player_render_preview,
            player_render_preview_count: 1,
            fireball_launch_intent: frame.fireball_launch_intent,
            fireball_projectile_progress_count: frame.fireball_projectile_progress.reports.len(),
            fireball_projectile_prune_count: frame
                .fireball_projectile_progress
                .reports
                .iter()
                .filter(|report| report.update.remove)
                .count(),
            fireball_render_preview_count: frame.fireball_render_previews.previews.len(),
            fireball_render_preview_suppressed_count: frame
                .fireball_render_previews
                .suppressed_projected_removal_indices
                .len(),
            fireball_map_target_probe_count: frame.fireball_map_target_probes.reports.len(),
            fireball_collision_probe_count: frame.fireball_collision_probes.reports.len(),
            fireball_collision_release_summary_count: frame
                .fireball_projectile_progress
                .release_summaries
                .len()
                + frame.fireball_collision_probes.release_summaries.len(),
            projected_fireball_projectile_collision_snapshot_count,
            projected_fireball_projectile_collision_state: frame
                .projected_fireball_projectile_collision_state
                .clone(),
            fireball_enemy_hit_intent_count: frame.fireball_enemy_hit_intents.len(),
            projected_fireball_enemy_hit_snapshot_count,
            projected_fireball_enemy_hit_state: frame.projected_fireball_enemy_hit_state.clone(),
            projected_fireball_count_snapshot_count,
            projected_fireball_count_state: frame.projected_fireball_count_state,
            audio_command_count: frame.frame.audio_commands.len(),
            audio_commands: frame.frame.audio_commands.clone(),
            coin_pickup_count: frame.coin_pickups.len(),
            coin_pickups: frame.coin_pickups,
            coin_counter_intent_count: frame.coin_counter_intents.len(),
            score_counter_intent_count: frame.score_counter_intents.len(),
            scrolling_score_intent_count: frame
                .score_counter_intents
                .iter()
                .filter(|intent| intent.scrolling_score.is_some())
                .count(),
            many_coins_timer_progress_count: frame.many_coins_timer_progress.reports.len(),
            many_coins_timer_start_count: frame.many_coins_timer_progress.starts.len(),
            projected_many_coins_timer_count: frame
                .many_coins_timer_progress
                .projected_timers
                .len(),
            tile_change_projection_count: frame.tile_change_projections.len(),
            projected_tile_change_snapshot_count: projected_tile_change_snapshot.len(),
            breakable_block_cleanup_projection_count: frame
                .breakable_block_cleanup_projections
                .len(),
            coin_block_animation_progress_count: frame.coin_block_animation_progress.reports.len(),
            coin_block_animation_prune_count: frame
                .coin_block_animation_progress
                .reports
                .iter()
                .filter(|report| report.remove)
                .count(),
            block_debris_animation_progress_count: frame
                .block_debris_animation_progress
                .reports
                .len(),
            block_debris_animation_prune_count: frame
                .block_debris_animation_progress
                .reports
                .iter()
                .filter(|report| report.remove)
                .count(),
            scrolling_score_animation_progress_count: frame
                .scrolling_score_animation_progress
                .reports
                .len(),
            scrolling_score_animation_prune_count: frame
                .scrolling_score_animation_progress
                .reports
                .iter()
                .filter(|report| report.remove)
                .count(),
            explicit_fireball_collision_scrolling_score_animation_progress_count: frame
                .scrolling_score_animation_progress
                .reports
                .iter()
                .filter(|report| {
                    matches!(
                        report.source,
                        LegacyRuntimeScoreSource::FireballCollisionProbe {
                            source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                            ..
                        }
                    )
                })
                .count(),
            explicit_fireball_collision_scrolling_score_animation_prune_count: frame
                .scrolling_score_animation_progress
                .reports
                .iter()
                .filter(|report| {
                    report.remove
                        && matches!(
                            report.source,
                            LegacyRuntimeScoreSource::FireballCollisionProbe {
                                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                                ..
                            }
                        )
                })
                .count(),
            collisions: frame.collisions,
            tile_change_projections: frame.tile_change_projections,
            projected_tile_change_snapshot,
            portal_target_probe: frame.portal_target_probe,
            portal_outcome_intent: frame.portal_outcome_intent,
            portal_reservation_projections: frame.portal_reservation_projections,
            portal_replacement_summaries: frame.portal_replacement_summaries,
            projected_portal_state_snapshot_count,
            projected_portal_state: frame.projected_portal_state,
            portal_pair_readiness_summary: frame.portal_pair_readiness_summary,
            portal_transit_candidate_probe: frame.portal_transit_candidate_probe,
            portalcoords_preview: frame.portalcoords_preview,
            portal_transit_outcome_summary: frame.portal_transit_outcome_summary,
            portal_transit_audio_intent: frame.portal_transit_audio_intent,
            portal_transit_projected_player_snapshot: frame
                .portal_transit_projected_player_snapshot,
            projected_player_state_snapshot_count,
            projected_player_state: frame.projected_player_state,
            background_color: frame.frame.background_color,
        });
    }

    Ok(LegacyRuntimeHarnessReport {
        selection: config.selection,
        settings_name,
        level_width,
        player_spawn,
        frame_count: config.frames,
        custom_tiles,
        background_count,
        portalability,
        final_player: player,
        player_render_preview_count,
        player_render_preview_detail_summary,
        last_frame,
        frame_audio_command_count,
        frame_audio_command_detail_summary,
        fireball_launch_intent_count,
        fireball_launch_detail_summary,
        fireball_projectile_progress_count,
        fireball_projectile_prune_count,
        fireball_projectile_detail_summary,
        fireball_render_preview_count,
        fireball_render_preview_suppressed_count,
        fireball_render_preview_detail_summary,
        fireball_map_target_probe_count,
        fireball_map_target_detail_summary,
        fireball_collision_probe_count,
        fireball_collision_release_summary_count,
        fireball_collision_detail_summary,
        projected_fireball_projectile_collision_snapshot_count,
        projected_fireball_projectile_collision_detail_summary,
        fireball_enemy_hit_intent_count,
        fireball_enemy_hit_detail_summary,
        projected_fireball_enemy_hit_snapshot_count,
        projected_fireball_enemy_hit_detail_summary,
        projected_fireball_count_snapshot_count,
        projected_fireball_count_detail_summary,
        coin_pickup_count,
        player_coin_pickup_detail_summary,
        coin_counter_intent_count,
        score_counter_intent_count,
        scrolling_score_intent_count,
        life_reward_counter_intent_count,
        coin_counter_detail_summary,
        life_reward_counter_detail_summary,
        score_counter_detail_summary,
        scrolling_score_detail_summary,
        horizontal_collision_count,
        vertical_collision_count,
        ceiling_block_hit_count,
        tile_collision_detail_summary,
        block_hit_portal_guard_summary,
        block_hit_portal_guard_suppression_count,
        block_hit_projected_portal_guard_suppression_count,
        block_bounce_schedule_count,
        block_bounce_detail_summary,
        coin_block_reward_intent_count,
        top_coin_collection_intent_count,
        coin_block_reward_detail_summary,
        tile_change_projection_count,
        projected_tile_change_snapshot_count,
        tile_change_projection_detail_summary,
        breakable_block_cleanup_projection_count,
        breakable_block_cleanup_projection_detail_summary,
        coin_block_animation_progress_count,
        coin_block_animation_prune_count,
        block_debris_animation_progress_count,
        block_debris_animation_prune_count,
        scrolling_score_animation_progress_count,
        scrolling_score_animation_prune_count,
        effect_animation_detail_summary,
        item_jump_request_intent_count,
        enemy_shot_request_intent_count,
        item_jump_request_detail_summary,
        enemy_shot_request_detail_summary,
        empty_breakable_block_destroy_intent_count,
        block_bounce_progress_count,
        block_bounce_prune_count,
        block_bounce_item_spawn_intent_count,
        many_coins_timer_progress_count,
        many_coins_timer_start_count,
        many_coins_timer_detail_summary,
        contained_reward_reveal_intent_count,
        contained_reward_reveal_detail_summary,
        portal_target_source_selection,
        portal_target_placement_summary,
        portal_open_outcome_summary,
        portal_reservation_projection_summary,
        portal_replacement_detail_summary,
        portal_pair_readiness_detail_summary,
        portal_transit_candidate_detail_summary,
        portalcoords_preview_detail_summary,
        portal_transit_outcome_detail_summary,
        portal_transit_audio_detail_summary,
        projected_player_state_detail_summary,
        portal_target_probe_count,
        portal_target_projected_player_source_count,
        portal_target_possible_count,
        portal_open_intent_count,
        portal_fizzle_intent_count,
        portal_reservation_projection_count,
        portal_replacement_summary_count,
        projected_portal_state_snapshot_count,
        portal_pair_readiness_summary_count,
        portal_pair_ready_count,
        portal_transit_candidate_probe_count,
        portal_transit_candidate_ready_count,
        portalcoords_preview_count,
        portal_transit_outcome_summary_count,
        portal_transit_audio_intent_count,
        portal_transit_success_preview_count,
        portal_transit_blocked_exit_bounce_preview_count,
        portal_transit_projected_player_snapshot_count,
        projected_player_state_snapshot_count,
        parity_gaps: LEGACY_RUNTIME_HARNESS_PARITY_GAPS,
    })
}

fn legacy_runtime_level_portalability_summary(
    shell: &LegacyRuntimeShell,
    tiles: &LegacyTileMetadataTable,
) -> LegacyRuntimePortalabilitySummary {
    let query = shell.metadata_map_query(tiles);
    let bounds = query.bounds();
    let mut summary = LegacyRuntimePortalabilitySummary::default();

    for y in 1..=bounds.height {
        for x in 1..=bounds.width {
            let coord = LegacyMapTileCoord::new(x, y);
            let Some(tile) = query.tile_metadata_at(coord) else {
                continue;
            };

            summary.queried_tile_count += 1;
            if tile.portalable() {
                summary.portalable_tile_count += 1;
            }
            if tile.solid_portalable() {
                summary.solid_portalable_tile_count += 1;
            }
            if tile.collides() && !tile.portalable() {
                summary.solid_non_portalable_tile_count += 1;
            }
        }
    }

    summary
}

#[must_use]
pub fn legacy_runtime_player_spawn(
    level: &Mari0Level,
    fallback: LegacyRuntimePlayer,
) -> LegacyRuntimePlayerSpawn {
    let mut spawn_coord = None;

    for y in 0..level.height() {
        for x in 0..level.width() {
            let Some(cell) = level.cell(x, y) else {
                continue;
            };
            let Some(entity) = cell.legacy_entity() else {
                continue;
            };

            if entity.kind == LegacyEntityKind::PlayerSpawn {
                spawn_coord = Some(LegacyMapTileCoord::new(
                    usize_to_i32_saturating(x.saturating_add(1)),
                    usize_to_i32_saturating(y.saturating_add(1)),
                ));
            }
        }
    }

    match spawn_coord {
        Some(coord) => LegacyRuntimePlayerSpawn {
            source: LegacyRuntimePlayerSpawnSource::LegacyPlayerSpawnEntity,
            coord: Some(coord),
            player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(
                    coord.x as f32 - 6.0 / 16.0,
                    coord.y as f32 - 1.0,
                    12.0 / 16.0,
                    12.0 / 16.0,
                ),
                PlayerMovementState::default(),
            ),
        },
        None => LegacyRuntimePlayerSpawn {
            source: LegacyRuntimePlayerSpawnSource::FixedSeedFallback,
            coord: None,
            player: fallback,
        },
    }
}

fn usize_to_i32_saturating(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn input_snapshot(input: LegacyRuntimeHarnessInput) -> BufferedLegacyInputSnapshot {
    let mut snapshot = BufferedLegacyInputSnapshot::new();

    if input.left {
        snapshot = snapshot.with_keyboard_key("left");
    }
    if input.right {
        snapshot = snapshot.with_keyboard_key("right");
    }
    if input.run {
        snapshot = snapshot.with_keyboard_key("run");
    }

    snapshot
}

const fn legacy_runtime_reveal_sound_effect(sound: LegacyBlockRevealSound) -> LegacySoundEffect {
    match sound {
        LegacyBlockRevealSound::MushroomAppear => LegacySoundEffect::MushroomAppear,
        LegacyBlockRevealSound::Vine => LegacySoundEffect::Vine,
    }
}

#[derive(Debug)]
pub enum LegacyRuntimeHarnessError {
    RuntimeLoad(LegacyRuntimeLoadError),
    TileMetadata(LegacyTileMetadataLoadError),
}

impl fmt::Display for LegacyRuntimeHarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeLoad(source) => write!(formatter, "{source}"),
            Self::TileMetadata(source) => write!(formatter, "{source}"),
        }
    }
}

impl Error for LegacyRuntimeHarnessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RuntimeLoad(source) => Some(source),
            Self::TileMetadata(source) => Some(source),
        }
    }
}

impl From<LegacyRuntimeLoadError> for LegacyRuntimeHarnessError {
    fn from(source: LegacyRuntimeLoadError) -> Self {
        Self::RuntimeLoad(source)
    }
}

impl From<LegacyTileMetadataLoadError> for LegacyRuntimeHarnessError {
    fn from(source: LegacyTileMetadataLoadError) -> Self {
        Self::TileMetadata(source)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};

    use super::{
        LegacyRuntimeHarnessConfig, LegacyRuntimeHarnessInput, LegacyRuntimePlayerSpawnSource,
        LegacyRuntimePortalOpenOutcomeDetail, LegacyRuntimePortalOpenOutcomeSummary,
        LegacyRuntimePortalReplacementDetail, LegacyRuntimePortalReplacementDetailSummary,
        LegacyRuntimePortalReservationProjectionDetail,
        LegacyRuntimePortalReservationProjectionSummary, LegacyRuntimePortalTargetPlacementDetail,
        LegacyRuntimePortalTargetPlacementSummary, LegacyRuntimePortalTargetSourceSelection,
        LegacyRuntimePortalTargetSourceSelectionSummary, legacy_runtime_player_spawn,
        run_legacy_runtime_harness,
    };
    use crate::{
        assets::BufferedLegacyAssetSource,
        audio::{LegacyAudioCommand, LegacySoundEffect},
        shell::{
            LegacyRuntimeBlockJumpItemSnapshot, LegacyRuntimeBlockTopEnemySnapshot,
            LegacyRuntimeBreakableBlockCleanupAction, LegacyRuntimeBreakableBlockCleanupProjection,
            LegacyRuntimeBreakableBlockCleanupSource, LegacyRuntimeCoinCounterSource,
            LegacyRuntimeFireballCallback, LegacyRuntimeFireballCollisionAxis,
            LegacyRuntimeFireballCollisionProbeRequest, LegacyRuntimeFireballCollisionProbeSource,
            LegacyRuntimeFireballEnemySnapshot, LegacyRuntimeFireballProjectileReleaseSource,
            LegacyRuntimeFireballRenderFrameKind, LegacyRuntimeFireballRenderQuad,
            LegacyRuntimeFireballRenderSource, LegacyRuntimePlayer, LegacyRuntimePlayerCoinPickup,
            LegacyRuntimePlayerPowerUp, LegacyRuntimePlayerRenderFrame,
            LegacyRuntimePlayerRenderHatSize, LegacyRuntimePlayerRenderQuad,
            LegacyRuntimePlayerRenderTintSource, LegacyRuntimePortalBlockGuardSource,
            LegacyRuntimePortalOutcomeKind, LegacyRuntimePortalPlacement,
            LegacyRuntimePortalReservationProjection, LegacyRuntimePortalSlot,
            LegacyRuntimePortalTargetPlayerSource, LegacyRuntimePortalTransitOutcomeKind,
            LegacyRuntimePortalWallReservation, LegacyRuntimeProjectedFireballCountSource,
            LegacyRuntimeProjectedPlayerState, LegacyRuntimeProjectedPlayerStateSnapshot,
            LegacyRuntimeProjectedPlayerStateSource, LegacyRuntimeProjectedPortalState,
            LegacyRuntimeRenderContext, LegacyRuntimeScoreSource,
            LegacyRuntimeTileChangeProjection, LegacyRuntimeTileChangeSource,
        },
        tiles::LegacyTileMetadata,
        time::LEGACY_MAX_UPDATE_DT,
    };
    use iw2wth_core::{
        Facing, HorizontalDirection, LegacyBlockBounceContentKind, LegacyBlockBounceReplayKind,
        LegacyBlockBounceReplaySpawn, LegacyBlockBounceSpawnKind, LegacyBlockJumpItemKind,
        LegacyBlockPortalReservation, LegacyBlockRevealSound, LegacyEnemyDirection,
        LegacyFireballCollisionOutcome, LegacyFireballCollisionTarget, LegacyFireballFrame,
        LegacyFireballUpdate, LegacyManyCoinsTimerEntry, LegacyMapTileCoord,
        LegacyScrollingScoreLabel, Mari0Level, PlayerAnimationState, PlayerBodyBounds,
        PlayerMovementInput, PlayerMovementState, TileCoord, TileId, content::MARI0_LEVEL_HEIGHT,
    };

    fn level_source(width: usize, properties: &str) -> String {
        let cells = vec!["1"; width * MARI0_LEVEL_HEIGHT];
        format!("{};{properties}", cells.join(","))
    }

    fn source_with_base_tile_atlases() -> BufferedLegacyAssetSource {
        BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}",
        );
    }

    fn atlas_png_from_metadata(metadata: &[LegacyTileMetadata]) -> Vec<u8> {
        let mut image = RgbaImage::new((metadata.len() as u32) * 17, 17);
        for (index, tile) in metadata.iter().enumerate() {
            let metadata_x = index as u32 * 17 + 16;
            set_metadata_flag(&mut image, metadata_x, 0, tile.collision);
            set_metadata_flag(&mut image, metadata_x, 1, tile.invisible);
            set_metadata_flag(&mut image, metadata_x, 2, tile.breakable);
            set_metadata_flag(&mut image, metadata_x, 3, tile.coin_block);
            set_metadata_flag(&mut image, metadata_x, 4, tile.coin);
            set_metadata_flag(&mut image, metadata_x, 5, !tile.portalable);
        }

        let mut bytes = Cursor::new(Vec::new());
        match ImageBuffer::write_to(&image, &mut bytes, ImageFormat::Png) {
            Ok(()) => bytes.into_inner(),
            Err(error) => panic!("failed to encode test PNG: {error}"),
        }
    }

    fn set_metadata_flag(image: &mut RgbaImage, x: u32, y: u32, enabled: bool) {
        if enabled {
            image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }

    #[test]
    fn harness_steps_the_one_level_shell_with_deterministic_input() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 3,
            input: LegacyRuntimeHarnessInput {
                right: true,
                run: true,
                ..LegacyRuntimeHarnessInput::default()
            },
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.settings_name.as_deref(), Some("Super Mario Bros."));
        assert_eq!(report.level_width, 3);
        assert_eq!(
            report.player_spawn.source,
            LegacyRuntimePlayerSpawnSource::FixedSeedFallback,
        );
        assert_eq!(report.frame_count, 3);
        assert_eq!(report.coin_counter_intent_count, 0);
        assert_eq!(report.score_counter_intent_count, 0);
        assert_eq!(report.scrolling_score_intent_count, 0);
        assert_eq!(report.coin_block_animation_progress_count, 0);
        assert_eq!(report.coin_block_animation_prune_count, 0);
        assert_eq!(report.scrolling_score_animation_progress_count, 0);
        assert_eq!(report.scrolling_score_animation_prune_count, 0);
        assert_eq!(report.life_reward_counter_intent_count, 0);
        assert_eq!(report.horizontal_collision_count, 0);
        assert_eq!(report.vertical_collision_count, 0);
        assert!(report.final_player.body.x > 1.0);
        assert!(report.final_player.body.y > 3.0);
        let last_frame = match report.last_frame.as_ref() {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert!(last_frame.should_update);
        assert_eq!(
            last_frame.movement_input,
            PlayerMovementInput::new(false, true, true),
        );
    }

    #[test]
    fn harness_exposes_player_render_preview_detail_without_live_rendering() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(4, "background=1"));
        let movement = PlayerMovementState {
            run_frame: 2,
            swim_frame: 1,
            ducking: true,
            animation_state: PlayerAnimationState::Running,
            animation_direction: HorizontalDirection::Left,
            ..PlayerMovementState::default()
        };
        let initial_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 24.0 / 16.0),
            movement,
        )
        .with_power_up(LegacyRuntimePlayerPowerUp::Fire)
        .with_fire_animation_timer(0.05);
        let config = LegacyRuntimeHarnessConfig {
            frames: 2,
            raw_dt: 0.0,
            force_initial_player_seed: true,
            initial_player,
            render: LegacyRuntimeRenderContext::new(1.25, 2.0),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.player_render_preview_count, 2);
        let detail = report
            .player_render_preview_detail_summary
            .last_preview
            .expect("harness should retain player render preview detail for CLI reporting");
        assert_eq!(detail.frame_index, 1);
        assert_eq!(detail.preview.player_index, 0);
        assert_eq!(detail.preview.power_up, LegacyRuntimePlayerPowerUp::Fire);
        assert_eq!(detail.preview.size, 3);
        assert_eq!(
            detail.preview.render_frame,
            LegacyRuntimePlayerRenderFrame::BigDuck,
        );
        assert_eq!(detail.preview.facing, HorizontalDirection::Left);
        assert_eq!(detail.preview.run_frame, 1);
        assert_eq!(detail.preview.swim_frame, 1);
        assert!(detail.preview.ducking);
        assert!(detail.preview.fire_animation_active);
        assert_eq!(
            detail.preview.quad,
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
            detail.preview.color_layers.map(|layer| (
                layer.draw_order,
                layer.graphic_layer_index,
                layer.image_path
            )),
            [
                (0, 1, "graphics/SMB/player/bigmarioanimations1.png"),
                (1, 2, "graphics/SMB/player/bigmarioanimations2.png"),
                (2, 3, "graphics/SMB/player/bigmarioanimations3.png"),
                (3, 0, "graphics/SMB/player/bigmarioanimations0.png"),
            ],
        );
        assert_eq!(
            detail.preview.color_layers.map(|layer| layer.tint_source),
            [
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::FlowerColor,
                LegacyRuntimePlayerRenderTintSource::White,
            ],
        );
        assert_eq!(detail.preview.hat_draw_count, 1);
        assert_eq!(detail.preview.hat_draws[0].hat_id, 1);
        assert_eq!(
            detail.preview.hat_draws[0].size,
            LegacyRuntimePlayerRenderHatSize::Big,
        );
        assert_eq!(
            detail.preview.hat_draws[0].image_path,
            "graphics/SMB/bighats/standard.png",
        );
        assert_eq!(detail.preview.hat_draws[0].offset_x_px, -5);
        assert_eq!(detail.preview.hat_draws[0].offset_y_px, -4);
        assert_eq!(detail.preview.hat_draws[0].follows_graphic_layer_index, 3);
        assert_eq!(detail.preview.hat_draws[0].precedes_graphic_layer_index, 0);
        assert_eq!(detail.preview.hat_draws[0].origin_x_px, 4);
        assert_eq!(detail.preview.hat_draws[0].origin_y_px, 16);
        assert!(!detail.preview.hat_draws[0].live_rendering_executed);
        assert!(!detail.preview.live_rendering_executed);
        assert!(!detail.preview.live_player_mutated);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain the final frame summary");
        assert_eq!(last_frame.player_render_preview_count, 1);
        assert_eq!(last_frame.player_render_preview, detail.preview);
        assert_eq!(report.final_player, detail.preview.player);
    }

    #[test]
    fn harness_exposes_report_only_fireball_launch_and_audio_detail() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            input: LegacyRuntimeHarnessInput {
                fire: true,
                fire_flower_power: true,
                active_fireball_count: 1,
                pointing_angle: 0.25,
                ..LegacyRuntimeHarnessInput::default()
            },
            initial_player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            force_initial_player_seed: true,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.fireball_launch_intent_count, 1);
        let detail = report
            .fireball_launch_detail_summary
            .last_intent
            .expect("harness should retain the last fireball launch for CLI reporting");
        assert_eq!(detail.frame_index, 0);
        assert_eq!(detail.intent.direction, LegacyEnemyDirection::Left);
        assert_close(detail.intent.source_x, 2.5);
        assert_close(detail.intent.spawn.x, 2.5);
        assert_eq!(detail.intent.fireball_count_before, 1);
        assert_eq!(detail.intent.fireball_count_after, 2);
        assert_eq!(detail.intent.sound, LegacySoundEffect::Fireball);
        assert_eq!(report.projected_fireball_count_snapshot_count, 1);
        let count_detail = report
            .projected_fireball_count_detail_summary
            .last_snapshot
            .expect(
                "harness should retain launch-side projected fireball counts for CLI reporting",
            );
        assert_eq!(count_detail.frame_index, 0);
        assert_eq!(
            count_detail.snapshot.source,
            LegacyRuntimeProjectedFireballCountSource::LaunchIntent,
        );
        assert_eq!(count_detail.snapshot.active_fireball_count_before, 1);
        assert_eq!(count_detail.snapshot.fireball_count_delta, 1);
        assert_eq!(count_detail.snapshot.active_fireball_count_after, 2);
        assert!(!count_detail.snapshot.live_fireball_counter_mutated);
        assert_eq!(
            report
                .last_frame
                .as_ref()
                .and_then(|frame| frame.fireball_launch_intent),
            Some(detail.intent),
        );
        assert_eq!(report.fireball_projectile_progress_count, 1);
        assert_eq!(report.fireball_projectile_prune_count, 0);
        let projectile_detail = report
            .fireball_projectile_detail_summary
            .last_progress
            .expect("harness should retain the last fireball projectile update for CLI reporting");
        assert_eq!(projectile_detail.frame_index, 0);
        assert_eq!(projectile_detail.progress.index, 0);
        assert_eq!(projectile_detail.progress.state_before, detail.intent.spawn);
        assert_eq!(
            projectile_detail.progress.state_after.frame,
            LegacyFireballFrame::FlyingOne
        );
        assert_close(
            projectile_detail.progress.state_after.timer,
            LEGACY_MAX_UPDATE_DT,
        );
        assert_eq!(
            projectile_detail.progress.update,
            LegacyFireballUpdate {
                remove: false,
                released_thrower: false,
            },
        );
        assert_eq!(projectile_detail.queue_len_after_prune, 1);
        assert_eq!(report.fireball_render_preview_count, 1);
        assert_eq!(report.fireball_render_preview_suppressed_count, 0);
        let render_detail = report
            .fireball_render_preview_detail_summary
            .last_preview
            .expect("harness should retain report-only fireball render preview details");
        assert_eq!(render_detail.frame_index, 0);
        assert_eq!(render_detail.preview.projectile_index, 0);
        assert_eq!(
            render_detail.preview.source,
            LegacyRuntimeFireballRenderSource::LiveProjectile,
        );
        assert_eq!(
            render_detail.preview.state,
            projectile_detail.progress.state_after,
        );
        assert_eq!(
            render_detail.preview.frame_kind,
            LegacyRuntimeFireballRenderFrameKind::Flying,
        );
        assert_eq!(
            render_detail.preview.quad,
            LegacyRuntimeFireballRenderQuad {
                x_px: 0,
                y_px: 0,
                width_px: 8,
                height_px: 8,
            },
        );
        assert_eq!(
            render_detail.preview.image_path,
            "graphics/SMB/fireball.png"
        );
        assert!(!render_detail.preview.live_rendering_executed);
        assert!(!render_detail.preview.live_projectile_queue_mutated);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(last_frame.fireball_projectile_progress_count, 1);
        assert_eq!(last_frame.fireball_projectile_prune_count, 0);
        assert_eq!(last_frame.fireball_render_preview_count, 1);
        assert_eq!(last_frame.fireball_render_preview_suppressed_count, 0);
        assert_eq!(last_frame.projected_fireball_count_snapshot_count, 1);
        assert_eq!(
            last_frame
                .projected_fireball_count_state
                .active_fireball_count(),
            Some(2),
        );

        let audio_detail = report
            .frame_audio_command_detail_summary
            .last_fireball_launch
            .expect("harness should retain source-specific fireball audio commands");
        assert_eq!(audio_detail.frame_index, 0);
        assert_eq!(audio_detail.intent, detail.intent);
        assert_eq!(
            audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Fireball),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Fireball),
            ],
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .fireball_launch_command_count,
            2,
        );
    }

    #[test]
    fn harness_exposes_seeded_fireball_projectile_removal_without_collisions() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                -2.0,
                4.0,
                LegacyEnemyDirection::Left,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.fireball_projectile_progress_count, 1);
        assert_eq!(report.fireball_projectile_prune_count, 1);
        let projectile_detail = report
            .fireball_projectile_detail_summary
            .last_progress
            .expect("harness should retain seeded projectile removal for CLI reporting");
        assert_eq!(projectile_detail.frame_index, 0);
        assert_eq!(projectile_detail.progress.index, 0);
        assert_eq!(
            projectile_detail.progress.update,
            LegacyFireballUpdate {
                remove: true,
                released_thrower: true,
            },
        );
        assert!(projectile_detail.progress.state_after.destroy);
        assert_eq!(projectile_detail.queue_len_after_prune, 0);
        assert_eq!(report.fireball_collision_release_summary_count, 1);
        let release_detail = report
            .fireball_collision_detail_summary
            .last_release_summary
            .expect("harness should retain offscreen fireballcallback metadata for CLI reporting");
        assert_eq!(release_detail.frame_index, 0);
        assert_eq!(release_detail.summary.projectile_index, 0);
        assert_eq!(
            release_detail.summary.source,
            LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate,
        );
        assert_eq!(
            release_detail.summary.callback.callback,
            LegacyRuntimeFireballCallback::MarioFireballCallback,
        );
        assert_eq!(release_detail.summary.callback.fireball_count_delta, -1);
        assert!(!release_detail.summary.live_projectile_queue_mutated);
        assert!(!release_detail.summary.live_fireball_counter_mutated);
        assert_eq!(report.projected_fireball_count_snapshot_count, 1);
        let count_detail = report
            .projected_fireball_count_detail_summary
            .last_snapshot
            .expect("harness should retain projected offscreen release counts for CLI reporting");
        assert_eq!(count_detail.frame_index, 0);
        assert_eq!(
            count_detail.snapshot.source,
            LegacyRuntimeProjectedFireballCountSource::ProjectileUpdateReleaseSummary {
                projectile_index: 0,
            },
        );
        assert_eq!(count_detail.snapshot.active_fireball_count_before, 1);
        assert_eq!(count_detail.snapshot.fireball_count_delta, -1);
        assert_eq!(count_detail.snapshot.active_fireball_count_after, 0);
        assert!(!count_detail.snapshot.live_fireball_counter_mutated);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(last_frame.fireball_projectile_progress_count, 1);
        assert_eq!(last_frame.fireball_projectile_prune_count, 1);
        assert_eq!(last_frame.fireball_collision_release_summary_count, 1);
        assert_eq!(
            last_frame
                .projected_fireball_count_state
                .active_fireball_count(),
            Some(0),
        );
        assert!(
            last_frame.collisions.block_hits.is_empty(),
            "seeded fireball removal stays report-only and does not execute live collisions",
        );
    }

    #[test]
    fn harness_exposes_fireball_map_target_probe_detail_without_live_collision() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(4 - 1) * 3 + (3 - 1)] = "2";
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", format!("{};", cells.join(",")));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                1.0,
                3.5,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            initial_player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 8.0, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            force_initial_player_seed: true,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.fireball_map_target_probe_count, 1);
        let detail = report
            .fireball_map_target_detail_summary
            .last_probe
            .expect("harness should retain fireball tile-target metadata for CLI reporting");
        assert_eq!(detail.frame_index, 0);
        assert_eq!(detail.probe.projectile_index, 0);
        assert_eq!(detail.probe.coord, LegacyMapTileCoord::new(3, 4));
        assert_eq!(detail.probe.tile_id, TileId(2));
        assert_eq!(detail.probe.axis, LegacyRuntimeFireballCollisionAxis::Right);
        assert!(detail.probe.collides);
        assert!(!detail.probe.invisible);
        assert!(detail.probe.play_block_hit_sound);
        assert!(!detail.probe.live_projectile_collision_mutated);
        assert_eq!(
            report.fireball_collision_probe_count, 1,
            "map target probes feed cloned tile collision outcome summaries for CLI reporting",
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .explicit_probe_count,
            0,
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .map_derived_probe_count,
            1,
        );
        assert_eq!(report.fireball_collision_release_summary_count, 1);
        let collision_detail = report
            .fireball_collision_detail_summary
            .last_probe
            .expect("harness should retain map-derived fireball tile collision metadata");
        assert_eq!(collision_detail.frame_index, 0);
        assert_eq!(
            collision_detail.probe.axis,
            LegacyRuntimeFireballCollisionAxis::Right,
        );
        assert_eq!(
            collision_detail.probe.target,
            LegacyFireballCollisionTarget::Tile,
        );
        assert_eq!(
            collision_detail.probe.source,
            LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe {
                coord: LegacyMapTileCoord::new(3, 4),
                tile_id: TileId(2),
            },
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .last_map_derived_probe,
            Some(collision_detail),
        );
        assert!(collision_detail.probe.outcome.play_block_hit_sound);
        assert!(collision_detail.probe.outcome.released_thrower);
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .fireball_collision_command_count,
            2,
            "map-derived side tile outcomes preserve block-hit sound commands as report-only audio intent",
        );
        assert_eq!(report.projected_fireball_count_snapshot_count, 1);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(last_frame.fireball_map_target_probe_count, 1);
        assert_eq!(last_frame.fireball_collision_probe_count, 1);
        assert_eq!(last_frame.fireball_collision_release_summary_count, 1);
        assert!(
            last_frame.collisions.block_hits.is_empty(),
            "fireball map target probes do not execute live block-hit/player collision paths",
        );
    }

    #[test]
    fn harness_exposes_report_only_fireball_collision_probe_detail() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            fireball_collision_probe: Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Passive,
                LegacyFireballCollisionTarget::Goomba,
            )),
            fireball_enemies: vec![
                LegacyRuntimeFireballEnemySnapshot::new(
                    LegacyFireballCollisionTarget::Goomba,
                    7,
                    3.5,
                    4.0,
                    1.0,
                    1.0,
                    true,
                ),
                LegacyRuntimeFireballEnemySnapshot::new(
                    LegacyFireballCollisionTarget::Goomba,
                    8,
                    3.5,
                    4.0,
                    1.0,
                    1.0,
                    true,
                ),
            ],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.fireball_collision_probe_count, 1);
        assert_eq!(report.fireball_enemy_hit_intent_count, 1);
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .explicit_probe_count,
            1,
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .map_derived_probe_count,
            0,
        );
        let detail = report
            .fireball_collision_detail_summary
            .last_probe
            .expect("harness should retain the last fireball collision probe for CLI reporting");
        assert_eq!(detail.frame_index, 0);
        assert_eq!(detail.probe.projectile_index, 0);
        assert_eq!(
            detail.probe.axis,
            LegacyRuntimeFireballCollisionAxis::Passive
        );
        assert_eq!(detail.probe.target, LegacyFireballCollisionTarget::Goomba);
        assert_eq!(
            detail.probe.source,
            LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
        );
        assert_eq!(
            report.fireball_collision_detail_summary.last_explicit_probe,
            Some(detail),
        );
        assert_eq!(
            detail.probe.outcome,
            LegacyFireballCollisionOutcome {
                suppress_default: true,
                released_thrower: true,
                play_block_hit_sound: false,
                shoot_target: Some(LegacyEnemyDirection::Right),
                points: Some(100),
            },
        );
        assert!(detail.probe.state_after.destroy_soon);
        assert_eq!(report.fireball_collision_release_summary_count, 1);
        let release_detail = report
            .fireball_collision_detail_summary
            .last_release_summary
            .expect("harness should retain fireballcallback release metadata for CLI reporting");
        assert_eq!(release_detail.frame_index, 0);
        assert_eq!(release_detail.summary.projectile_index, 0);
        assert_eq!(
            release_detail.summary.source,
            LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
        );
        assert_eq!(
            release_detail.summary.callback.callback,
            LegacyRuntimeFireballCallback::MarioFireballCallback,
        );
        assert_eq!(release_detail.summary.callback.fireball_count_delta, -1);
        assert!(!release_detail.summary.live_projectile_queue_mutated);
        assert!(!release_detail.summary.live_fireball_counter_mutated);
        assert_eq!(report.projected_fireball_count_snapshot_count, 1);
        let count_detail = report
            .projected_fireball_count_detail_summary
            .last_snapshot
            .expect("harness should retain the last projected fireball count for CLI reporting");
        assert_eq!(count_detail.frame_index, 0);
        assert_eq!(
            count_detail.snapshot.source,
            LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
                projectile_index: 0,
                collision_source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
        );
        assert_eq!(count_detail.snapshot.active_fireball_count_before, 1);
        assert_eq!(count_detail.snapshot.fireball_count_delta, -1);
        assert_eq!(count_detail.snapshot.active_fireball_count_after, 0);
        assert!(!count_detail.snapshot.live_fireball_counter_mutated);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(last_frame.fireball_collision_probe_count, 1);
        assert_eq!(last_frame.fireball_collision_release_summary_count, 1);
        assert_eq!(last_frame.fireball_enemy_hit_intent_count, 1);
        assert_eq!(report.projected_fireball_enemy_hit_snapshot_count, 1);
        assert_eq!(last_frame.projected_fireball_enemy_hit_snapshot_count, 1);
        assert_eq!(last_frame.projected_fireball_enemy_hit_state.len(), 1);
        assert_eq!(last_frame.projected_fireball_count_snapshot_count, 1);
        assert_eq!(
            last_frame
                .projected_fireball_count_state
                .active_fireball_count(),
            Some(0),
        );
        assert_eq!(last_frame.fireball_projectile_prune_count, 0);
        assert_eq!(report.score_counter_intent_count, 1);
        assert_eq!(report.scrolling_score_intent_count, 1);
        assert_eq!(last_frame.score_counter_intent_count, 1);
        assert_eq!(last_frame.scrolling_score_intent_count, 1);
        let score_detail = report
            .score_counter_detail_summary
            .last_intent
            .expect("harness should retain fireball collision score intent details");
        assert_eq!(score_detail.frame_index, 0);
        assert_eq!(
            score_detail.intent.source,
            LegacyRuntimeScoreSource::FireballCollisionProbe {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
        );
        assert_eq!(score_detail.intent.score_delta, 100);
        let enemy_hit_detail = report
            .fireball_enemy_hit_detail_summary
            .last_intent
            .expect("harness should retain explicit fireball enemy-hit intent details");
        assert_eq!(enemy_hit_detail.frame_index, 0);
        assert_eq!(enemy_hit_detail.intent.enemy.index, 7);
        assert_eq!(
            enemy_hit_detail.intent.shot_direction,
            LegacyEnemyDirection::Right,
        );
        assert_eq!(enemy_hit_detail.intent.score_delta, Some(100));
        assert!(!enemy_hit_detail.intent.live_enemy_mutated);
        let projected_enemy_hit = report
            .projected_fireball_enemy_hit_detail_summary
            .last_snapshot
            .expect("harness should retain projected fireball enemy-hit snapshot details");
        assert_eq!(projected_enemy_hit.frame_index, 0);
        assert_eq!(projected_enemy_hit.snapshot.intent, enemy_hit_detail.intent);
        assert_eq!(projected_enemy_hit.snapshot.enemy.index, 7);
        assert!(!projected_enemy_hit.snapshot.active_after);
        assert!(projected_enemy_hit.snapshot.shot_after);
        assert!(projected_enemy_hit.snapshot.removed_from_future_queries);
        assert!(
            !projected_enemy_hit.snapshot.live_enemy_mutated,
            "projected enemy hit/removal snapshots do not mutate live enemy state",
        );
        let scrolling_score = score_detail
            .intent
            .scrolling_score
            .expect("fireball enemy score probes should preserve Lua floating score metadata");
        assert_eq!(
            scrolling_score.label,
            LegacyScrollingScoreLabel::Points(100)
        );
        assert_close(scrolling_score.x, detail.probe.state_after.x);
        assert_close(scrolling_score.y, detail.probe.state_after.y);
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .fireball_collision_command_count,
            0,
        );
        assert!(
            last_frame.collisions.enemy_shot_requests.is_empty(),
            "collision probes report enemy effects without mutating live enemies",
        );
    }

    #[test]
    fn harness_exposes_report_only_fireball_enemy_overlap_probe_detail() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 2,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            fireball_enemies: vec![
                LegacyRuntimeFireballEnemySnapshot::new(
                    LegacyFireballCollisionTarget::Goomba,
                    7,
                    3.5,
                    4.0,
                    1.0,
                    1.0,
                    true,
                ),
                LegacyRuntimeFireballEnemySnapshot::new(
                    LegacyFireballCollisionTarget::Goomba,
                    8,
                    3.5,
                    4.0,
                    1.0,
                    1.0,
                    true,
                ),
            ],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(
            report.fireball_collision_probe_count, 1,
            "projected projectile collision suppresses the second frame's overlap probe before live collision mutation is migrated",
        );
        assert_eq!(report.fireball_enemy_hit_intent_count, 1);
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .enemy_overlap_probe_count,
            1,
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .explicit_probe_count,
            0,
        );
        assert_eq!(
            report
                .fireball_collision_detail_summary
                .map_derived_probe_count,
            0,
        );
        let detail = report
            .fireball_collision_detail_summary
            .last_enemy_overlap_probe
            .expect("harness should retain automatic enemy-overlap fireball probe details");
        assert_eq!(detail.frame_index, 0);
        assert_eq!(
            detail.probe.source,
            LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { enemy_index: 7 },
        );
        assert_eq!(
            detail.probe.axis,
            LegacyRuntimeFireballCollisionAxis::Passive
        );
        assert_eq!(detail.probe.target, LegacyFireballCollisionTarget::Goomba);
        assert_eq!(
            report.fireball_collision_detail_summary.last_probe,
            Some(detail)
        );
        let enemy_hit_detail = report
            .fireball_enemy_hit_detail_summary
            .last_intent
            .expect("harness should retain automatic enemy-overlap hit intent details");
        assert_eq!(enemy_hit_detail.intent.source, detail.probe.source);
        assert_eq!(enemy_hit_detail.intent.enemy.index, 7);
        assert!(!enemy_hit_detail.intent.live_enemy_mutated);
        assert_eq!(report.projected_fireball_enemy_hit_snapshot_count, 1);
        assert_eq!(
            report.projected_fireball_projectile_collision_snapshot_count,
            1
        );
        let projectile_collision_detail = report
            .projected_fireball_projectile_collision_detail_summary
            .last_snapshot
            .expect("harness should retain projected fireball projectile collision details");
        assert_eq!(projectile_collision_detail.frame_index, 0);
        assert_eq!(projectile_collision_detail.snapshot.projectile_index, 0);
        assert_eq!(
            projectile_collision_detail.snapshot.source,
            detail.probe.source
        );
        assert!(
            projectile_collision_detail
                .snapshot
                .removed_from_future_collision_queries
        );
        assert!(
            !projectile_collision_detail
                .snapshot
                .live_projectile_queue_mutated
        );
        assert_eq!(
            report.fireball_projectile_progress_count, 2,
            "later projectile progress previews the projected collision state",
        );
        let projectile_progress_detail = report
            .fireball_projectile_detail_summary
            .last_progress
            .expect("harness should retain projected fireball progress details");
        assert_eq!(projectile_progress_detail.frame_index, 1);
        assert_eq!(
            projectile_progress_detail.progress.state_before.frame,
            iw2wth_core::LegacyFireballFrame::ExplosionOne,
        );
        assert_eq!(
            projectile_progress_detail.progress.state_after.frame,
            iw2wth_core::LegacyFireballFrame::ExplosionOne,
        );
        assert!(!projectile_progress_detail.progress.update.remove);
        assert_eq!(report.fireball_render_preview_count, 2);
        assert_eq!(report.fireball_render_preview_suppressed_count, 0);
        let render_detail = report
            .fireball_render_preview_detail_summary
            .last_preview
            .expect("harness should retain projected fireball render preview details");
        assert_eq!(render_detail.frame_index, 1);
        assert_eq!(render_detail.preview.projectile_index, 0);
        assert_eq!(
            render_detail.preview.source,
            LegacyRuntimeFireballRenderSource::ProjectedProjectileCollision,
        );
        assert_eq!(
            render_detail.preview.frame_kind,
            LegacyRuntimeFireballRenderFrameKind::Explosion,
        );
        assert_eq!(
            render_detail.preview.frame,
            LegacyFireballFrame::ExplosionOne
        );
        assert_eq!(
            render_detail.preview.quad,
            LegacyRuntimeFireballRenderQuad {
                x_px: 32,
                y_px: 0,
                width_px: 16,
                height_px: 16,
            },
        );
        assert!(!render_detail.preview.live_rendering_executed);
        assert!(!render_detail.preview.live_projectile_queue_mutated);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(
            last_frame.fireball_collision_probe_count, 0,
            "the final frame has no repeated overlap after projected projectile collision",
        );
        assert_eq!(last_frame.projected_fireball_enemy_hit_state.len(), 1);
        assert_eq!(
            last_frame
                .projected_fireball_projectile_collision_state
                .len(),
            1
        );
        assert!(
            last_frame.collisions.enemy_shot_requests.is_empty(),
            "automatic fireball enemy overlap probes stay out of live enemy mutation paths",
        );
    }

    #[test]
    fn harness_exposes_fireball_collision_scrolling_score_animation_provenance() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 2,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            fireball_collision_probe: Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Passive,
                LegacyFireballCollisionTarget::Goomba,
            )),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert!(report.scrolling_score_intent_count > 0);
        assert_eq!(
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_progress_count,
            1,
            "the first explicit enemy probe queues a report-only scrolling-score animation that progresses on the next frame",
        );
        assert_eq!(
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_prune_count,
            0,
        );
        let detail = report
            .effect_animation_detail_summary
            .last_explicit_fireball_collision_scrolling_score
            .expect("harness should retain fireball collision scrolling-score provenance");
        assert_eq!(detail.frame_index, 1);
        assert_eq!(
            detail.report.source,
            LegacyRuntimeScoreSource::FireballCollisionProbe {
                projectile_index: 0,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            },
        );
        assert_eq!(
            detail.report.state.label,
            LegacyScrollingScoreLabel::Points(100),
        );
        assert!(!detail.report.remove);
        assert!(
            report
                .effect_animation_detail_summary
                .last_explicit_fireball_collision_scrolling_score_prune
                .is_none(),
        );
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should retain final frame counts");
        assert_eq!(last_frame.scrolling_score_animation_progress_count, 1);
        assert_eq!(last_frame.scrolling_score_animation_prune_count, 0);
        assert_eq!(
            last_frame.explicit_fireball_collision_scrolling_score_animation_progress_count,
            1,
        );
        assert_eq!(
            last_frame.explicit_fireball_collision_scrolling_score_animation_prune_count,
            0,
        );
        assert!(
            last_frame.collisions.enemy_shot_requests.is_empty(),
            "fireball collision scrolling-score reports do not execute live enemy mutation",
        );
    }

    #[test]
    fn harness_threads_fireball_collision_block_hit_audio_detail_without_live_audio() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_fireball_projectiles: vec![iw2wth_core::LegacyFireballState::spawn(
                3.0,
                4.0,
                LegacyEnemyDirection::Right,
                iw2wth_core::LegacyFireballConstants::default(),
            )],
            fireball_collision_probe: Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                0,
                LegacyRuntimeFireballCollisionAxis::Left,
                LegacyFireballCollisionTarget::Tile,
            )),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.fireball_collision_probe_count, 1);
        assert_eq!(report.frame_audio_command_count, 2);
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .fireball_collision_command_count,
            2,
        );
        let audio_detail = report
            .frame_audio_command_detail_summary
            .last_fireball_collision
            .expect("harness should retain source-specific fireball collision audio commands");
        assert_eq!(audio_detail.frame_index, 0);
        assert_eq!(
            audio_detail.probe.target,
            LegacyFireballCollisionTarget::Tile
        );
        assert_eq!(
            audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
            ],
        );
        assert_eq!(report.score_counter_intent_count, 0);
        assert!(
            report
                .last_frame
                .as_ref()
                .expect("harness should retain final frame counts")
                .collisions
                .block_hits
                .is_empty(),
            "fireball collision audio stays report-only and does not synthesize block hits",
        );
    }

    #[test]
    fn harness_counts_coin_counter_and_life_reward_intents() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(4 - 1) * 3 + (2 - 1)] = "3";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        ..LegacyTileMetadata::empty()
                    },
                    LegacyTileMetadata {
                        coin: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            coin_count: 99,
            life_count_enabled: false,
            player_count: 2,
            initial_player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.coin_pickup_count, 1);
        assert_eq!(report.frame_audio_command_count, 2);
        assert_eq!(report.coin_counter_intent_count, 1);
        assert_eq!(report.score_counter_intent_count, 1);
        assert_eq!(report.scrolling_score_intent_count, 0);
        assert_eq!(report.life_reward_counter_intent_count, 1);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.coin_pickup_count, 1);
        assert_eq!(last_frame.audio_command_count, 2);
        assert_eq!(
            last_frame.audio_commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        assert_eq!(
            last_frame.coin_pickups,
            vec![LegacyRuntimePlayerCoinPickup {
                coord: LegacyMapTileCoord::new(2, 4),
                tile_id: TileId(3),
                clear_tile_id: TileId(1),
                score_delta: 200,
                sound: LegacySoundEffect::Coin,
            }],
        );
        assert_eq!(last_frame.coin_counter_intent_count, 1);
        assert_eq!(last_frame.score_counter_intent_count, 1);
        assert_eq!(last_frame.scrolling_score_intent_count, 0);
        let audio_detail = report
            .frame_audio_command_detail_summary
            .last_commands
            .expect("harness should retain the last frame audio commands for CLI reporting");
        assert_eq!(audio_detail.frame_index, 0);
        assert_eq!(
            audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        let pickup_detail = report
            .player_coin_pickup_detail_summary
            .last_pickup
            .expect("harness should retain the last player coin pickup for CLI reporting");
        assert_eq!(pickup_detail.frame_index, 0);
        assert_eq!(
            pickup_detail.pickup,
            LegacyRuntimePlayerCoinPickup {
                coord: LegacyMapTileCoord::new(2, 4),
                tile_id: TileId(3),
                clear_tile_id: TileId(1),
                score_delta: 200,
                sound: LegacySoundEffect::Coin,
            },
        );
        let coin_detail = report
            .coin_counter_detail_summary
            .last_intent
            .expect("harness should retain the last coin counter intent for CLI reporting");
        assert_eq!(coin_detail.frame_index, 0);
        assert_eq!(
            coin_detail.intent.source,
            LegacyRuntimeCoinCounterSource::PlayerCoinPickup {
                coord: LegacyMapTileCoord::new(2, 4),
            },
        );
        assert_eq!(coin_detail.intent.coin_count_before, 99);
        assert_eq!(coin_detail.intent.coin_count_after, 0);
        assert_eq!(coin_detail.intent.score_delta, 200);
        let life_detail = report
            .life_reward_counter_detail_summary
            .last_intent
            .expect("harness should retain the last life reward intent for CLI reporting");
        assert_eq!(life_detail.frame_index, 0);
        let life_reward = life_detail
            .intent
            .life_reward
            .expect("life reward summary should retain the reward payload");
        assert_eq!(life_reward.grant_lives_to_players, 0);
        assert!(!life_reward.respawn_players);
        assert!(life_reward.play_sound);
        let score_detail = report
            .score_counter_detail_summary
            .last_intent
            .expect("harness should retain the last score counter intent for CLI reporting");
        assert_eq!(score_detail.frame_index, 0);
        assert_eq!(
            score_detail.intent.source,
            LegacyRuntimeScoreSource::PlayerCoinPickup {
                coord: LegacyMapTileCoord::new(2, 4),
            },
        );
        assert_eq!(score_detail.intent.score_count_before, 0);
        assert_eq!(score_detail.intent.score_delta, 200);
        assert_eq!(score_detail.intent.score_count_after, 200);
        assert_eq!(report.scrolling_score_detail_summary.last_intent, None);
    }

    #[test]
    fn harness_tile_collisions_use_legacy_atlas_metadata() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(5 - 1) * 3 + (2 - 1)] = "2";
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: 20.0,
                    falling: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.vertical_collision_count, 1);
        assert_eq!(report.horizontal_collision_count, 0);
        let vertical_detail = report
            .tile_collision_detail_summary
            .last_vertical
            .expect("harness should retain the last vertical collision for CLI reporting");
        assert_eq!(vertical_detail.frame_index, 0);
        assert_eq!(
            vertical_detail.collision.coord,
            LegacyMapTileCoord::new(2, 5)
        );
        assert_eq!(vertical_detail.collision.tile_id, TileId(2));
        assert_eq!(report.tile_collision_detail_summary.last_horizontal, None);
        assert!(report.final_player.body.y < 4.0);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(
            last_frame
                .collisions
                .vertical
                .map(|collision| collision.coord),
            Some(LegacyMapTileCoord::new(2, 5)),
        );
    }

    #[test]
    fn harness_retains_last_horizontal_tile_collision_detail_for_cli_reporting() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(4 - 1) * 3 + (2 - 1)] = "2";
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(0.2, 3.05, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_x: 80.0,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.horizontal_collision_count, 1);
        let horizontal_detail = report
            .tile_collision_detail_summary
            .last_horizontal
            .expect("harness should retain the last horizontal collision for CLI reporting");
        assert_eq!(horizontal_detail.frame_index, 0);
        assert_eq!(
            horizontal_detail.collision.coord,
            LegacyMapTileCoord::new(2, 4)
        );
        assert_eq!(horizontal_detail.collision.tile_id, TileId(2));
    }

    #[test]
    fn harness_reports_current_level_portalability_from_adapter_metadata() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[0] = "2";
        cells[1] = "3";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
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
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 0,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.portalability.queried_tile_count, 45);
        assert_eq!(report.portalability.portalable_tile_count, 44);
        assert_eq!(report.portalability.solid_portalable_tile_count, 1);
        assert_eq!(report.portalability.solid_non_portalable_tile_count, 1);
        assert_eq!(report.coin_pickup_count, 0);
        assert_eq!(report.horizontal_collision_count, 0);
    }

    #[test]
    fn harness_counts_report_only_portal_target_probes_from_input_snapshot() {
        let mut cells = vec!["1"; 6 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 6 + (4 - 1)] = "2";
        cells[(4 - 1) * 6 + (4 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        portalable: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            raw_dt: 0.0,
            input: LegacyRuntimeHarnessInput {
                portal_1: true,
                pointing_angle: -1.5,
                ..LegacyRuntimeHarnessInput::default()
            },
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(2.0, 3.125, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.portal_target_probe_count, 1);
        assert_eq!(
            report.portal_target_source_selection,
            LegacyRuntimePortalTargetSourceSelectionSummary {
                live_player_count: 1,
                projected_portal_transit_count: 0,
                last_selection: Some(LegacyRuntimePortalTargetSourceSelection {
                    frame_index: 0,
                    player_source: LegacyRuntimePortalTargetPlayerSource::LivePlayer,
                    source_x: 2.0 + 6.0 / 16.0,
                    source_y: 3.125 + 6.0 / 16.0,
                    pointing_angle: -1.5,
                    requested_slot: Some(LegacyRuntimePortalSlot::Portal1),
                }),
            },
        );
        assert_eq!(report.portal_target_projected_player_source_count, 0);
        assert_eq!(report.portal_target_possible_count, 1);
        assert_eq!(report.portal_open_intent_count, 1);
        assert_eq!(report.portal_fizzle_intent_count, 0);
        assert_eq!(
            report.portal_target_placement_summary,
            LegacyRuntimePortalTargetPlacementSummary {
                possible_count: 1,
                impossible_count: 0,
                last_summary: Some(LegacyRuntimePortalTargetPlacementDetail {
                    frame_index: 0,
                    requested_slot: Some(LegacyRuntimePortalSlot::Portal1),
                    trace_hit: report
                        .last_frame
                        .as_ref()
                        .and_then(|frame| frame.portal_target_probe)
                        .and_then(|probe| probe.trace_hit),
                    placement: Some(LegacyRuntimePortalPlacement {
                        coord: LegacyMapTileCoord::new(4, 4),
                        side: Facing::Left,
                    }),
                }),
            },
        );
        assert_eq!(
            report.portal_open_outcome_summary,
            LegacyRuntimePortalOpenOutcomeSummary {
                open_count: 1,
                fizzle_count: 0,
                last_summary: Some(LegacyRuntimePortalOpenOutcomeDetail {
                    frame_index: 0,
                    requested_slot: LegacyRuntimePortalSlot::Portal1,
                    kind: LegacyRuntimePortalOutcomeKind::Open,
                    placement: Some(LegacyRuntimePortalPlacement {
                        coord: LegacyMapTileCoord::new(4, 4),
                        side: Facing::Left,
                    }),
                    sound: LegacySoundEffect::Portal1Open,
                }),
            },
        );
        assert_eq!(report.portal_reservation_projection_count, 1);
        assert_eq!(report.portal_replacement_summary_count, 1);
        assert_eq!(report.projected_portal_state_snapshot_count, 1);
        assert_eq!(report.portal_pair_readiness_summary_count, 1);
        assert_eq!(report.portal_pair_ready_count, 0);
        let last_frame = report
            .last_frame
            .expect("harness should record the final frame");
        assert_eq!(last_frame.projected_portal_state_snapshot_count, 1);
        let readiness = last_frame
            .portal_pair_readiness_summary
            .expect("single projected slot should expose a not-ready portal pair summary");
        assert!(!readiness.ready);
        assert!(readiness.portal_1.is_some());
        assert_eq!(readiness.portal_2, None);
        let probe = last_frame
            .portal_target_probe
            .expect("last frame should expose the report-only portal probe");
        assert_eq!(
            probe.player_source,
            LegacyRuntimePortalTargetPlayerSource::LivePlayer,
        );
        assert_eq!(
            probe.trace_hit.map(|hit| hit.coord),
            Some(LegacyMapTileCoord::new(4, 4))
        );
        let placement = probe.placement.expect("portal target should be valid");
        assert_eq!(placement.coord, LegacyMapTileCoord::new(4, 4));
        assert_eq!(placement.side, Facing::Left);
        let outcome = last_frame
            .portal_outcome_intent
            .expect("requested valid portal target should expose an open intent");
        assert_eq!(outcome.kind, LegacyRuntimePortalOutcomeKind::Open);
        assert_eq!(outcome.placement, Some(placement));
        assert_eq!(last_frame.portal_reservation_projections.len(), 1);
        let projection = last_frame.portal_reservation_projections[0];
        assert_eq!(projection.requested_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(projection.placement, placement);
        assert_eq!(last_frame.portal_replacement_summaries.len(), 1);
        let replacement = last_frame.portal_replacement_summaries[0];
        assert_eq!(replacement.requested_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(replacement.previous_slot, None);
        assert_eq!(replacement.replacement_slot.placement, placement);
        assert_eq!(replacement.preserved_other_slot, None);
        assert_eq!(
            report.portal_reservation_projection_summary,
            LegacyRuntimePortalReservationProjectionSummary {
                projection_count: 1,
                last_projection: Some(LegacyRuntimePortalReservationProjectionDetail {
                    frame_index: 0,
                    projection,
                }),
            },
        );
        assert_eq!(
            report.portal_replacement_detail_summary,
            LegacyRuntimePortalReplacementDetailSummary {
                replacement_count: 1,
                last_replacement: Some(LegacyRuntimePortalReplacementDetail {
                    frame_index: 0,
                    summary: replacement,
                }),
            },
        );
        assert_eq!(
            projection.tile_reservations,
            [LegacyMapTileCoord::new(4, 4), LegacyMapTileCoord::new(4, 3),],
        );
        assert_eq!(
            projection.wall_reservations,
            [
                LegacyRuntimePortalWallReservation::new(4, 2, 0, 2),
                LegacyRuntimePortalWallReservation::new(3, 2, 1, 0),
                LegacyRuntimePortalWallReservation::new(3, 4, 1, 0),
            ],
        );
        assert_eq!(
            last_frame
                .projected_portal_state
                .block_portal_reservations(),
            vec![LegacyBlockPortalReservation::new(
                TileCoord::new(4, 4),
                Facing::Up,
            )],
        );
        assert_eq!(
            last_frame
                .projected_portal_state
                .portal_1
                .expect("open portal should update the adapter-side projected slot")
                .placement,
            placement,
        );
    }

    #[test]
    fn harness_summarizes_rejected_portal_target_open_outcomes() {
        let mut cells = vec!["1"; 6 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 6 + (4 - 1)] = "3";
        cells[(4 - 1) * 6 + (4 - 1)] = "3";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
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
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            raw_dt: 0.0,
            input: LegacyRuntimeHarnessInput {
                portal_2: true,
                pointing_angle: -1.5,
                ..LegacyRuntimeHarnessInput::default()
            },
            initial_player: LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(2.0, 3.125, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.portal_target_probe_count, 1);
        assert_eq!(report.portal_target_possible_count, 0);
        assert_eq!(report.portal_open_intent_count, 0);
        assert_eq!(report.portal_fizzle_intent_count, 1);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should record the final frame");
        let trace_hit = last_frame
            .portal_target_probe
            .expect("active portal aim should expose a rejected target probe")
            .trace_hit;
        assert_eq!(
            report.portal_target_placement_summary,
            LegacyRuntimePortalTargetPlacementSummary {
                possible_count: 0,
                impossible_count: 1,
                last_summary: Some(LegacyRuntimePortalTargetPlacementDetail {
                    frame_index: 0,
                    requested_slot: Some(LegacyRuntimePortalSlot::Portal2),
                    trace_hit,
                    placement: None,
                }),
            },
        );
        assert_eq!(
            report.portal_open_outcome_summary,
            LegacyRuntimePortalOpenOutcomeSummary {
                open_count: 0,
                fizzle_count: 1,
                last_summary: Some(LegacyRuntimePortalOpenOutcomeDetail {
                    frame_index: 0,
                    requested_slot: LegacyRuntimePortalSlot::Portal2,
                    kind: LegacyRuntimePortalOutcomeKind::Fizzle,
                    placement: None,
                    sound: LegacySoundEffect::PortalFizzle,
                }),
            },
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .portal_outcome_command_count,
            2
        );
        let portal_audio_detail = report
            .frame_audio_command_detail_summary
            .last_portal_outcome
            .expect("harness should retain source-specific portal outcome audio commands");
        assert_eq!(portal_audio_detail.frame_index, 0);
        assert_eq!(
            portal_audio_detail.intent.requested_slot,
            LegacyRuntimePortalSlot::Portal2
        );
        assert_eq!(
            portal_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::PortalFizzle),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::PortalFizzle),
            ],
        );
        assert!(last_frame.portal_reservation_projections.is_empty());
        assert_eq!(report.final_player.body.x, 2.0);
        assert_eq!(report.final_player.body.y, 3.125);
    }

    #[test]
    fn harness_exposes_projected_player_source_for_portal_target_probe() {
        let mut cells = vec!["1"; 6 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 6 + (4 - 1)] = "2";
        cells[(4 - 1) * 6 + (4 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        portalable: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let live_body = PlayerBodyBounds::new(0.25, 8.0, 12.0 / 16.0, 12.0 / 16.0);
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
        let mut initial_projected_player_state = LegacyRuntimeProjectedPlayerState::default();
        initial_projected_player_state.apply_snapshot(snapshot);
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            raw_dt: 0.0,
            input: LegacyRuntimeHarnessInput {
                portal_1: true,
                pointing_angle: -1.5,
                ..LegacyRuntimeHarnessInput::default()
            },
            initial_player: LegacyRuntimePlayer::new(live_body, PlayerMovementState::default()),
            initial_projected_player_state,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.portal_target_probe_count, 1);
        assert_eq!(
            report.portal_target_source_selection,
            LegacyRuntimePortalTargetSourceSelectionSummary {
                live_player_count: 0,
                projected_portal_transit_count: 1,
                last_selection: Some(LegacyRuntimePortalTargetSourceSelection {
                    frame_index: 0,
                    player_source: LegacyRuntimePortalTargetPlayerSource::ProjectedPortalTransit,
                    source_x: projected_body.x + 6.0 / 16.0,
                    source_y: projected_body.y + 6.0 / 16.0,
                    pointing_angle: -1.5,
                    requested_slot: Some(LegacyRuntimePortalSlot::Portal1),
                }),
            },
        );
        assert_eq!(report.portal_target_projected_player_source_count, 1);
        assert_eq!(report.portal_target_possible_count, 1);
        assert_eq!(report.portal_open_intent_count, 1);
        assert_eq!(report.portal_target_placement_summary.possible_count, 1);
        assert_eq!(report.portal_open_outcome_summary.open_count, 1);
        assert_eq!(report.portal_transit_projected_player_snapshot_count, 0);
        assert_eq!(report.projected_player_state_snapshot_count, 1);
        assert_eq!(report.final_player.body, live_body);
        let last_frame = report
            .last_frame
            .expect("harness should record the final frame");
        assert_eq!(last_frame.player.body, live_body);
        assert_eq!(last_frame.projected_player_state_snapshot_count, 1);
        assert_eq!(
            last_frame.projected_player_state.latest_portal_transit(),
            Some(snapshot),
        );
        let probe = last_frame
            .portal_target_probe
            .expect("active portal aim should expose a projected-source target probe");
        assert_eq!(
            probe.player_source,
            LegacyRuntimePortalTargetPlayerSource::ProjectedPortalTransit,
        );
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
    }

    #[test]
    fn harness_counts_report_only_portal_transit_candidate_probes_from_projected_pair() {
        let mut projected_portal_state = LegacyRuntimeProjectedPortalState::default();
        projected_portal_state.apply_projection(LegacyRuntimePortalReservationProjection {
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
        projected_portal_state.apply_projection(LegacyRuntimePortalReservationProjection {
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
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(10, ""));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_projected_portal_state: projected_portal_state,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(3.875, 4.875, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState::default(),
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.portal_pair_readiness_summary_count, 1);
        assert_eq!(report.portal_pair_ready_count, 1);
        assert_eq!(report.portal_transit_candidate_probe_count, 1);
        assert_eq!(report.portal_transit_candidate_ready_count, 1);
        assert_eq!(report.portalcoords_preview_count, 1);
        assert_eq!(report.portal_transit_outcome_summary_count, 1);
        assert_eq!(report.portal_transit_audio_intent_count, 1);
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .portal_transit_command_count,
            2
        );
        assert_eq!(report.portal_transit_success_preview_count, 1);
        assert_eq!(report.portal_transit_blocked_exit_bounce_preview_count, 0);
        assert_eq!(report.portal_transit_projected_player_snapshot_count, 1);
        assert_eq!(report.projected_player_state_snapshot_count, 1);
        let readiness_detail = report
            .portal_pair_readiness_detail_summary
            .last_summary
            .expect("harness should retain the last readiness detail for CLI reporting");
        assert_eq!(readiness_detail.frame_index, 0);
        assert!(readiness_detail.summary.ready);
        assert_eq!(
            readiness_detail
                .summary
                .portal_1
                .expect("portal 1")
                .placement
                .coord,
            LegacyMapTileCoord::new(5, 6),
        );
        assert_eq!(
            readiness_detail
                .summary
                .portal_2
                .expect("portal 2")
                .placement
                .coord,
            LegacyMapTileCoord::new(9, 4),
        );
        let candidate_detail = report
            .portal_transit_candidate_detail_summary
            .last_probe
            .expect("harness should retain the last candidate detail for CLI reporting");
        assert_eq!(candidate_detail.frame_index, 0);
        assert_eq!(
            candidate_detail.probe.center_coord,
            LegacyMapTileCoord::new(5, 6)
        );
        let preview_detail = report
            .portalcoords_preview_detail_summary
            .last_preview
            .expect("harness should retain the last portalcoords detail for CLI reporting");
        assert_eq!(preview_detail.frame_index, 0);
        assert_eq!(
            preview_detail.preview.entry_slot,
            LegacyRuntimePortalSlot::Portal1
        );
        assert_eq!(
            preview_detail.preview.exit_slot,
            LegacyRuntimePortalSlot::Portal2
        );
        assert_eq!(preview_detail.preview.entry_facing, Facing::Up);
        assert_eq!(preview_detail.preview.exit_facing, Facing::Right);
        assert!(!preview_detail.preview.exit_blocked);
        let outcome_detail = report
            .portal_transit_outcome_detail_summary
            .last_summary
            .expect("harness should retain the last outcome detail for CLI reporting");
        assert_eq!(outcome_detail.frame_index, 0);
        assert_eq!(
            outcome_detail.summary.kind,
            LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
        );
        let audio_detail = report
            .portal_transit_audio_detail_summary
            .last_intent
            .expect("harness should retain the last portal-enter audio detail for CLI reporting");
        assert_eq!(audio_detail.frame_index, 0);
        assert_eq!(
            audio_detail.intent.outcome_kind,
            LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
        );
        assert_eq!(audio_detail.intent.sound, LegacySoundEffect::PortalEnter);
        let portal_transit_audio_detail = report
            .frame_audio_command_detail_summary
            .last_portal_transit
            .expect("harness should retain source-specific portal-transit audio commands");
        assert_eq!(portal_transit_audio_detail.frame_index, 0);
        assert_eq!(
            portal_transit_audio_detail.intent.outcome_kind,
            LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
        );
        assert_eq!(
            portal_transit_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::PortalEnter),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::PortalEnter),
            ],
        );
        let last_frame = report
            .last_frame
            .expect("harness should record the final frame");
        assert_eq!(last_frame.projected_player_state_snapshot_count, 1);
        let candidate = last_frame
            .portal_transit_candidate_probe
            .expect("ready portal pair should expose a player-center candidate probe");
        assert_eq!(candidate.center_coord, LegacyMapTileCoord::new(5, 6));
        assert_eq!(
            candidate.candidate_entry_tile,
            Some(LegacyMapTileCoord::new(5, 6)),
        );
        assert_eq!(
            candidate
                .candidate_pairing
                .expect("center should be inside portal 1 aperture")
                .entry_slot,
            LegacyRuntimePortalSlot::Portal1,
        );
        let preview = last_frame
            .portalcoords_preview
            .expect("ready portal candidate should expose a report-only portalcoords preview");
        assert_eq!(preview.entry_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(preview.exit_slot, LegacyRuntimePortalSlot::Portal2);
        assert_eq!(preview.entry_facing, Facing::Up);
        assert_eq!(preview.exit_facing, Facing::Right);
        assert!(!preview.exit_blocked);
        let outcome = last_frame
            .portal_transit_outcome_summary
            .expect("portalcoords preview should feed a report-only outcome summary");
        assert_eq!(
            outcome.kind,
            LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
        );
        assert_eq!(outcome.entry_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(outcome.exit_slot, LegacyRuntimePortalSlot::Portal2);
        assert_eq!(outcome.blocked_exit_probe, None);
        let audio_intent = last_frame
            .portal_transit_audio_intent
            .expect("successful portal transit should report the portal-enter sound intent");
        assert_eq!(
            audio_intent.outcome_kind,
            LegacyRuntimePortalTransitOutcomeKind::TeleportPreview,
        );
        assert_eq!(audio_intent.entry_slot, LegacyRuntimePortalSlot::Portal1);
        assert_eq!(audio_intent.exit_slot, LegacyRuntimePortalSlot::Portal2);
        assert_eq!(audio_intent.sound, LegacySoundEffect::PortalEnter);
        let expected_snapshot = LegacyRuntimeProjectedPlayerStateSnapshot {
            source: LegacyRuntimeProjectedPlayerStateSource::PortalTransitTeleportPreview,
            entry_slot: LegacyRuntimePortalSlot::Portal1,
            exit_slot: LegacyRuntimePortalSlot::Portal2,
            entry_facing: Facing::Up,
            exit_facing: Facing::Right,
            body: PlayerBodyBounds::new(8.8972225, 4.375, 12.0 / 16.0, 12.0 / 16.0),
            speed_x: 1.3333335,
            speed_y: 0.0,
            animation_direction: HorizontalDirection::Right,
        };
        assert_eq!(
            last_frame.portal_transit_projected_player_snapshot,
            Some(expected_snapshot),
        );
        let projected_player_detail = report
            .projected_player_state_detail_summary
            .last_snapshot
            .expect(
                "harness should retain the last projected player snapshot detail for CLI reporting",
            );
        assert_eq!(projected_player_detail.frame_index, 0);
        assert_eq!(projected_player_detail.snapshot, expected_snapshot);
        assert_eq!(
            last_frame.projected_player_state.latest_portal_transit(),
            Some(expected_snapshot),
        );
    }

    #[test]
    fn harness_counts_completed_block_bounce_item_spawn_intents() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2-2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 30,
            raw_dt: 0.015,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        let block_hit_detail = report
            .tile_collision_detail_summary
            .last_block_hit
            .expect("harness should retain the last ceiling block hit for CLI reporting");
        assert_eq!(block_hit_detail.frame_index, 0);
        assert_eq!(
            block_hit_detail.block_hit.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(block_hit_detail.block_hit.tile_id, TileId(2));
        assert!(block_hit_detail.block_hit.breakable);
        assert!(block_hit_detail.block_hit.coin_block);
        assert_eq!(report.block_bounce_schedule_count, 1);
        assert_eq!(report.contained_reward_reveal_intent_count, 1);
        assert_eq!(report.block_bounce_item_spawn_intent_count, 1);
        let schedule_detail = report
            .block_bounce_detail_summary
            .last_schedule
            .expect("harness should retain the last block-bounce schedule for CLI reporting");
        assert_eq!(schedule_detail.frame_index, 0);
        assert_eq!(
            schedule_detail.schedule.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            schedule_detail.schedule.schedule.coord,
            TileCoord::new(2, 3)
        );
        assert_eq!(
            schedule_detail.schedule.schedule.spawn_content,
            Some(LegacyBlockBounceSpawnKind::Mushroom),
        );
        assert_eq!(schedule_detail.schedule.schedule.hitter_size, 1);
        assert!(schedule_detail.schedule.schedule.regenerate_sprite_batch);
        let reveal_detail = report
            .contained_reward_reveal_detail_summary
            .last_intent
            .expect("harness should retain the last contained reward reveal for CLI reporting");
        assert_eq!(reveal_detail.frame_index, 0);
        assert_eq!(reveal_detail.intent.coord, LegacyMapTileCoord::new(2, 3));
        assert_eq!(
            reveal_detail.intent.content,
            LegacyBlockBounceContentKind::Mushroom
        );
        assert_eq!(
            reveal_detail.intent.outcome.tile_change.coord,
            TileCoord::new(2, 3)
        );
        assert_eq!(reveal_detail.intent.outcome.tile_change.tile, TileId(113));
        assert_eq!(
            reveal_detail.intent.outcome.sound,
            LegacyBlockRevealSound::MushroomAppear
        );
        assert_eq!(report.frame_audio_command_count, 4);
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .block_hit_command_count,
            2
        );
        let block_hit_audio_detail = report
            .frame_audio_command_detail_summary
            .last_block_hit
            .expect("harness should retain source-specific block-hit audio commands");
        assert_eq!(block_hit_audio_detail.frame_index, 0);
        assert_eq!(
            block_hit_audio_detail.block_hit.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            block_hit_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockHit),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockHit),
            ],
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .reward_reveal_command_count,
            2
        );
        let reward_reveal_audio_detail = report
            .frame_audio_command_detail_summary
            .last_reward_reveal
            .expect("harness should retain source-specific reward-reveal audio commands");
        assert_eq!(reward_reveal_audio_detail.frame_index, 0);
        assert_eq!(
            reward_reveal_audio_detail.intent.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            reward_reveal_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::MushroomAppear),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::MushroomAppear),
            ],
        );
        let completion_detail = report
            .block_bounce_detail_summary
            .last_completion
            .expect("harness should retain the last block-bounce completion for CLI reporting");
        assert_eq!(completion_detail.completion.index, 0);
        assert_eq!(completion_detail.completion.coord, TileCoord::new(2, 3));
        assert!(completion_detail.completion.remove);
        assert_eq!(
            completion_detail.completion.suppressed_replay_spawn,
            Some(LegacyBlockBounceReplaySpawn {
                kind: LegacyBlockBounceReplayKind::Mushroom,
                x: 1.5,
                y: 2.875,
            }),
        );
        let spawn_detail = report
            .block_bounce_detail_summary
            .last_item_spawn
            .expect("harness should retain the last block-bounce item spawn for CLI reporting");
        assert_eq!(spawn_detail.intent.source_index, 0);
        assert_eq!(spawn_detail.intent.source_coord, TileCoord::new(2, 3));
        assert_eq!(
            spawn_detail.intent.spawn,
            LegacyBlockBounceReplaySpawn {
                kind: LegacyBlockBounceReplayKind::Mushroom,
                x: 1.5,
                y: 2.875,
            },
        );
        assert_eq!(report.block_bounce_detail_summary.queue_len_after_prune, 0);
        assert_eq!(
            report
                .block_bounce_detail_summary
                .regenerate_sprite_batch_count,
            1,
        );
    }

    #[test]
    fn harness_counts_many_coin_block_reward_intents() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2-5";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.block_bounce_schedule_count, 1);
        assert_eq!(report.coin_block_reward_intent_count, 1);
        assert_eq!(report.many_coins_timer_start_count, 1);
        assert_eq!(report.contained_reward_reveal_intent_count, 0);
        assert_eq!(report.block_bounce_item_spawn_intent_count, 0);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.many_coins_timer_start_count, 1);
        assert_eq!(last_frame.projected_many_coins_timer_count, 1);
        let timer_start_detail = report
            .many_coins_timer_detail_summary
            .last_start
            .expect("harness should retain the last many-coins timer start for CLI reporting");
        assert_eq!(timer_start_detail.frame_index, 0);
        assert_eq!(timer_start_detail.report.reward_index, 0);
        assert_eq!(timer_start_detail.report.coord, TileCoord::new(2, 3));
        assert_close(timer_start_detail.report.duration, 4.0);
        assert_eq!(
            report.many_coins_timer_detail_summary.projected_timer_count,
            1,
        );
        let reward_detail = report
            .coin_block_reward_detail_summary
            .last_reward
            .expect("harness should retain the last coin-block reward for CLI reporting");
        assert_eq!(reward_detail.frame_index, 0);
        assert_eq!(reward_detail.intent.coord, LegacyMapTileCoord::new(2, 3));
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .coin_block_reward_command_count,
            2
        );
        let reward_audio_detail = report
            .frame_audio_command_detail_summary
            .last_coin_block_reward
            .expect("harness should retain source-specific coin-block reward audio commands");
        assert_eq!(reward_audio_detail.frame_index, 0);
        assert_eq!(
            reward_audio_detail.intent.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            reward_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
        assert!(
            reward_detail
                .intent
                .outcome
                .start_many_coins_timer
                .is_some()
        );
    }

    #[test]
    fn harness_counts_report_only_coin_block_animation_progress_and_prunes() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 31,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert!(report.coin_block_reward_intent_count > 0);
        assert!(report.coin_block_animation_progress_count > 0);
        assert!(report.coin_block_animation_prune_count > 0);
        let last_frame = report
            .last_frame
            .as_ref()
            .expect("harness should record the final coin-block animation frame");
        assert!(last_frame.coin_block_animation_progress_count > 0);
        assert!(last_frame.coin_block_animation_prune_count > 0);
        let detail = report
            .effect_animation_detail_summary
            .last_coin_block
            .expect("harness should retain the last coin-block animation report for CLI detail");
        assert!(detail.report.state.frame >= 1);
        let prune_detail = report
            .effect_animation_detail_summary
            .last_coin_block_prune
            .expect("harness should retain the last coin-block animation prune for CLI detail");
        assert!(prune_detail.report.remove);
        assert!(prune_detail.report.state.frame >= 31);
        assert!(prune_detail.report.score.is_some());
        assert!(prune_detail.report.scrolling_score.is_some());
        assert_eq!(
            report
                .effect_animation_detail_summary
                .coin_block_queue_len_after_prune,
            0,
        );
    }

    #[test]
    fn harness_counts_many_coins_timer_progress_reports_from_adapter_snapshot() {
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", level_source(3, "background=1"));
        let config = LegacyRuntimeHarnessConfig {
            frames: 2,
            raw_dt: 0.01,
            many_coins_timers: vec![LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(2, 3),
                remaining: 0.005,
            }],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.many_coins_timer_progress_count, 2);
        assert_eq!(report.many_coins_timer_start_count, 0);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.many_coins_timer_progress_count, 1);
        assert_eq!(last_frame.projected_many_coins_timer_count, 1);
        let timer_progress_detail = report
            .many_coins_timer_detail_summary
            .last_progress
            .expect("harness should retain the last many-coins timer progress for CLI reporting");
        assert_eq!(timer_progress_detail.frame_index, 1);
        assert_eq!(timer_progress_detail.report.index, 0);
        assert_eq!(timer_progress_detail.report.coord, TileCoord::new(2, 3));
        assert_close(timer_progress_detail.report.remaining_before, 0.005);
        assert_close(timer_progress_detail.report.remaining_after, -0.005);
        assert_eq!(
            report.many_coins_timer_detail_summary.projected_timer_count,
            1,
        );
    }

    #[test]
    fn harness_counts_top_coin_collection_intents() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[4] = "3";
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                    LegacyTileMetadata {
                        coin: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.coin_block_reward_intent_count, 1);
        assert_eq!(report.top_coin_collection_intent_count, 1);
        assert_eq!(report.tile_change_projection_count, 2);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.collisions.top_coin_collections.len(), 1);
        assert_eq!(last_frame.tile_change_projection_count, 2);
        let reward_detail = report
            .coin_block_reward_detail_summary
            .last_reward
            .expect("harness should retain the last coin-block reward for CLI reporting");
        assert_eq!(reward_detail.frame_index, 0);
        assert_eq!(reward_detail.intent.coord, LegacyMapTileCoord::new(2, 3));
        let top_coin_detail = report
            .coin_block_reward_detail_summary
            .last_top_coin_collection
            .expect("harness should retain the last top-coin collection for CLI reporting");
        assert_eq!(top_coin_detail.frame_index, 0);
        assert_eq!(
            top_coin_detail.intent.block_coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            top_coin_detail.intent.coin_coord,
            LegacyMapTileCoord::new(2, 2)
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .coin_block_reward_command_count,
            2
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .top_coin_collection_command_count,
            2
        );
        let top_coin_audio_detail = report
            .frame_audio_command_detail_summary
            .last_top_coin_collection
            .expect("harness should retain source-specific top-coin audio commands");
        assert_eq!(top_coin_audio_detail.frame_index, 0);
        assert_eq!(
            top_coin_audio_detail.intent.coin_coord,
            LegacyMapTileCoord::new(2, 2)
        );
        assert_eq!(
            top_coin_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ],
        );
    }

    #[test]
    fn harness_exposes_projected_tile_change_snapshot_counts_and_ordered_history() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[4] = "3";
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                    LegacyTileMetadata {
                        coin: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 2,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        let expected_snapshot = vec![
            LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::CoinBlockReward {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                tile_change: iw2wth_core::LegacyTileChange {
                    coord: TileCoord::new(2, 3),
                    tile: TileId(113),
                },
            },
            LegacyRuntimeTileChangeProjection {
                source: LegacyRuntimeTileChangeSource::TopCoinCollection {
                    block_coord: LegacyMapTileCoord::new(2, 3),
                    coin_coord: LegacyMapTileCoord::new(2, 2),
                },
                tile_change: iw2wth_core::LegacyTileChange {
                    coord: TileCoord::new(2, 2),
                    tile: TileId(1),
                },
            },
        ];

        assert_eq!(report.tile_change_projection_count, 2);
        assert_eq!(report.projected_tile_change_snapshot_count, 2);
        assert_eq!(
            report
                .tile_change_projection_detail_summary
                .projection_count,
            2,
        );
        assert_eq!(
            report
                .tile_change_projection_detail_summary
                .projected_snapshot_count,
            2,
        );
        assert_eq!(
            report
                .tile_change_projection_detail_summary
                .last_projection
                .expect("harness should retain last tile-change detail for CLI reporting")
                .projection,
            expected_snapshot[1],
        );
        assert!(
            report
                .tile_change_projection_detail_summary
                .last_frame_projections
                .is_empty(),
        );
        assert_eq!(
            report
                .tile_change_projection_detail_summary
                .projected_snapshot,
            expected_snapshot.clone(),
        );
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.tile_change_projection_count, 0);
        assert_eq!(last_frame.projected_tile_change_snapshot_count, 2);
        assert_eq!(last_frame.projected_tile_change_snapshot, expected_snapshot);
    }

    #[test]
    fn harness_counts_item_jump_request_intents_from_adapter_snapshots() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            jump_items: vec![LegacyRuntimeBlockJumpItemSnapshot::new(
                LegacyBlockJumpItemKind::Mushroom,
                9,
                0.5,
                1.5,
                1.0,
                0.5,
                true,
            )],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.item_jump_request_intent_count, 1);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.collisions.item_jump_requests.len(), 1);
        let item_detail = report
            .item_jump_request_detail_summary
            .last_intent
            .expect("harness should retain the last item jump request for CLI reporting");
        assert_eq!(item_detail.frame_index, 0);
        assert_eq!(
            item_detail.intent,
            last_frame.collisions.item_jump_requests[0]
        );
        assert_eq!(item_detail.intent.coord, LegacyMapTileCoord::new(2, 3));
        assert_eq!(
            item_detail.intent.request.kind,
            LegacyBlockJumpItemKind::Mushroom
        );
        assert_eq!(item_detail.intent.request.index, 9);
        assert_close(item_detail.intent.request.source_x, 2.0);
    }

    #[test]
    fn harness_counts_enemy_shot_request_intents_from_adapter_snapshots() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            ),
            top_enemies: vec![LegacyRuntimeBlockTopEnemySnapshot::new(
                9, 0.5, 1.5, 1.0, 0.5, true,
            )],
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.enemy_shot_request_intent_count, 1);
        assert_eq!(report.score_counter_intent_count, 2);
        assert_eq!(report.scrolling_score_intent_count, 1);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.collisions.enemy_shot_requests.len(), 1);
        assert_eq!(last_frame.score_counter_intent_count, 2);
        assert_eq!(last_frame.scrolling_score_intent_count, 1);
        let enemy_detail = report
            .enemy_shot_request_detail_summary
            .last_intent
            .expect("harness should retain the last enemy shot request for CLI reporting");
        assert_eq!(enemy_detail.frame_index, 0);
        assert_eq!(
            enemy_detail.intent,
            last_frame.collisions.enemy_shot_requests[0]
        );
        assert_eq!(enemy_detail.intent.coord, LegacyMapTileCoord::new(2, 3));
        assert_eq!(enemy_detail.intent.request.index, 9);
        assert_eq!(
            enemy_detail.intent.request.direction,
            iw2wth_core::LegacyEnemyDirection::Left,
        );
        assert_eq!(enemy_detail.intent.request.score_delta, 100);
        assert_close(enemy_detail.intent.request.score_x, 1.0);
        assert_close(enemy_detail.intent.request.score_y, 1.5);
        let score_detail = report
            .score_counter_detail_summary
            .last_intent
            .expect("harness should retain the last score counter intent for CLI reporting");
        assert_eq!(score_detail.frame_index, 0);
        assert_eq!(
            score_detail.intent.source,
            LegacyRuntimeScoreSource::EnemyShotRequest {
                block_coord: LegacyMapTileCoord::new(2, 3),
                enemy_index: 9,
            },
        );
        assert_eq!(score_detail.intent.score_count_before, 200);
        assert_eq!(score_detail.intent.score_delta, 100);
        assert_eq!(score_detail.intent.score_count_after, 300);
        let scrolling_detail = report
            .scrolling_score_detail_summary
            .last_intent
            .expect("harness should retain the last scrolling score intent for CLI reporting");
        assert_eq!(scrolling_detail.frame_index, 0);
        assert_eq!(
            scrolling_detail.intent.source,
            LegacyRuntimeScoreSource::EnemyShotRequest {
                block_coord: LegacyMapTileCoord::new(2, 3),
                enemy_index: 9,
            },
        );
        let scrolling_score = scrolling_detail
            .intent
            .scrolling_score
            .expect("scrolling-score summary should retain the effect payload");
        assert_eq!(
            scrolling_score.label,
            LegacyScrollingScoreLabel::Points(100)
        );
        assert_close(scrolling_score.x, 1.0);
        assert_close(scrolling_score.y, 1.5);
        assert_close(scrolling_score.timer, 0.0);
    }

    #[test]
    fn harness_counts_empty_breakable_block_destroy_intents() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            )
            .with_big_mario(true),
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.block_bounce_schedule_count, 1);
        assert_eq!(report.empty_breakable_block_destroy_intent_count, 1);
        assert_eq!(report.coin_block_reward_intent_count, 0);
        assert_eq!(report.tile_change_projection_count, 1);
        assert_eq!(report.breakable_block_cleanup_projection_count, 7);
        assert_eq!(
            report
                .breakable_block_cleanup_projection_detail_summary
                .projection_count,
            7,
        );
        assert_eq!(report.score_counter_intent_count, 1);
        assert_eq!(report.scrolling_score_intent_count, 0);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(
            last_frame.collisions.empty_breakable_block_destroys.len(),
            1
        );
        assert_eq!(last_frame.tile_change_projection_count, 1);
        assert_eq!(last_frame.breakable_block_cleanup_projection_count, 7);
        assert_eq!(
            report
                .breakable_block_cleanup_projection_detail_summary
                .last_frame_projections,
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
                        debris: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -23.0,),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 1,
                        debris: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -23.0,),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 2,
                        debris: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -14.0,),
                    },
                },
                LegacyRuntimeBreakableBlockCleanupProjection {
                    source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                        coord: LegacyMapTileCoord::new(2, 3),
                    },
                    action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                        index: 3,
                        debris: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 2.5, -3.5, -14.0,),
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
            report
                .breakable_block_cleanup_projection_detail_summary
                .last_projection
                .expect("harness should retain last cleanup detail for CLI reporting")
                .projection,
            LegacyRuntimeBreakableBlockCleanupProjection {
                source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                    coord: LegacyMapTileCoord::new(2, 3),
                },
                action: LegacyRuntimeBreakableBlockCleanupAction::RegenerateSpriteBatch,
            },
        );
        assert_eq!(
            report
                .frame_audio_command_detail_summary
                .block_break_command_count,
            2
        );
        let block_break_audio_detail = report
            .frame_audio_command_detail_summary
            .last_block_break
            .expect("harness should retain source-specific block-break audio commands");
        assert_eq!(block_break_audio_detail.frame_index, 0);
        assert_eq!(
            block_break_audio_detail.intent.coord,
            LegacyMapTileCoord::new(2, 3)
        );
        assert_eq!(
            block_break_audio_detail.commands,
            vec![
                LegacyAudioCommand::StopSound(LegacySoundEffect::BlockBreak),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::BlockBreak),
            ],
        );
        assert_eq!(last_frame.score_counter_intent_count, 1);
        assert_eq!(last_frame.scrolling_score_intent_count, 0);
    }

    #[test]
    fn harness_exposes_projected_portal_block_hit_guard_detail_without_live_effects() {
        let mut cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let mut projected_portal_state = LegacyRuntimeProjectedPortalState::default();
        projected_portal_state.apply_projection(LegacyRuntimePortalReservationProjection {
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
        let config = LegacyRuntimeHarnessConfig {
            frames: 1,
            initial_player: super::LegacyRuntimePlayer::new(
                PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                PlayerMovementState {
                    speed_y: -80.0,
                    jumping: true,
                    ..PlayerMovementState::default()
                },
            )
            .with_big_mario(true),
            initial_projected_portal_state: projected_portal_state,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report = match run_legacy_runtime_harness(&source, config) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(report.ceiling_block_hit_count, 1);
        assert_eq!(report.block_hit_portal_guard_suppression_count, 1);
        assert_eq!(report.block_hit_projected_portal_guard_suppression_count, 1);
        assert_eq!(
            report
                .block_hit_portal_guard_summary
                .projected_portal_state_count,
            1
        );
        assert_eq!(
            report
                .block_hit_portal_guard_summary
                .explicit_reservation_count,
            0
        );
        let detail = match report.block_hit_portal_guard_summary.last_guard {
            Some(detail) => detail,
            None => panic!("projected portal guard should be exposed"),
        };
        assert_eq!(detail.frame_index, 0);
        assert_eq!(detail.coord, LegacyMapTileCoord::new(2, 3));
        assert_eq!(detail.tile_id, TileId(2));
        assert!(detail.breakable);
        assert!(!detail.coin_block);
        assert!(!detail.play_hit_sound);
        assert_eq!(
            detail.guard.source,
            LegacyRuntimePortalBlockGuardSource::ProjectedPortalState
        );
        assert_eq!(
            detail.guard.reservation,
            LegacyBlockPortalReservation::new(TileCoord::new(2, 3), Facing::Right)
        );
        assert_eq!(report.block_bounce_schedule_count, 0);
        assert_eq!(report.empty_breakable_block_destroy_intent_count, 1);
        assert_eq!(report.tile_change_projection_count, 0);
        assert_eq!(report.breakable_block_cleanup_projection_count, 0);
        let last_frame = match report.last_frame {
            Some(frame) => frame,
            None => panic!("harness should record the final frame"),
        };
        assert_eq!(last_frame.collisions.block_hits.len(), 1);
        assert_eq!(
            last_frame.collisions.block_hits[0].portal_guard,
            Some(detail.guard)
        );
        assert!(last_frame.collisions.block_bounce_schedules.is_empty());
    }

    #[test]
    fn harness_counts_report_only_effect_animation_progress_and_prunes() {
        let mut score_cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        score_cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let score_source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        coin_block: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", score_cells.join(","));
        let score_report = match run_legacy_runtime_harness(
            &score_source,
            LegacyRuntimeHarnessConfig {
                frames: 50,
                initial_player: super::LegacyRuntimePlayer::new(
                    PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                    PlayerMovementState {
                        speed_y: -80.0,
                        jumping: true,
                        ..PlayerMovementState::default()
                    },
                ),
                top_enemies: vec![LegacyRuntimeBlockTopEnemySnapshot::new(
                    9, 0.5, 1.5, 1.0, 0.5, true,
                )],
                ..LegacyRuntimeHarnessConfig::default()
            },
        ) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert!(score_report.scrolling_score_intent_count > 0);
        assert!(score_report.scrolling_score_animation_progress_count > 0);
        assert!(score_report.scrolling_score_animation_prune_count > 0);
        let score_last_frame = score_report
            .last_frame
            .as_ref()
            .expect("harness should record the final scrolling-score frame");
        assert!(score_last_frame.scrolling_score_animation_progress_count > 0);
        assert!(score_last_frame.scrolling_score_animation_prune_count > 0);
        let score_detail = score_report
            .effect_animation_detail_summary
            .last_scrolling_score
            .expect(
                "harness should retain the last scrolling-score animation report for CLI detail",
            );
        assert!(matches!(
            score_detail.report.state.label,
            LegacyScrollingScoreLabel::Points(100) | LegacyScrollingScoreLabel::Points(200)
        ));
        assert!(
            score_report
                .effect_animation_detail_summary
                .scrolling_score_queue_len_after_prune
                <= score_report.scrolling_score_animation_progress_count
        );
        let score_prune_detail = score_report
            .effect_animation_detail_summary
            .last_scrolling_score_prune
            .expect(
                "harness should retain the last scrolling-score animation prune for CLI detail",
            );
        assert!(score_prune_detail.report.remove);
        assert!(matches!(
            score_prune_detail.report.state.label,
            LegacyScrollingScoreLabel::Points(100) | LegacyScrollingScoreLabel::Points(200)
        ));

        let mut debris_cells = vec!["1"; 3 * MARI0_LEVEL_HEIGHT];
        debris_cells[(3 - 1) * 3 + (2 - 1)] = "2";
        let debris_source = BufferedLegacyAssetSource::new()
            .with_file_bytes(
                "graphics/SMB/smbtiles.png",
                atlas_png_from_metadata(&[
                    LegacyTileMetadata::empty(),
                    LegacyTileMetadata {
                        collision: true,
                        breakable: true,
                        ..LegacyTileMetadata::empty()
                    },
                ]),
            )
            .with_file_bytes(
                "graphics/SMB/portaltiles.png",
                atlas_png_from_metadata(&[LegacyTileMetadata::empty()]),
            )
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", debris_cells.join(","));
        let debris_report = match run_legacy_runtime_harness(
            &debris_source,
            LegacyRuntimeHarnessConfig {
                frames: 69,
                initial_player: super::LegacyRuntimePlayer::new(
                    PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
                    PlayerMovementState {
                        speed_y: -80.0,
                        jumping: true,
                        ..PlayerMovementState::default()
                    },
                )
                .with_big_mario(true),
                ..LegacyRuntimeHarnessConfig::default()
            },
        ) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        };

        assert!(debris_report.empty_breakable_block_destroy_intent_count > 0);
        assert!(debris_report.block_debris_animation_progress_count > 0);
        assert!(debris_report.block_debris_animation_prune_count > 0);
        let debris_last_frame = debris_report
            .last_frame
            .as_ref()
            .expect("harness should record the final block-debris frame");
        assert!(debris_last_frame.block_debris_animation_progress_count > 0);
        assert!(debris_last_frame.block_debris_animation_prune_count > 0);
        let debris_detail = debris_report
            .effect_animation_detail_summary
            .last_block_debris
            .expect("harness should retain the last block-debris animation report for CLI detail");
        assert_eq!(
            debris_detail.report.source,
            LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                coord: LegacyMapTileCoord::new(2, 3),
            },
        );
        assert_eq!(
            debris_report
                .effect_animation_detail_summary
                .block_debris_queue_len_after_prune,
            0,
        );
        let debris_prune_detail = debris_report
            .effect_animation_detail_summary
            .last_block_debris_prune
            .expect("harness should retain the last block-debris animation prune for CLI detail");
        assert!(debris_prune_detail.report.remove);
        assert_eq!(
            debris_prune_detail.report.source,
            LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                coord: LegacyMapTileCoord::new(2, 3),
            },
        );
    }

    #[test]
    fn harness_discovers_legacy_player_spawn_entities_before_fixed_seed() {
        let mut cells = vec!["1"; 4 * MARI0_LEVEL_HEIGHT];
        cells[(13 - 1) * 4 + (2 - 1)] = "1-8";
        cells[(12 - 1) * 4 + (4 - 1)] = "1-8";
        let level = Mari0Level::parse(&cells.join(",")).expect("test level should parse");
        let fallback = super::LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState::default(),
        );

        let spawn = legacy_runtime_player_spawn(&level, fallback);

        assert_eq!(
            spawn.source,
            LegacyRuntimePlayerSpawnSource::LegacyPlayerSpawnEntity,
        );
        assert_eq!(spawn.coord, Some(LegacyMapTileCoord::new(2, 13)));
        assert_eq!(spawn.player.body.x, 2.0 - 6.0 / 16.0);
        assert_eq!(spawn.player.body.y, 12.0);
        assert_eq!(spawn.player.body.width, 12.0 / 16.0);
        assert_eq!(spawn.player.movement, PlayerMovementState::default());
    }

    #[test]
    fn harness_keeps_fixed_seed_when_no_player_spawn_exists() {
        let level =
            Mari0Level::parse(&level_source(2, "background=1")).expect("test level should parse");
        let fallback = super::LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.0, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState {
                speed_x: 2.0,
                ..PlayerMovementState::default()
            },
        );

        let spawn = legacy_runtime_player_spawn(&level, fallback);

        assert_eq!(
            spawn.source,
            LegacyRuntimePlayerSpawnSource::FixedSeedFallback,
        );
        assert_eq!(spawn.coord, None);
        assert_eq!(spawn.player, fallback);
    }

    #[test]
    fn harness_can_force_configured_player_seed_for_cli_repro_runs() {
        let mut cells = vec!["1"; 4 * MARI0_LEVEL_HEIGHT];
        cells[(13 - 1) * 4 + (2 - 1)] = "1-8";
        let source = source_with_base_tile_atlases()
            .with_file_contents("mappacks/smb/settings.txt", "name=Super Mario Bros.\n")
            .with_file_contents("mappacks/smb/1-1.txt", cells.join(","));
        let seeded_player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(1.0, 3.2, 12.0 / 16.0, 12.0 / 16.0),
            PlayerMovementState {
                speed_y: -80.0,
                ..PlayerMovementState::default()
            },
        );
        let config = LegacyRuntimeHarnessConfig {
            frames: 0,
            force_initial_player_seed: true,
            initial_player: seeded_player,
            ..LegacyRuntimeHarnessConfig::default()
        };

        let report =
            run_legacy_runtime_harness(&source, config).expect("configured player seed should run");

        assert_eq!(
            report.player_spawn.source,
            LegacyRuntimePlayerSpawnSource::ConfiguredSeed,
        );
        assert_eq!(report.player_spawn.coord, None);
        assert_eq!(report.player_spawn.player, seeded_player);
        assert_eq!(report.final_player, seeded_player);
    }
}

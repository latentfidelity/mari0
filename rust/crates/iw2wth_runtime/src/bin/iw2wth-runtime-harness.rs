use std::{env, error::Error, path::PathBuf};

use iw2wth_core::{
    Facing, HorizontalDirection, LegacyBlockBounceContentKind, LegacyBlockBounceReplayKind,
    LegacyBlockBounceReplaySpawn, LegacyBlockBounceSpawnKind, LegacyBlockJumpItemKind,
    LegacyBlockRevealSound, LegacyCoinBlockAnimationScore, LegacyCoinBlockAnimationState,
    LegacyCoinBlockTimerSpawn, LegacyCoinLifeReward, LegacyEnemyDirection,
    LegacyFireballCollisionTarget, LegacyFireballFrame, LegacyFireballState,
    LegacyManyCoinsTimerEntry, LegacyMapTileCoord, LegacyScrollingScoreLabel,
    LegacyScrollingScoreState, LegacyTileChange, PlayerBodyBounds, TileCoord,
};
use iw2wth_runtime::{
    assets::FsLegacyAssetSource,
    audio::{LegacyAudioCommand, LegacySoundEffect},
    harness::{LegacyRuntimeHarnessConfig, LegacyRuntimeHarnessInput, run_legacy_runtime_harness},
    shell::{
        LegacyRuntimeBlockBounceCompletionReport, LegacyRuntimeBlockBounceItemSpawnIntent,
        LegacyRuntimeBlockContainedRewardRevealIntent,
        LegacyRuntimeBlockDebrisAnimationUpdateReport, LegacyRuntimeBlockJumpItemSnapshot,
        LegacyRuntimeBlockTopCoinCollectionIntent, LegacyRuntimeBlockTopEnemySnapshot,
        LegacyRuntimeBreakableBlockCleanupAction, LegacyRuntimeBreakableBlockCleanupProjection,
        LegacyRuntimeBreakableBlockCleanupSource, LegacyRuntimeCoinBlockAnimationUpdateReport,
        LegacyRuntimeCoinBlockRewardIntent, LegacyRuntimeCoinCounterSource,
        LegacyRuntimeFireballCallback, LegacyRuntimeFireballCollisionAxis,
        LegacyRuntimeFireballCollisionProbe, LegacyRuntimeFireballCollisionProbeRequest,
        LegacyRuntimeFireballCollisionProbeSource, LegacyRuntimeFireballEnemyHitIntent,
        LegacyRuntimeFireballEnemySnapshot, LegacyRuntimeFireballLaunchIntent,
        LegacyRuntimeFireballMapTargetProbe, LegacyRuntimeFireballProjectileReleaseSource,
        LegacyRuntimeFireballProjectileReleaseSummary, LegacyRuntimeFireballRenderFrameKind,
        LegacyRuntimeFireballRenderIntentPreview, LegacyRuntimeFireballRenderSource,
        LegacyRuntimePlayerBlockBounceSchedule, LegacyRuntimePlayerCeilingBlockHit,
        LegacyRuntimePlayerCoinPickup, LegacyRuntimePlayerCollisionAxis,
        LegacyRuntimePlayerPowerUp, LegacyRuntimePlayerRenderColorLayerPreview,
        LegacyRuntimePlayerRenderFrame, LegacyRuntimePlayerRenderHatPreview,
        LegacyRuntimePlayerRenderHatSize, LegacyRuntimePlayerRenderIntentPreview,
        LegacyRuntimePlayerRenderQuad, LegacyRuntimePlayerRenderTintSource,
        LegacyRuntimePlayerTileCollision, LegacyRuntimePortalBlockGuardSource,
        LegacyRuntimePortalBlockedExitBounceAxis, LegacyRuntimePortalBlockedExitProbe,
        LegacyRuntimePortalOutcomeIntent, LegacyRuntimePortalOutcomeKind,
        LegacyRuntimePortalPairing, LegacyRuntimePortalPlacement, LegacyRuntimePortalSlot,
        LegacyRuntimePortalTargetPlayerSource, LegacyRuntimePortalTraceHit,
        LegacyRuntimePortalTransitAudioIntent, LegacyRuntimePortalTransitOutcomeKind,
        LegacyRuntimePortalWallReservation, LegacyRuntimeProjectedFireballCountSnapshot,
        LegacyRuntimeProjectedFireballCountSource, LegacyRuntimeProjectedFireballEnemyHitSnapshot,
        LegacyRuntimeProjectedFireballProjectileCollisionSnapshot,
        LegacyRuntimeProjectedPlayerStateSource, LegacyRuntimeProjectedPortal,
        LegacyRuntimeScoreSource, LegacyRuntimeScrollingScoreAnimationUpdateReport,
        LegacyRuntimeTileChangeProjection, LegacyRuntimeTileChangeSource,
        legacy_runtime_portal_reservation_projection,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let (repo_root, config) = parse_args(args)?;
    let source = FsLegacyAssetSource::new(repo_root);
    let report = run_legacy_runtime_harness(&source, config)?;

    println!(
        "IW2WTH Rust runtime harness: mappack={} level={} frames={}",
        report.selection.mappack, report.selection.filename, report.frame_count,
    );
    println!(
        "loaded: settings={} width={} custom_tiles={} backgrounds={}",
        report.settings_name.as_deref().unwrap_or(""),
        report.level_width,
        report.custom_tiles,
        report.background_count,
    );
    match report.player_spawn.coord {
        Some(coord) => println!(
            "spawn: source={:?} tile=({}, {})",
            report.player_spawn.source, coord.x, coord.y,
        ),
        None => println!("spawn: source={:?}", report.player_spawn.source),
    }
    println!(
        "player: x={:.6} y={:.6} speed_x={:.6} speed_y={:.6} falling={}",
        report.final_player.body.x,
        report.final_player.body.y,
        report.final_player.movement.speed_x,
        report.final_player.movement.speed_y,
        report.final_player.movement.falling,
    );
    match report.player_render_preview_detail_summary.last_preview {
        Some(detail) => println!(
            "player render preview: total={} last_frame={} last_index={} last_power_up={} last_size={} last_render_frame={} last_animation_state={:?} last_facing={} last_run_frame={} last_swim_frame={} last_ducking={} last_fire_animation_timer={:.6} last_fire_animation_active={} last_quad={} last_color_layers={} last_hat_draws={} last_draw_x={:.6} last_draw_y={:.6} last_scale={:.6} last_live_rendering_executed={} last_live_player_mutated={} last_detail={}",
            report.player_render_preview_count,
            detail.frame_index,
            detail.preview.player_index,
            player_power_up_label(detail.preview.power_up),
            detail.preview.size,
            player_render_frame_label(detail.preview.render_frame),
            detail.preview.animation_state,
            horizontal_direction_label(detail.preview.facing),
            detail.preview.run_frame,
            detail.preview.swim_frame,
            bool_label(detail.preview.ducking),
            detail.preview.fire_animation_timer,
            bool_label(detail.preview.fire_animation_active),
            player_render_quad_label(detail.preview.quad),
            player_render_color_layers_label(&detail.preview.color_layers),
            player_render_hat_draws_label(&detail.preview.hat_draws, detail.preview.hat_draw_count),
            detail.preview.draw_x_px,
            detail.preview.draw_y_px,
            detail.preview.scale,
            detail.preview.live_rendering_executed,
            detail.preview.live_player_mutated,
            player_render_preview_label(detail.preview),
        ),
        None => println!(
            "player render preview: total={} last_frame=none last_index=none last_power_up=none last_size=none last_render_frame=none last_animation_state=none last_facing=none last_run_frame=none last_swim_frame=none last_ducking=none last_fire_animation_timer=none last_fire_animation_active=none last_quad=none last_color_layers=none last_hat_draws=none last_draw_x=none last_draw_y=none last_scale=none last_live_rendering_executed=none last_live_player_mutated=none last_detail=none",
            report.player_render_preview_count,
        ),
    }
    match report.fireball_launch_detail_summary.last_intent {
        Some(detail) => println!(
            "player fireball launch: total={} last_frame={} last_direction={} last_source_x={:.6} last_source_y={:.6} last_spawn_x={:.6} last_spawn_y={:.6} last_speed_x={:.6} last_count_before={} last_count_after={} last_sound={} last_detail={}",
            report.fireball_launch_intent_count,
            detail.frame_index,
            enemy_direction_label(detail.intent.direction),
            detail.intent.source_x,
            detail.intent.source_y,
            detail.intent.spawn.x,
            detail.intent.spawn.y,
            detail.intent.spawn.speed_x,
            detail.intent.fireball_count_before,
            detail.intent.fireball_count_after,
            sound_effect_label(detail.intent.sound),
            fireball_launch_intent_label(detail.intent),
        ),
        None => println!(
            "player fireball launch: total={} last_frame=none last_direction=none last_source_x=none last_source_y=none last_spawn_x=none last_spawn_y=none last_speed_x=none last_count_before=none last_count_after=none last_sound=none last_detail=none",
            report.fireball_launch_intent_count,
        ),
    }
    match report.fireball_projectile_detail_summary.last_progress {
        Some(detail) => println!(
            "player fireball projectile progress: total={} prunes={} last_frame={} last_index={} last_before_frame={} last_after_frame={} last_after_timer={:.6} last_remove={} last_released_thrower={} last_queue_after_prune={}",
            report.fireball_projectile_progress_count,
            report.fireball_projectile_prune_count,
            detail.frame_index,
            detail.progress.index,
            fireball_frame_label(detail.progress.state_before.frame),
            fireball_frame_label(detail.progress.state_after.frame),
            detail.progress.state_after.timer,
            detail.progress.update.remove,
            detail.progress.update.released_thrower,
            detail.queue_len_after_prune,
        ),
        None => println!(
            "player fireball projectile progress: total={} prunes={} last_frame=none last_index=none last_before_frame=none last_after_frame=none last_after_timer=none last_remove=none last_released_thrower=none last_queue_after_prune=none",
            report.fireball_projectile_progress_count, report.fireball_projectile_prune_count,
        ),
    }
    match report.fireball_render_preview_detail_summary.last_preview {
        Some(detail) => println!(
            "player fireball render preview: total={} suppressed_projected_removals={} last_frame={} last_index={} last_source={} last_kind={} last_projectile_frame={} last_quad={} last_draw_x={:.6} last_draw_y={:.6} last_rotation={:.6} last_scale={:.6} last_live_rendering_executed={} last_live_queue_mutated={} last_suppressed_projected_removal_index={} last_detail={}",
            report.fireball_render_preview_count,
            report.fireball_render_preview_suppressed_count,
            detail.frame_index,
            detail.preview.projectile_index,
            fireball_render_source_label(detail.preview.source),
            fireball_render_frame_kind_label(detail.preview.frame_kind),
            fireball_frame_label(detail.preview.frame),
            fireball_render_quad_label(detail.preview),
            detail.preview.draw_x_px,
            detail.preview.draw_y_px,
            detail.preview.rotation,
            detail.preview.scale,
            detail.preview.live_rendering_executed,
            detail.preview.live_projectile_queue_mutated,
            report
                .fireball_render_preview_detail_summary
                .last_suppressed_projected_removal_index
                .map_or_else(|| "none".to_owned(), |index| index.to_string()),
            fireball_render_preview_label(detail.preview),
        ),
        None => println!(
            "player fireball render preview: total={} suppressed_projected_removals={} last_frame=none last_index=none last_source=none last_kind=none last_projectile_frame=none last_quad=none last_draw_x=none last_draw_y=none last_rotation=none last_scale=none last_live_rendering_executed=none last_live_queue_mutated=none last_suppressed_projected_removal_index={} last_detail=none",
            report.fireball_render_preview_count,
            report.fireball_render_preview_suppressed_count,
            report
                .fireball_render_preview_detail_summary
                .last_suppressed_projected_removal_index
                .map_or_else(|| "none".to_owned(), |index| index.to_string()),
        ),
    }
    match report.fireball_map_target_detail_summary.last_probe {
        Some(detail) => println!(
            "player fireball map target probe: total={} last_frame={} last_index={} last_coord={} last_tile_id={} last_axis={} last_collides={} last_invisible={} last_breakable={} last_coin_block={} last_block_hit_sound={} last_live_collision_mutated={} last_detail={}",
            report.fireball_map_target_probe_count,
            detail.frame_index,
            detail.probe.projectile_index,
            coord_label(detail.probe.coord),
            detail.probe.tile_id.0,
            fireball_collision_axis_label(detail.probe.axis),
            detail.probe.collides,
            detail.probe.invisible,
            detail.probe.breakable,
            detail.probe.coin_block,
            detail.probe.play_block_hit_sound,
            detail.probe.live_projectile_collision_mutated,
            fireball_map_target_probe_label(detail.probe),
        ),
        None => println!(
            "player fireball map target probe: total={} last_frame=none last_index=none last_coord=none last_tile_id=none last_axis=none last_collides=none last_invisible=none last_breakable=none last_coin_block=none last_block_hit_sound=none last_live_collision_mutated=none last_detail=none",
            report.fireball_map_target_probe_count,
        ),
    }
    match report.fireball_collision_detail_summary.last_probe {
        Some(detail) => println!(
            "player fireball collision probe: total={} explicit_total={} map_derived_total={} enemy_overlap_total={} last_frame={} last_source={} last_index={} last_axis={} last_target={} last_before_active={} last_after_active={} last_after_destroy_soon={} last_speed_x={:.6} last_speed_y={:.6} last_suppress_default={} last_released_thrower={} last_block_hit_sound={} last_shoot_target={} last_points={} last_detail={}",
            report.fireball_collision_probe_count,
            report
                .fireball_collision_detail_summary
                .explicit_probe_count,
            report
                .fireball_collision_detail_summary
                .map_derived_probe_count,
            report
                .fireball_collision_detail_summary
                .enemy_overlap_probe_count,
            detail.frame_index,
            fireball_collision_probe_source_label(detail.probe.source),
            detail.probe.projectile_index,
            fireball_collision_axis_label(detail.probe.axis),
            fireball_collision_target_label(detail.probe.target),
            detail.probe.state_before.active,
            detail.probe.state_after.active,
            detail.probe.state_after.destroy_soon,
            detail.probe.state_after.speed_x,
            detail.probe.state_after.speed_y,
            detail.probe.outcome.suppress_default,
            detail.probe.outcome.released_thrower,
            detail.probe.outcome.play_block_hit_sound,
            detail
                .probe
                .outcome
                .shoot_target
                .map(enemy_direction_label)
                .unwrap_or("none"),
            detail
                .probe
                .outcome
                .points
                .map_or_else(|| "none".to_owned(), |points| points.to_string()),
            fireball_collision_probe_label(detail.probe),
        ),
        None => println!(
            "player fireball collision probe: total={} explicit_total={} map_derived_total={} enemy_overlap_total={} last_frame=none last_source=none last_index=none last_axis=none last_target=none last_before_active=none last_after_active=none last_after_destroy_soon=none last_speed_x=none last_speed_y=none last_suppress_default=none last_released_thrower=none last_block_hit_sound=none last_shoot_target=none last_points=none last_detail=none",
            report.fireball_collision_probe_count,
            report
                .fireball_collision_detail_summary
                .explicit_probe_count,
            report
                .fireball_collision_detail_summary
                .map_derived_probe_count,
            report
                .fireball_collision_detail_summary
                .enemy_overlap_probe_count,
        ),
    }
    match report.fireball_enemy_hit_detail_summary.last_intent {
        Some(detail) => println!(
            "player fireball enemy hit intent: total={} last_frame={} last_projectile_index={} last_source={} last_axis={} last_target={} last_enemy_index={} last_shot_direction={} last_score_delta={} last_score_x={:.6} last_score_y={:.6} last_live_enemy_mutated={} last_detail={}",
            report.fireball_enemy_hit_intent_count,
            detail.frame_index,
            detail.intent.projectile_index,
            fireball_collision_probe_source_label(detail.intent.source),
            fireball_collision_axis_label(detail.intent.axis),
            fireball_collision_target_label(detail.intent.target),
            detail.intent.enemy.index,
            enemy_direction_label(detail.intent.shot_direction),
            optional_u32_label(detail.intent.score_delta),
            detail.intent.score_x,
            detail.intent.score_y,
            detail.intent.live_enemy_mutated,
            fireball_enemy_hit_intent_label(detail.intent),
        ),
        None => println!(
            "player fireball enemy hit intent: total={} last_frame=none last_projectile_index=none last_source=none last_axis=none last_target=none last_enemy_index=none last_shot_direction=none last_score_delta=none last_score_x=none last_score_y=none last_live_enemy_mutated=none last_detail=none",
            report.fireball_enemy_hit_intent_count,
        ),
    }
    match report
        .projected_fireball_enemy_hit_detail_summary
        .last_snapshot
    {
        Some(detail) => println!(
            "projected fireball enemy hit snapshot: total={} last_frame={} last_projectile_index={} last_target={} last_enemy_index={} last_active_after={} last_shot_after={} last_removed_from_future_queries={} last_live_enemy_mutated={} last_detail={}",
            report.projected_fireball_enemy_hit_snapshot_count,
            detail.frame_index,
            detail.snapshot.intent.projectile_index,
            fireball_collision_target_label(detail.snapshot.intent.target),
            detail.snapshot.enemy.index,
            detail.snapshot.active_after,
            detail.snapshot.shot_after,
            detail.snapshot.removed_from_future_queries,
            detail.snapshot.live_enemy_mutated,
            projected_fireball_enemy_hit_snapshot_label(detail.snapshot),
        ),
        None => println!(
            "projected fireball enemy hit snapshot: total={} last_frame=none last_projectile_index=none last_target=none last_enemy_index=none last_active_after=none last_shot_after=none last_removed_from_future_queries=none last_live_enemy_mutated=none last_detail=none",
            report.projected_fireball_enemy_hit_snapshot_count,
        ),
    }
    match report
        .projected_fireball_projectile_collision_detail_summary
        .last_snapshot
    {
        Some(detail) => println!(
            "projected fireball projectile collision snapshot: total={} last_frame={} last_projectile_index={} last_source={} last_axis={} last_target={} last_after_active={} last_after_destroy_soon={} last_removed_from_future_queries={} last_live_projectile_queue_mutated={} last_detail={}",
            report.projected_fireball_projectile_collision_snapshot_count,
            detail.frame_index,
            detail.snapshot.projectile_index,
            fireball_collision_probe_source_label(detail.snapshot.source),
            fireball_collision_axis_label(detail.snapshot.axis),
            fireball_collision_target_label(detail.snapshot.target),
            detail.snapshot.state_after.active,
            detail.snapshot.state_after.destroy_soon,
            detail.snapshot.removed_from_future_collision_queries,
            detail.snapshot.live_projectile_queue_mutated,
            projected_fireball_projectile_collision_snapshot_label(detail.snapshot),
        ),
        None => println!(
            "projected fireball projectile collision snapshot: total={} last_frame=none last_projectile_index=none last_source=none last_axis=none last_target=none last_after_active=none last_after_destroy_soon=none last_removed_from_future_queries=none last_live_projectile_queue_mutated=none last_detail=none",
            report.projected_fireball_projectile_collision_snapshot_count,
        ),
    }
    match report
        .fireball_collision_detail_summary
        .last_release_summary
    {
        Some(detail) => println!(
            "player fireball release summary: total={} last_frame={} last_index={} last_axis={} last_target={} last_callback={} last_count_delta={} last_live_queue_mutated={} last_live_counter_mutated={} last_detail={}",
            report.fireball_collision_release_summary_count,
            detail.frame_index,
            detail.summary.projectile_index,
            fireball_release_axis_label(detail.summary.source),
            fireball_release_target_label(detail.summary.source),
            fireball_callback_label(detail.summary.callback.callback),
            detail.summary.callback.fireball_count_delta,
            detail.summary.live_projectile_queue_mutated,
            detail.summary.live_fireball_counter_mutated,
            fireball_release_summary_label(detail.summary),
        ),
        None => println!(
            "player fireball release summary: total={} last_frame=none last_index=none last_axis=none last_target=none last_callback=none last_count_delta=none last_live_queue_mutated=none last_live_counter_mutated=none last_detail=none",
            report.fireball_collision_release_summary_count,
        ),
    }
    match report.projected_fireball_count_detail_summary.last_snapshot {
        Some(detail) => println!(
            "projected fireball count: total={} last_frame={} last_source={} last_count_before={} last_delta={} last_count_after={} last_live_counter_mutated={} last_detail={}",
            report.projected_fireball_count_snapshot_count,
            detail.frame_index,
            projected_fireball_count_source_label(detail.snapshot.source),
            detail.snapshot.active_fireball_count_before,
            detail.snapshot.fireball_count_delta,
            detail.snapshot.active_fireball_count_after,
            detail.snapshot.live_fireball_counter_mutated,
            projected_fireball_count_snapshot_label(detail.snapshot),
        ),
        None => println!(
            "projected fireball count: total={} last_frame=none last_source=none last_count_before=none last_delta=none last_count_after=none last_live_counter_mutated=none last_detail=none",
            report.projected_fireball_count_snapshot_count,
        ),
    }
    println!(
        "map interactions: coin_pickups={} coin_counter_intents={} score_counter_intents={} scrolling_score_intents={} life_reward_counter_intents={} horizontal_collisions={} vertical_collisions={} ceiling_block_hits={} block_hit_portal_guard_suppressions={} block_hit_projected_portal_guard_suppressions={} block_bounce_schedules={} coin_block_reward_intents={} top_coin_collection_intents={} tile_change_projections={} projected_tile_change_snapshots={} breakable_block_cleanup_projections={} coin_block_animation_progressions={} coin_block_animation_prunes={} block_debris_animation_progressions={} block_debris_animation_prunes={} scrolling_score_animation_progressions={} scrolling_score_animation_prunes={} item_jump_request_intents={} enemy_shot_request_intents={} empty_breakable_block_destroy_intents={} contained_reward_reveal_intents={} block_bounce_progressions={} block_bounce_prunes={} block_bounce_item_spawn_intents={} many_coins_timer_progressions={} many_coins_timer_starts={} player_render_previews={} fireball_render_previews={} fireball_render_preview_suppressions={} fireball_map_target_probes={} fireball_collision_probes={} projected_fireball_projectile_collision_snapshots={} fireball_enemy_hit_intents={} projected_fireball_enemy_hit_snapshots={} portal_target_probes={} portal_target_projected_player_sources={} portal_targets_possible={} portal_open_intents={} portal_fizzle_intents={} portal_reservation_projections={} portal_replacement_summaries={} projected_portal_state_snapshots={} portal_pair_readiness_summaries={} portal_pairs_ready={} portal_transit_candidate_probes={} portal_transit_candidates_ready={} portalcoords_previews={} portal_transit_outcome_summaries={} portal_transit_audio_intents={} portal_transit_success_previews={} portal_transit_blocked_exit_bounce_previews={} portal_transit_projected_player_snapshots={} projected_player_state_snapshots={}",
        report.coin_pickup_count,
        report.coin_counter_intent_count,
        report.score_counter_intent_count,
        report.scrolling_score_intent_count,
        report.life_reward_counter_intent_count,
        report.horizontal_collision_count,
        report.vertical_collision_count,
        report.ceiling_block_hit_count,
        report.block_hit_portal_guard_suppression_count,
        report.block_hit_projected_portal_guard_suppression_count,
        report.block_bounce_schedule_count,
        report.coin_block_reward_intent_count,
        report.top_coin_collection_intent_count,
        report.tile_change_projection_count,
        report.projected_tile_change_snapshot_count,
        report.breakable_block_cleanup_projection_count,
        report.coin_block_animation_progress_count,
        report.coin_block_animation_prune_count,
        report.block_debris_animation_progress_count,
        report.block_debris_animation_prune_count,
        report.scrolling_score_animation_progress_count,
        report.scrolling_score_animation_prune_count,
        report.item_jump_request_intent_count,
        report.enemy_shot_request_intent_count,
        report.empty_breakable_block_destroy_intent_count,
        report.contained_reward_reveal_intent_count,
        report.block_bounce_progress_count,
        report.block_bounce_prune_count,
        report.block_bounce_item_spawn_intent_count,
        report.many_coins_timer_progress_count,
        report.many_coins_timer_start_count,
        report.player_render_preview_count,
        report.fireball_render_preview_count,
        report.fireball_render_preview_suppressed_count,
        report.fireball_map_target_probe_count,
        report.fireball_collision_probe_count,
        report.projected_fireball_projectile_collision_snapshot_count,
        report.fireball_enemy_hit_intent_count,
        report.projected_fireball_enemy_hit_snapshot_count,
        report.portal_target_probe_count,
        report.portal_target_projected_player_source_count,
        report.portal_target_possible_count,
        report.portal_open_intent_count,
        report.portal_fizzle_intent_count,
        report.portal_reservation_projection_count,
        report.portal_replacement_summary_count,
        report.projected_portal_state_snapshot_count,
        report.portal_pair_readiness_summary_count,
        report.portal_pair_ready_count,
        report.portal_transit_candidate_probe_count,
        report.portal_transit_candidate_ready_count,
        report.portalcoords_preview_count,
        report.portal_transit_outcome_summary_count,
        report.portal_transit_audio_intent_count,
        report.portal_transit_success_preview_count,
        report.portal_transit_blocked_exit_bounce_preview_count,
        report.portal_transit_projected_player_snapshot_count,
        report.projected_player_state_snapshot_count,
    );
    match report
        .frame_audio_command_detail_summary
        .last_commands
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio commands: total={} last_frame={} last_count={} last_commands={} last_detail={}",
            report.frame_audio_command_count,
            detail.frame_index,
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio commands: total={} last_frame=none last_count=0 last_commands=none last_detail=none",
            report.frame_audio_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_fireball_launch
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio fireball launch: total={} last_frame={} last_launch={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .fireball_launch_command_count,
            detail.frame_index,
            fireball_launch_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio fireball launch: total={} last_frame=none last_launch=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .fireball_launch_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_fireball_collision
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio fireball collision: total={} last_frame={} last_probe={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .fireball_collision_command_count,
            detail.frame_index,
            fireball_collision_probe_label(detail.probe),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio fireball collision: total={} last_frame=none last_probe=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .fireball_collision_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_block_hit
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio block-hit: total={} last_frame={} last_block_hit={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .block_hit_command_count,
            detail.frame_index,
            ceiling_block_hit_label(detail.block_hit),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio block-hit: total={} last_frame=none last_block_hit=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .block_hit_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_reward_reveal
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio reward reveal: total={} last_frame={} last_reveal={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .reward_reveal_command_count,
            detail.frame_index,
            contained_reward_reveal_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio reward reveal: total={} last_frame=none last_reveal=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .reward_reveal_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_coin_block_reward
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio coin-block reward: total={} last_frame={} last_reward={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .coin_block_reward_command_count,
            detail.frame_index,
            coin_block_reward_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio coin-block reward: total={} last_frame=none last_reward=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .coin_block_reward_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_top_coin_collection
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio top-coin collection: total={} last_frame={} last_collection={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .top_coin_collection_command_count,
            detail.frame_index,
            top_coin_collection_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio top-coin collection: total={} last_frame=none last_collection=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .top_coin_collection_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_block_break
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio block-break: total={} last_frame={} last_destroy={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .block_break_command_count,
            detail.frame_index,
            coord_label(detail.intent.coord),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio block-break: total={} last_frame=none last_destroy=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .block_break_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_portal_outcome
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio portal outcome: total={} last_frame={} last_outcome={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .portal_outcome_command_count,
            detail.frame_index,
            portal_outcome_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio portal outcome: total={} last_frame=none last_outcome=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .portal_outcome_command_count,
        ),
    }
    match report
        .frame_audio_command_detail_summary
        .last_portal_transit
        .as_ref()
    {
        Some(detail) => println!(
            "frame audio portal transit: total={} last_frame={} last_intent={} last_count={} last_commands={} last_detail={}",
            report
                .frame_audio_command_detail_summary
                .portal_transit_command_count,
            detail.frame_index,
            portal_transit_audio_intent_label(detail.intent),
            detail.commands.len(),
            audio_command_sequence_label(&detail.commands),
            audio_command_sequence_detail_label(&detail.commands),
        ),
        None => println!(
            "frame audio portal transit: total={} last_frame=none last_intent=none last_count=0 last_commands=none last_detail=none",
            report
                .frame_audio_command_detail_summary
                .portal_transit_command_count,
        ),
    }
    match report.player_coin_pickup_detail_summary.last_pickup {
        Some(detail) => println!(
            "player coin pickup: total={} last_frame={} last_pickup_tile={} last_pickup_tile_id={} last_clear_tile={} last_clear_tile_id={} last_score_delta={} last_sound={} last_detail={}",
            report.coin_pickup_count,
            detail.frame_index,
            coord_label(detail.pickup.coord),
            detail.pickup.tile_id.0,
            coord_label(detail.pickup.coord),
            detail.pickup.clear_tile_id.0,
            detail.pickup.score_delta,
            sound_effect_label(detail.pickup.sound),
            player_coin_pickup_label(detail.pickup),
        ),
        None => println!(
            "player coin pickup: total={} last_frame=none last_pickup_tile=none last_pickup_tile_id=none last_clear_tile=none last_clear_tile_id=none last_score_delta=none last_sound=none last_detail=none",
            report.coin_pickup_count,
        ),
    }
    println!(
        "player tile collisions: horizontal_total={} vertical_total={} ceiling_block_hits={} last_horizontal={} last_vertical={} last_block_hit={}",
        report.horizontal_collision_count,
        report.vertical_collision_count,
        report.ceiling_block_hit_count,
        optional_frame_detail_label(report.tile_collision_detail_summary.last_horizontal.map(
            |detail| {
                (
                    detail.frame_index,
                    player_tile_collision_label(detail.collision),
                )
            }
        ),),
        optional_frame_detail_label(report.tile_collision_detail_summary.last_vertical.map(
            |detail| {
                (
                    detail.frame_index,
                    player_tile_collision_label(detail.collision),
                )
            }
        ),),
        optional_frame_detail_label(report.tile_collision_detail_summary.last_block_hit.map(
            |detail| {
                (
                    detail.frame_index,
                    ceiling_block_hit_label(detail.block_hit),
                )
            }
        ),),
    );
    match report.block_bounce_detail_summary.last_schedule {
        Some(detail) => println!(
            "block bounce schedule: total={} last_frame={} last_block_tile={} last_queue_tile={} last_timer={:.9} last_spawn_content={} last_hitter_size={} last_regenerate_sprite_batch={} last_detail={}",
            report.block_bounce_schedule_count,
            detail.frame_index,
            coord_label(detail.schedule.coord),
            tile_coord_label(detail.schedule.schedule.coord),
            detail.schedule.schedule.timer,
            block_bounce_spawn_content_label(detail.schedule.schedule.spawn_content),
            detail.schedule.schedule.hitter_size,
            bool_label(detail.schedule.schedule.regenerate_sprite_batch),
            block_bounce_schedule_label(detail.schedule),
        ),
        None => println!(
            "block bounce schedule: total={} last_frame=none last_block_tile=none last_queue_tile=none last_timer=none last_spawn_content=none last_hitter_size=none last_regenerate_sprite_batch=none last_detail=none",
            report.block_bounce_schedule_count,
        ),
    }
    match report.block_bounce_detail_summary.last_completion {
        Some(detail) => println!(
            "block bounce progress: total={} prunes={} queue_after_prune={} last_frame={} last_index={} last_tile={} last_timer={:.6} last_remove={} last_suppressed_replay_spawn={} last_completion={}",
            report.block_bounce_progress_count,
            report
                .block_bounce_detail_summary
                .regenerate_sprite_batch_count,
            report.block_bounce_detail_summary.queue_len_after_prune,
            detail.frame_index,
            detail.completion.index,
            tile_coord_label(detail.completion.coord),
            detail.completion.timer,
            bool_label(detail.completion.remove),
            block_bounce_optional_replay_spawn_label(detail.completion.suppressed_replay_spawn),
            block_bounce_completion_label(detail.completion),
        ),
        None => println!(
            "block bounce progress: total={} prunes={} queue_after_prune={} last_frame=none last_index=none last_tile=none last_timer=none last_remove=none last_suppressed_replay_spawn=none last_completion=none",
            report.block_bounce_progress_count,
            report
                .block_bounce_detail_summary
                .regenerate_sprite_batch_count,
            report.block_bounce_detail_summary.queue_len_after_prune,
        ),
    }
    match report.block_bounce_detail_summary.last_item_spawn {
        Some(detail) => println!(
            "block bounce item spawn: total={} last_frame={} last_source_index={} last_source_tile={} last_spawn={} last_detail={}",
            report.block_bounce_item_spawn_intent_count,
            detail.frame_index,
            detail.intent.source_index,
            tile_coord_label(detail.intent.source_coord),
            block_bounce_replay_spawn_label(detail.intent.spawn),
            block_bounce_item_spawn_intent_label(detail.intent),
        ),
        None => println!(
            "block bounce item spawn: total={} last_frame=none last_source_index=none last_source_tile=none last_spawn=none last_detail=none",
            report.block_bounce_item_spawn_intent_count,
        ),
    }
    match report.contained_reward_reveal_detail_summary.last_intent {
        Some(detail) => println!(
            "contained reward reveal: total={} last_frame={} last_block_tile={} last_content={} last_tile={} last_tile_id={} last_sound={} last_detail={}",
            report.contained_reward_reveal_intent_count,
            detail.frame_index,
            coord_label(detail.intent.coord),
            block_bounce_content_label(detail.intent.content),
            tile_coord_label(detail.intent.outcome.tile_change.coord),
            detail.intent.outcome.tile_change.tile.0,
            block_reveal_sound_label(detail.intent.outcome.sound),
            contained_reward_reveal_intent_label(detail.intent),
        ),
        None => println!(
            "contained reward reveal: total={} last_frame=none last_block_tile=none last_content=none last_tile=none last_tile_id=none last_sound=none last_detail=none",
            report.contained_reward_reveal_intent_count,
        ),
    }
    match report.coin_block_reward_detail_summary.last_reward {
        Some(detail) => println!(
            "coin-block reward: total={} last_frame={} last_block_tile={} last_coin_sound={} last_score_delta={} last_coin_count={} last_life_reward={} last_tile_change={} last_many_coins_timer={} last_animation={} last_detail={}",
            report.coin_block_reward_intent_count,
            detail.frame_index,
            coord_label(detail.intent.coord),
            bool_label(detail.intent.outcome.play_coin_sound),
            detail.intent.outcome.score_delta,
            detail.intent.outcome.coin_count,
            optional_life_reward_label(detail.intent.outcome.life_reward),
            optional_tile_change_label(detail.intent.outcome.tile_change),
            optional_many_coins_timer_spawn_label(detail.intent.outcome.start_many_coins_timer),
            coin_block_animation_state_label(detail.intent.outcome.animation),
            coin_block_reward_intent_label(detail.intent),
        ),
        None => println!(
            "coin-block reward: total={} last_frame=none last_block_tile=none last_coin_sound=none last_score_delta=none last_coin_count=none last_life_reward=none last_tile_change=none last_many_coins_timer=none last_animation=none last_detail=none",
            report.coin_block_reward_intent_count,
        ),
    }
    match report
        .coin_block_reward_detail_summary
        .last_top_coin_collection
    {
        Some(detail) => println!(
            "top-coin collection: total={} last_frame={} last_block_tile={} last_coin_tile={} last_coin_sound={} last_score_delta={} last_coin_count={} last_life_reward={} last_tile_change={} last_animation={} last_detail={}",
            report.top_coin_collection_intent_count,
            detail.frame_index,
            coord_label(detail.intent.block_coord),
            coord_label(detail.intent.coin_coord),
            bool_label(detail.intent.outcome.play_coin_sound),
            detail.intent.outcome.score_delta,
            detail.intent.outcome.coin_count,
            optional_life_reward_label(detail.intent.outcome.life_reward),
            tile_change_label(detail.intent.outcome.tile_change),
            coin_block_animation_state_label(detail.intent.outcome.animation),
            top_coin_collection_intent_label(detail.intent),
        ),
        None => println!(
            "top-coin collection: total={} last_frame=none last_block_tile=none last_coin_tile=none last_coin_sound=none last_score_delta=none last_coin_count=none last_life_reward=none last_tile_change=none last_animation=none last_detail=none",
            report.top_coin_collection_intent_count,
        ),
    }
    println!(
        "portalability: queried_tiles={} portalable_tiles={} solid_portalable_tiles={} solid_non_portalable_tiles={}",
        report.portalability.queried_tile_count,
        report.portalability.portalable_tile_count,
        report.portalability.solid_portalable_tile_count,
        report.portalability.solid_non_portalable_tile_count,
    );
    match report.coin_counter_detail_summary.last_intent {
        Some(detail) => println!(
            "coin counter: total={} last_frame={} last_source={} last_coin_before={} last_coin_after={} last_score_delta={}",
            report.coin_counter_intent_count,
            detail.frame_index,
            coin_counter_source_label(detail.intent.source),
            detail.intent.coin_count_before,
            detail.intent.coin_count_after,
            detail.intent.score_delta,
        ),
        None => println!(
            "coin counter: total={} last_frame=none last_source=none last_coin_before=none last_coin_after=none last_score_delta=none",
            report.coin_counter_intent_count,
        ),
    }
    match report.life_reward_counter_detail_summary.last_intent {
        Some(detail) => {
            let life_reward = detail
                .intent
                .life_reward
                .expect("life reward detail should retain the reward payload");
            println!(
                "life reward counter: total={} last_frame={} last_source={} last_grant_lives={} last_respawn_players={} last_play_sound={}",
                report.life_reward_counter_intent_count,
                detail.frame_index,
                coin_counter_source_label(detail.intent.source),
                life_reward.grant_lives_to_players,
                bool_label(life_reward.respawn_players),
                bool_label(life_reward.play_sound),
            );
        }
        None => println!(
            "life reward counter: total={} last_frame=none last_source=none last_grant_lives=none last_respawn_players=none last_play_sound=none",
            report.life_reward_counter_intent_count,
        ),
    }
    match report.score_counter_detail_summary.last_intent {
        Some(detail) => println!(
            "score counter: total={} last_frame={} last_source={} last_score_before={} last_score_delta={} last_score_after={}",
            report.score_counter_intent_count,
            detail.frame_index,
            score_source_label(detail.intent.source),
            detail.intent.score_count_before,
            detail.intent.score_delta,
            detail.intent.score_count_after,
        ),
        None => println!(
            "score counter: total={} last_frame=none last_source=none last_score_before=none last_score_delta=none last_score_after=none",
            report.score_counter_intent_count,
        ),
    }
    match report.scrolling_score_detail_summary.last_intent {
        Some(detail) => {
            let scrolling_score = detail
                .intent
                .scrolling_score
                .expect("scrolling-score detail should retain the scrolling score payload");
            println!(
                "scrolling score: total={} last_frame={} last_source={} last_label={} last_x={:.6} last_y={:.6} last_timer={:.6}",
                report.scrolling_score_intent_count,
                detail.frame_index,
                score_source_label(detail.intent.source),
                scrolling_score_label(scrolling_score.label),
                scrolling_score.x,
                scrolling_score.y,
                scrolling_score.timer,
            );
        }
        None => println!(
            "scrolling score: total={} last_frame=none last_source=none last_label=none last_x=none last_y=none last_timer=none",
            report.scrolling_score_intent_count,
        ),
    }
    match report.effect_animation_detail_summary.last_coin_block {
        Some(detail) => println!(
            "coin-block animation: total={} prunes={} queue_after_prune={} last_frame={} last_index={} last_x={:.6} last_y={:.6} last_timer={:.6} last_animation_frame={} last_remove={} last_score={} last_scrolling_score={} last_detail={} last_prune_detail={}",
            report.coin_block_animation_progress_count,
            report.coin_block_animation_prune_count,
            report
                .effect_animation_detail_summary
                .coin_block_queue_len_after_prune,
            detail.frame_index,
            detail.report.index,
            detail.report.state.x,
            detail.report.state.y,
            detail.report.state.timer,
            detail.report.state.frame,
            bool_label(detail.report.remove),
            coin_block_animation_score_label(detail.report.score),
            optional_scrolling_score_state_label(detail.report.scrolling_score),
            coin_block_animation_update_label(detail.report),
            optional_frame_detail_label(
                report
                    .effect_animation_detail_summary
                    .last_coin_block_prune
                    .map(|detail| {
                        (
                            detail.frame_index,
                            coin_block_animation_update_label(detail.report),
                        )
                    }),
            ),
        ),
        None => println!(
            "coin-block animation: total={} prunes={} queue_after_prune={} last_frame=none last_index=none last_x=none last_y=none last_timer=none last_animation_frame=none last_remove=none last_score=none last_scrolling_score=none last_detail=none last_prune_detail=none",
            report.coin_block_animation_progress_count,
            report.coin_block_animation_prune_count,
            report
                .effect_animation_detail_summary
                .coin_block_queue_len_after_prune,
        ),
    }
    match report.effect_animation_detail_summary.last_block_debris {
        Some(detail) => println!(
            "block-debris animation: total={} prunes={} queue_after_prune={} last_frame={} last_index={} last_source={} last_debris_index={} last_x={:.6} last_y={:.6} last_speed_x={:.6} last_speed_y={:.6} last_timer={:.6} last_animation_frame={} last_remove={} last_detail={} last_prune_detail={}",
            report.block_debris_animation_progress_count,
            report.block_debris_animation_prune_count,
            report
                .effect_animation_detail_summary
                .block_debris_queue_len_after_prune,
            detail.frame_index,
            detail.report.index,
            breakable_cleanup_source_label(detail.report.source),
            detail.report.debris_index,
            detail.report.state.x,
            detail.report.state.y,
            detail.report.state.speed_x,
            detail.report.state.speed_y,
            detail.report.state.timer,
            detail.report.state.frame,
            bool_label(detail.report.remove),
            block_debris_animation_update_label(detail.report),
            optional_frame_detail_label(
                report
                    .effect_animation_detail_summary
                    .last_block_debris_prune
                    .map(|detail| {
                        (
                            detail.frame_index,
                            block_debris_animation_update_label(detail.report),
                        )
                    }),
            ),
        ),
        None => println!(
            "block-debris animation: total={} prunes={} queue_after_prune={} last_frame=none last_index=none last_source=none last_debris_index=none last_x=none last_y=none last_speed_x=none last_speed_y=none last_timer=none last_animation_frame=none last_remove=none last_detail=none last_prune_detail=none",
            report.block_debris_animation_progress_count,
            report.block_debris_animation_prune_count,
            report
                .effect_animation_detail_summary
                .block_debris_queue_len_after_prune,
        ),
    }
    match report.effect_animation_detail_summary.last_scrolling_score {
        Some(detail) => println!(
            "scrolling-score animation: total={} prunes={} queue_after_prune={} last_frame={} last_index={} last_source={} last_label={} last_x={:.6} last_y={:.6} last_timer={:.6} last_presentation_x={:.6} last_presentation_y={:.6} last_remove={} last_detail={} last_prune_detail={}",
            report.scrolling_score_animation_progress_count,
            report.scrolling_score_animation_prune_count,
            report
                .effect_animation_detail_summary
                .scrolling_score_queue_len_after_prune,
            detail.frame_index,
            detail.report.index,
            score_source_label(detail.report.source),
            scrolling_score_label(detail.report.state.label),
            detail.report.state.x,
            detail.report.state.y,
            detail.report.state.timer,
            detail.report.presentation.x,
            detail.report.presentation.y,
            bool_label(detail.report.remove),
            scrolling_score_animation_update_label(detail.report),
            optional_frame_detail_label(
                report
                    .effect_animation_detail_summary
                    .last_scrolling_score_prune
                    .map(|detail| {
                        (
                            detail.frame_index,
                            scrolling_score_animation_update_label(detail.report),
                        )
                    }),
            ),
        ),
        None => println!(
            "scrolling-score animation: total={} prunes={} queue_after_prune={} last_frame=none last_index=none last_source=none last_label=none last_x=none last_y=none last_timer=none last_presentation_x=none last_presentation_y=none last_remove=none last_detail=none last_prune_detail=none",
            report.scrolling_score_animation_progress_count,
            report.scrolling_score_animation_prune_count,
            report
                .effect_animation_detail_summary
                .scrolling_score_queue_len_after_prune,
        ),
    }
    match report
        .effect_animation_detail_summary
        .last_explicit_fireball_collision_scrolling_score
    {
        Some(detail) => println!(
            "scrolling-score animation fireball collision: total={} prunes={} last_frame={} last_index={} last_source={} last_label={} last_remove={} last_detail={} last_prune_detail={}",
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_progress_count,
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_prune_count,
            detail.frame_index,
            detail.report.index,
            score_source_label(detail.report.source),
            scrolling_score_label(detail.report.state.label),
            bool_label(detail.report.remove),
            scrolling_score_animation_update_label(detail.report),
            optional_frame_detail_label(
                report
                    .effect_animation_detail_summary
                    .last_explicit_fireball_collision_scrolling_score_prune
                    .map(|detail| {
                        (
                            detail.frame_index,
                            scrolling_score_animation_update_label(detail.report),
                        )
                    }),
            ),
        ),
        None => println!(
            "scrolling-score animation fireball collision: total={} prunes={} last_frame=none last_index=none last_source=none last_label=none last_remove=none last_detail=none last_prune_detail=none",
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_progress_count,
            report
                .effect_animation_detail_summary
                .explicit_fireball_collision_scrolling_score_prune_count,
        ),
    }
    match report.item_jump_request_detail_summary.last_intent {
        Some(detail) => println!(
            "item jump request: total={} last_frame={} last_block_tile={} last_item_kind={} last_item_index={} last_source_x={:.6}",
            report.item_jump_request_intent_count,
            detail.frame_index,
            coord_label(detail.intent.coord),
            item_kind_label(detail.intent.request.kind),
            detail.intent.request.index,
            detail.intent.request.source_x,
        ),
        None => println!(
            "item jump request: total={} last_frame=none last_block_tile=none last_item_kind=none last_item_index=none last_source_x=none",
            report.item_jump_request_intent_count,
        ),
    }
    match report.enemy_shot_request_detail_summary.last_intent {
        Some(detail) => println!(
            "enemy shot request: total={} last_frame={} last_block_tile={} last_enemy_index={} last_direction={} last_score_delta={} last_score_x={:.6} last_score_y={:.6}",
            report.enemy_shot_request_intent_count,
            detail.frame_index,
            coord_label(detail.intent.coord),
            detail.intent.request.index,
            enemy_direction_label(detail.intent.request.direction),
            detail.intent.request.score_delta,
            detail.intent.request.score_x,
            detail.intent.request.score_y,
        ),
        None => println!(
            "enemy shot request: total={} last_frame=none last_block_tile=none last_enemy_index=none last_direction=none last_score_delta=none last_score_x=none last_score_y=none",
            report.enemy_shot_request_intent_count,
        ),
    }
    match report.many_coins_timer_detail_summary.last_progress {
        Some(detail) => println!(
            "many-coins timer progress: total={} projected_timers={} last_frame={} last_index={} last_tile={} last_remaining_before={:.6} last_remaining_after={:.6}",
            report.many_coins_timer_progress_count,
            report.many_coins_timer_detail_summary.projected_timer_count,
            detail.frame_index,
            detail.report.index,
            tile_coord_label(detail.report.coord),
            detail.report.remaining_before,
            detail.report.remaining_after,
        ),
        None => println!(
            "many-coins timer progress: total={} projected_timers={} last_frame=none last_index=none last_tile=none last_remaining_before=none last_remaining_after=none",
            report.many_coins_timer_progress_count,
            report.many_coins_timer_detail_summary.projected_timer_count,
        ),
    }
    match report.many_coins_timer_detail_summary.last_start {
        Some(detail) => println!(
            "many-coins timer start: total={} last_frame={} last_reward_index={} last_tile={} last_duration={:.6}",
            report.many_coins_timer_start_count,
            detail.frame_index,
            detail.report.reward_index,
            tile_coord_label(detail.report.coord),
            detail.report.duration,
        ),
        None => println!(
            "many-coins timer start: total={} last_frame=none last_reward_index=none last_tile=none last_duration=none",
            report.many_coins_timer_start_count,
        ),
    }
    match report.tile_change_projection_detail_summary.last_projection {
        Some(detail) => println!(
            "tile change projection: total={} projected_snapshot={} last_frame={} last_source={} last_tile={} last_tile_id={} last_frame_changes={} projected_snapshot_changes={}",
            report
                .tile_change_projection_detail_summary
                .projection_count,
            report
                .tile_change_projection_detail_summary
                .projected_snapshot_count,
            detail.frame_index,
            tile_change_source_label(detail.projection.source),
            tile_coord_label(detail.projection.tile_change.coord),
            detail.projection.tile_change.tile.0,
            tile_change_projections_label(
                &report
                    .tile_change_projection_detail_summary
                    .last_frame_projections,
            ),
            tile_change_projections_label(
                &report
                    .tile_change_projection_detail_summary
                    .projected_snapshot,
            ),
        ),
        None => println!(
            "tile change projection: total={} projected_snapshot={} last_frame=none last_source=none last_tile=none last_tile_id=none last_frame_changes={} projected_snapshot_changes={}",
            report
                .tile_change_projection_detail_summary
                .projection_count,
            report
                .tile_change_projection_detail_summary
                .projected_snapshot_count,
            tile_change_projections_label(
                &report
                    .tile_change_projection_detail_summary
                    .last_frame_projections,
            ),
            tile_change_projections_label(
                &report
                    .tile_change_projection_detail_summary
                    .projected_snapshot,
            ),
        ),
    }
    match report
        .breakable_block_cleanup_projection_detail_summary
        .last_projection
    {
        Some(detail) => println!(
            "breakable cleanup projection: total={} last_frame={} last_source={} last_action={} last_frame_actions={}",
            report
                .breakable_block_cleanup_projection_detail_summary
                .projection_count,
            detail.frame_index,
            breakable_cleanup_source_label(detail.projection.source),
            breakable_cleanup_action_label(detail.projection.action),
            breakable_cleanup_projections_label(
                &report
                    .breakable_block_cleanup_projection_detail_summary
                    .last_frame_projections,
            ),
        ),
        None => println!(
            "breakable cleanup projection: total={} last_frame=none last_source=none last_action=none last_frame_actions={}",
            report
                .breakable_block_cleanup_projection_detail_summary
                .projection_count,
            breakable_cleanup_projections_label(
                &report
                    .breakable_block_cleanup_projection_detail_summary
                    .last_frame_projections,
            ),
        ),
    }
    match report.block_hit_portal_guard_summary.last_guard {
        Some(detail) => println!(
            "block-hit portal guard: total={} projected_portal_state={} explicit_reservations={} last_frame={} last_tile={} last_tile_id={} last_breakable={} last_coin_block={} last_play_hit_sound={} last_source={} last_reservation_tile={} last_reservation_facing={}",
            report.block_hit_portal_guard_suppression_count,
            report
                .block_hit_portal_guard_summary
                .projected_portal_state_count,
            report
                .block_hit_portal_guard_summary
                .explicit_reservation_count,
            detail.frame_index,
            coord_label(detail.coord),
            detail.tile_id.0,
            bool_label(detail.breakable),
            bool_label(detail.coin_block),
            bool_label(detail.play_hit_sound),
            portal_block_guard_source_label(detail.guard.source),
            tile_coord_label(detail.guard.reservation.coord),
            facing_label(detail.guard.reservation.facing),
        ),
        None => println!(
            "block-hit portal guard: total={} projected_portal_state={} explicit_reservations={} last_frame=none last_tile=none last_tile_id=none last_breakable=none last_coin_block=none last_play_hit_sound=none last_source=none last_reservation_tile=none last_reservation_facing=none",
            report.block_hit_portal_guard_suppression_count,
            report
                .block_hit_portal_guard_summary
                .projected_portal_state_count,
            report
                .block_hit_portal_guard_summary
                .explicit_reservation_count,
        ),
    }
    match report.portal_target_source_selection.last_selection {
        Some(selection) => println!(
            "portal target source selection: live_player_sources={} projected_portal_transit_sources={} last_frame={} last_source={} last_source_x={:.6} last_source_y={:.6} last_requested_slot={} last_aim={:.6}",
            report.portal_target_source_selection.live_player_count,
            report
                .portal_target_source_selection
                .projected_portal_transit_count,
            selection.frame_index,
            portal_target_player_source_label(selection.player_source),
            selection.source_x,
            selection.source_y,
            portal_slot_label(selection.requested_slot),
            selection.pointing_angle,
        ),
        None => println!(
            "portal target source selection: live_player_sources={} projected_portal_transit_sources={} last_frame=none last_source=none last_source_x=none last_source_y=none last_requested_slot=none last_aim=none",
            report.portal_target_source_selection.live_player_count,
            report
                .portal_target_source_selection
                .projected_portal_transit_count,
        ),
    }
    match report.portal_target_placement_summary.last_summary {
        Some(summary) => println!(
            "portal target placement: possible={} impossible={} last_frame={} last_requested_slot={} last_hit_tile={} last_hit_side={} last_hit_tendency={} last_placement_tile={} last_placement_side={}",
            report.portal_target_placement_summary.possible_count,
            report.portal_target_placement_summary.impossible_count,
            summary.frame_index,
            portal_slot_label(summary.requested_slot),
            portal_trace_hit_coord_label(summary.trace_hit),
            portal_trace_hit_side_label(summary.trace_hit),
            portal_trace_hit_tendency_label(summary.trace_hit),
            portal_placement_coord_label(summary.placement),
            portal_placement_side_label(summary.placement),
        ),
        None => println!(
            "portal target placement: possible={} impossible={} last_frame=none last_requested_slot=none last_hit_tile=none last_hit_side=none last_hit_tendency=none last_placement_tile=none last_placement_side=none",
            report.portal_target_placement_summary.possible_count,
            report.portal_target_placement_summary.impossible_count,
        ),
    }
    match report.portal_open_outcome_summary.last_summary {
        Some(summary) => println!(
            "portal open outcome: opens={} fizzles={} last_frame={} last_requested_slot={} last_kind={} last_placement_tile={} last_placement_side={} last_sound={}",
            report.portal_open_outcome_summary.open_count,
            report.portal_open_outcome_summary.fizzle_count,
            summary.frame_index,
            portal_slot_label(Some(summary.requested_slot)),
            portal_outcome_kind_label(summary.kind),
            portal_placement_coord_label(summary.placement),
            portal_placement_side_label(summary.placement),
            summary.sound.lua_global(),
        ),
        None => println!(
            "portal open outcome: opens={} fizzles={} last_frame=none last_requested_slot=none last_kind=none last_placement_tile=none last_placement_side=none last_sound=none",
            report.portal_open_outcome_summary.open_count,
            report.portal_open_outcome_summary.fizzle_count,
        ),
    }
    match report.portal_reservation_projection_summary.last_projection {
        Some(detail) => println!(
            "portal reservation projection: total={} last_frame={} last_requested_slot={} last_placement_tile={} last_placement_side={} last_tile_reservations={} last_wall_reservations={}",
            report
                .portal_reservation_projection_summary
                .projection_count,
            detail.frame_index,
            portal_slot_label(Some(detail.projection.requested_slot)),
            portal_placement_coord_label(Some(detail.projection.placement)),
            portal_placement_side_label(Some(detail.projection.placement)),
            tile_reservations_label(detail.projection.tile_reservations),
            wall_reservations_label(detail.projection.wall_reservations),
        ),
        None => println!(
            "portal reservation projection: total={} last_frame=none last_requested_slot=none last_placement_tile=none last_placement_side=none last_tile_reservations=none last_wall_reservations=none",
            report
                .portal_reservation_projection_summary
                .projection_count,
        ),
    }
    match report.portal_replacement_detail_summary.last_replacement {
        Some(detail) => println!(
            "portal replacement summary: total={} last_frame={} last_requested_slot={} last_previous_slot={} last_previous_tile={} last_previous_side={} last_replacement_slot={} last_replacement_tile={} last_replacement_side={} last_preserved_slot={} last_preserved_tile={} last_preserved_side={}",
            report.portal_replacement_detail_summary.replacement_count,
            detail.frame_index,
            portal_slot_label(Some(detail.summary.requested_slot)),
            projected_portal_slot_label(detail.summary.previous_slot),
            projected_portal_coord_label(detail.summary.previous_slot),
            projected_portal_side_label(detail.summary.previous_slot),
            portal_slot_label(Some(detail.summary.replacement_slot.requested_slot)),
            portal_placement_coord_label(Some(detail.summary.replacement_slot.placement)),
            portal_placement_side_label(Some(detail.summary.replacement_slot.placement)),
            projected_portal_slot_label(detail.summary.preserved_other_slot),
            projected_portal_coord_label(detail.summary.preserved_other_slot),
            projected_portal_side_label(detail.summary.preserved_other_slot),
        ),
        None => println!(
            "portal replacement summary: total={} last_frame=none last_requested_slot=none last_previous_slot=none last_previous_tile=none last_previous_side=none last_replacement_slot=none last_replacement_tile=none last_replacement_side=none last_preserved_slot=none last_preserved_tile=none last_preserved_side=none",
            report.portal_replacement_detail_summary.replacement_count,
        ),
    }
    match report.portal_pair_readiness_detail_summary.last_summary {
        Some(detail) => println!(
            "portal pair readiness: total={} ready={} last_frame={} last_ready={} last_portal1_slot={} last_portal1_tile={} last_portal1_side={} last_portal2_slot={} last_portal2_tile={} last_portal2_side={} last_portal1_to_2={} last_portal2_to_1={}",
            report.portal_pair_readiness_summary_count,
            report.portal_pair_ready_count,
            detail.frame_index,
            bool_label(detail.summary.ready),
            projected_portal_slot_label(detail.summary.portal_1),
            projected_portal_coord_label(detail.summary.portal_1),
            projected_portal_side_label(detail.summary.portal_1),
            projected_portal_slot_label(detail.summary.portal_2),
            projected_portal_coord_label(detail.summary.portal_2),
            projected_portal_side_label(detail.summary.portal_2),
            portal_pairing_label(detail.summary.portal_1_to_2),
            portal_pairing_label(detail.summary.portal_2_to_1),
        ),
        None => println!(
            "portal pair readiness: total={} ready={} last_frame=none last_ready=none last_portal1_slot=none last_portal1_tile=none last_portal1_side=none last_portal2_slot=none last_portal2_tile=none last_portal2_side=none last_portal1_to_2=none last_portal2_to_1=none",
            report.portal_pair_readiness_summary_count, report.portal_pair_ready_count,
        ),
    }
    match report.portal_transit_candidate_detail_summary.last_probe {
        Some(detail) => println!(
            "portal transit candidate: total={} matched={} last_frame={} last_center_x={:.6} last_center_y={:.6} last_center_tile={} last_entry_tile={} last_entry_slot={} last_exit_slot={}",
            report.portal_transit_candidate_probe_count,
            report.portal_transit_candidate_ready_count,
            detail.frame_index,
            detail.probe.center_x,
            detail.probe.center_y,
            coord_label(detail.probe.center_coord),
            optional_coord_label(detail.probe.candidate_entry_tile),
            portal_pairing_entry_slot_label(detail.probe.candidate_pairing),
            portal_pairing_exit_slot_label(detail.probe.candidate_pairing),
        ),
        None => println!(
            "portal transit candidate: total={} matched={} last_frame=none last_center_x=none last_center_y=none last_center_tile=none last_entry_tile=none last_entry_slot=none last_exit_slot=none",
            report.portal_transit_candidate_probe_count,
            report.portal_transit_candidate_ready_count,
        ),
    }
    match report.portalcoords_preview_detail_summary.last_preview {
        Some(detail) => println!(
            "portalcoords preview: total={} last_frame={} last_entry_slot={} last_exit_slot={} last_entry_facing={} last_exit_facing={} last_input_x={:.6} last_input_y={:.6} last_input_speed_x={:.6} last_input_speed_y={:.6} last_input_rotation={:.6} last_output_x={:.6} last_output_y={:.6} last_output_speed_x={:.6} last_output_speed_y={:.6} last_output_rotation={:.6} last_output_animation_direction={} last_exit_blocked={} last_blocked_exit={}",
            report.portalcoords_preview_count,
            detail.frame_index,
            portal_slot_label(Some(detail.preview.entry_slot)),
            portal_slot_label(Some(detail.preview.exit_slot)),
            facing_label(detail.preview.entry_facing),
            facing_label(detail.preview.exit_facing),
            detail.preview.input_body.x,
            detail.preview.input_body.y,
            detail.preview.input_speed_x,
            detail.preview.input_speed_y,
            detail.preview.input_rotation,
            detail.preview.output_body.x,
            detail.preview.output_body.y,
            detail.preview.output_speed_x,
            detail.preview.output_speed_y,
            detail.preview.output_rotation,
            horizontal_direction_label(detail.preview.output_animation_direction),
            bool_label(detail.preview.exit_blocked),
            blocked_exit_probe_label(detail.preview.blocked_exit_probe),
        ),
        None => println!(
            "portalcoords preview: total={} last_frame=none last_entry_slot=none last_exit_slot=none last_entry_facing=none last_exit_facing=none last_input_x=none last_input_y=none last_input_speed_x=none last_input_speed_y=none last_input_rotation=none last_output_x=none last_output_y=none last_output_speed_x=none last_output_speed_y=none last_output_rotation=none last_output_animation_direction=none last_exit_blocked=none last_blocked_exit=none",
            report.portalcoords_preview_count,
        ),
    }
    match report.portal_transit_outcome_detail_summary.last_summary {
        Some(detail) => println!(
            "portal transit outcome: total={} teleport_previews={} blocked_exit_bounce_previews={} last_frame={} last_kind={} last_entry_slot={} last_exit_slot={} last_entry_facing={} last_exit_facing={} last_input_x={:.6} last_input_y={:.6} last_output_x={:.6} last_output_y={:.6} last_output_speed_x={:.6} last_output_speed_y={:.6} last_blocked_exit={}",
            report.portal_transit_outcome_summary_count,
            report.portal_transit_success_preview_count,
            report.portal_transit_blocked_exit_bounce_preview_count,
            detail.frame_index,
            portal_transit_outcome_kind_label(detail.summary.kind),
            portal_slot_label(Some(detail.summary.entry_slot)),
            portal_slot_label(Some(detail.summary.exit_slot)),
            facing_label(detail.summary.entry_facing),
            facing_label(detail.summary.exit_facing),
            detail.summary.input_body.x,
            detail.summary.input_body.y,
            detail.summary.output_body.x,
            detail.summary.output_body.y,
            detail.summary.output_speed_x,
            detail.summary.output_speed_y,
            blocked_exit_probe_label(detail.summary.blocked_exit_probe),
        ),
        None => println!(
            "portal transit outcome: total={} teleport_previews={} blocked_exit_bounce_previews={} last_frame=none last_kind=none last_entry_slot=none last_exit_slot=none last_entry_facing=none last_exit_facing=none last_input_x=none last_input_y=none last_output_x=none last_output_y=none last_output_speed_x=none last_output_speed_y=none last_blocked_exit=none",
            report.portal_transit_outcome_summary_count,
            report.portal_transit_success_preview_count,
            report.portal_transit_blocked_exit_bounce_preview_count,
        ),
    }
    match report.portal_transit_audio_detail_summary.last_intent {
        Some(detail) => println!(
            "portal enter audio: total={} last_frame={} last_outcome_kind={} last_entry_slot={} last_exit_slot={} last_sound={}",
            report.portal_transit_audio_intent_count,
            detail.frame_index,
            portal_transit_outcome_kind_label(detail.intent.outcome_kind),
            portal_slot_label(Some(detail.intent.entry_slot)),
            portal_slot_label(Some(detail.intent.exit_slot)),
            detail.intent.sound.lua_global(),
        ),
        None => println!(
            "portal enter audio: total={} last_frame=none last_outcome_kind=none last_entry_slot=none last_exit_slot=none last_sound=none",
            report.portal_transit_audio_intent_count,
        ),
    }
    match report.projected_player_state_detail_summary.last_snapshot {
        Some(detail) => println!(
            "projected player state: total={} last_frame={} last_source={} last_entry_slot={} last_exit_slot={} last_entry_facing={} last_exit_facing={} last_x={:.6} last_y={:.6} last_speed_x={:.6} last_speed_y={:.6} last_animation_direction={}",
            report.portal_transit_projected_player_snapshot_count,
            detail.frame_index,
            projected_player_state_source_label(detail.snapshot.source),
            portal_slot_label(Some(detail.snapshot.entry_slot)),
            portal_slot_label(Some(detail.snapshot.exit_slot)),
            facing_label(detail.snapshot.entry_facing),
            facing_label(detail.snapshot.exit_facing),
            detail.snapshot.body.x,
            detail.snapshot.body.y,
            detail.snapshot.speed_x,
            detail.snapshot.speed_y,
            horizontal_direction_label(detail.snapshot.animation_direction),
        ),
        None => println!(
            "projected player state: total={} last_frame=none last_source=none last_entry_slot=none last_exit_slot=none last_entry_facing=none last_exit_facing=none last_x=none last_y=none last_speed_x=none last_speed_y=none last_animation_direction=none",
            report.portal_transit_projected_player_snapshot_count,
        ),
    }
    println!("known parity gaps:");
    for gap in report.parity_gaps {
        println!("- {gap}");
    }

    Ok(())
}

fn portal_trace_hit_coord_label(hit: Option<LegacyRuntimePortalTraceHit>) -> String {
    match hit {
        Some(hit) => coord_label(hit.coord),
        None => "none".to_owned(),
    }
}

fn coin_counter_source_label(source: LegacyRuntimeCoinCounterSource) -> String {
    match source {
        LegacyRuntimeCoinCounterSource::PlayerCoinPickup { coord } => {
            format!("player_coin_pickup:{}", coord_label(coord))
        }
        LegacyRuntimeCoinCounterSource::CoinBlockReward { coord } => {
            format!("coin_block_reward:{}", coord_label(coord))
        }
        LegacyRuntimeCoinCounterSource::TopCoinCollection {
            block_coord,
            coin_coord,
        } => format!(
            "top_coin_collection:block={};coin={}",
            coord_label(block_coord),
            coord_label(coin_coord)
        ),
    }
}

fn score_source_label(source: LegacyRuntimeScoreSource) -> String {
    match source {
        LegacyRuntimeScoreSource::PlayerCoinPickup { coord } => {
            format!("player_coin_pickup:{}", coord_label(coord))
        }
        LegacyRuntimeScoreSource::CoinBlockReward { coord } => {
            format!("coin_block_reward:{}", coord_label(coord))
        }
        LegacyRuntimeScoreSource::TopCoinCollection {
            block_coord,
            coin_coord,
        } => format!(
            "top_coin_collection:block={};coin={}",
            coord_label(block_coord),
            coord_label(coin_coord)
        ),
        LegacyRuntimeScoreSource::EnemyShotRequest {
            block_coord,
            enemy_index,
        } => format!(
            "enemy_shot_request:block={};enemy_index={}",
            coord_label(block_coord),
            enemy_index
        ),
        LegacyRuntimeScoreSource::EmptyBreakableBlockDestroy { coord } => {
            format!("empty_breakable_block_destroy:{}", coord_label(coord))
        }
        LegacyRuntimeScoreSource::CoinBlockAnimation { source_index } => {
            format!("coin_block_animation:source_index={source_index}")
        }
        LegacyRuntimeScoreSource::FireballCollisionProbe {
            projectile_index,
            source,
            axis,
            target,
        } => format!(
            "fireball_collision_probe:index={};source={};axis={};target={}",
            projectile_index,
            fireball_collision_probe_source_label(source),
            fireball_collision_axis_label(axis),
            fireball_collision_target_label(target),
        ),
    }
}

fn coin_block_reward_intent_label(intent: LegacyRuntimeCoinBlockRewardIntent) -> String {
    format!(
        "block={};coin_sound={};animation={};score_delta={};coin_count={};life_reward={};tile_change={};many_coins_timer={}",
        coord_label(intent.coord),
        bool_label(intent.outcome.play_coin_sound),
        coin_block_animation_state_label(intent.outcome.animation),
        intent.outcome.score_delta,
        intent.outcome.coin_count,
        optional_life_reward_label(intent.outcome.life_reward),
        optional_tile_change_label(intent.outcome.tile_change),
        optional_many_coins_timer_spawn_label(intent.outcome.start_many_coins_timer),
    )
}

fn top_coin_collection_intent_label(intent: LegacyRuntimeBlockTopCoinCollectionIntent) -> String {
    format!(
        "block={};coin={};coin_sound={};animation={};score_delta={};coin_count={};life_reward={};tile_change={}",
        coord_label(intent.block_coord),
        coord_label(intent.coin_coord),
        bool_label(intent.outcome.play_coin_sound),
        coin_block_animation_state_label(intent.outcome.animation),
        intent.outcome.score_delta,
        intent.outcome.coin_count,
        optional_life_reward_label(intent.outcome.life_reward),
        tile_change_label(intent.outcome.tile_change),
    )
}

fn coin_block_animation_state_label(state: LegacyCoinBlockAnimationState) -> String {
    format!(
        "x={:.6};y={:.6};timer={:.6};frame={}",
        state.x, state.y, state.timer, state.frame,
    )
}

fn optional_life_reward_label(life_reward: Option<LegacyCoinLifeReward>) -> String {
    match life_reward {
        Some(life_reward) => format!(
            "grant_lives={};respawn_players={};play_sound={}",
            life_reward.grant_lives_to_players,
            bool_label(life_reward.respawn_players),
            bool_label(life_reward.play_sound),
        ),
        None => "none".to_owned(),
    }
}

fn optional_u32_label(value: Option<u32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn tile_change_label(tile_change: LegacyTileChange) -> String {
    format!(
        "{}:{}",
        tile_coord_label(tile_change.coord),
        tile_change.tile.0,
    )
}

fn optional_tile_change_label(tile_change: Option<LegacyTileChange>) -> String {
    match tile_change {
        Some(tile_change) => tile_change_label(tile_change),
        None => "none".to_owned(),
    }
}

fn optional_many_coins_timer_spawn_label(spawn: Option<LegacyCoinBlockTimerSpawn>) -> String {
    match spawn {
        Some(spawn) => format!(
            "coord={};duration={:.6}",
            tile_coord_label(spawn.coord),
            spawn.duration,
        ),
        None => "none".to_owned(),
    }
}

fn tile_change_projections_label(projections: &[LegacyRuntimeTileChangeProjection]) -> String {
    if projections.is_empty() {
        return "[]".to_owned();
    }

    format!(
        "[{}]",
        projections
            .iter()
            .copied()
            .map(tile_change_projection_label)
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn tile_change_projection_label(projection: LegacyRuntimeTileChangeProjection) -> String {
    format!(
        "{}->{}:{}",
        tile_change_source_label(projection.source),
        tile_coord_label(projection.tile_change.coord),
        projection.tile_change.tile.0,
    )
}

fn tile_change_source_label(source: LegacyRuntimeTileChangeSource) -> String {
    match source {
        LegacyRuntimeTileChangeSource::ContainedRewardReveal { coord, content } => format!(
            "contained_reward_reveal:coord={};content={}",
            coord_label(coord),
            block_bounce_content_label(content),
        ),
        LegacyRuntimeTileChangeSource::CoinBlockReward { coord } => {
            format!("coin_block_reward:{}", coord_label(coord))
        }
        LegacyRuntimeTileChangeSource::TopCoinCollection {
            block_coord,
            coin_coord,
        } => format!(
            "top_coin_collection:block={};coin={}",
            coord_label(block_coord),
            coord_label(coin_coord)
        ),
        LegacyRuntimeTileChangeSource::EmptyBreakableBlockDestroy { coord } => {
            format!("empty_breakable_block_destroy:{}", coord_label(coord))
        }
    }
}

fn fireball_launch_intent_label(intent: LegacyRuntimeFireballLaunchIntent) -> String {
    format!(
        "source=({:.6},{:.6});direction={};spawn=({:.6},{:.6});speed=({:.6},{:.6});size=({:.6},{:.6});active={};count_before={};count_after={};fire_animation_timer={:.6};sound={}",
        intent.source_x,
        intent.source_y,
        enemy_direction_label(intent.direction),
        intent.spawn.x,
        intent.spawn.y,
        intent.spawn.speed_x,
        intent.spawn.speed_y,
        intent.spawn.width,
        intent.spawn.height,
        intent.spawn.active,
        intent.fireball_count_before,
        intent.fireball_count_after,
        intent.fire_animation_timer_reset,
        sound_effect_label(intent.sound),
    )
}

fn fireball_frame_label(frame: LegacyFireballFrame) -> &'static str {
    match frame {
        LegacyFireballFrame::FlyingOne => "flying_1",
        LegacyFireballFrame::FlyingTwo => "flying_2",
        LegacyFireballFrame::FlyingThree => "flying_3",
        LegacyFireballFrame::FlyingFour => "flying_4",
        LegacyFireballFrame::ExplosionOne => "explosion_1",
        LegacyFireballFrame::ExplosionTwo => "explosion_2",
        LegacyFireballFrame::ExplosionThree => "explosion_3",
    }
}

fn fireball_collision_probe_label(probe: LegacyRuntimeFireballCollisionProbe) -> String {
    format!(
        "index={};source={};axis={};target={};before=({:.6},{:.6});after=({:.6},{:.6});speed=({:.6},{:.6});active={};destroy_soon={};suppress_default={};released_thrower={};block_hit_sound={};shoot_target={};points={}",
        probe.projectile_index,
        fireball_collision_probe_source_label(probe.source),
        fireball_collision_axis_label(probe.axis),
        fireball_collision_target_label(probe.target),
        probe.state_before.x,
        probe.state_before.y,
        probe.state_after.x,
        probe.state_after.y,
        probe.state_after.speed_x,
        probe.state_after.speed_y,
        probe.state_after.active,
        probe.state_after.destroy_soon,
        probe.outcome.suppress_default,
        probe.outcome.released_thrower,
        probe.outcome.play_block_hit_sound,
        probe
            .outcome
            .shoot_target
            .map(enemy_direction_label)
            .unwrap_or("none"),
        probe
            .outcome
            .points
            .map_or_else(|| "none".to_owned(), |points| points.to_string()),
    )
}

fn fireball_collision_probe_source_label(
    source: LegacyRuntimeFireballCollisionProbeSource,
) -> String {
    match source {
        LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest => "explicit_request".to_owned(),
        LegacyRuntimeFireballCollisionProbeSource::MapTargetProbe { coord, tile_id } => {
            format!(
                "map_target_probe:coord={};tile={}",
                coord_label(coord),
                tile_id.0
            )
        }
        LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { enemy_index } => {
            format!("enemy_overlap_probe:enemy_index={enemy_index}")
        }
    }
}

fn fireball_enemy_hit_intent_label(intent: LegacyRuntimeFireballEnemyHitIntent) -> String {
    format!(
        "index={};source={};axis={};target={};enemy_index={};enemy_target={};enemy=({:.6},{:.6},{:.6},{:.6});shot_direction={};score_delta={};score=({:.6},{:.6});live_enemy_mutated={}",
        intent.projectile_index,
        fireball_collision_probe_source_label(intent.source),
        fireball_collision_axis_label(intent.axis),
        fireball_collision_target_label(intent.target),
        intent.enemy.index,
        fireball_collision_target_label(intent.enemy.target),
        intent.enemy.x,
        intent.enemy.y,
        intent.enemy.width,
        intent.enemy.height,
        enemy_direction_label(intent.shot_direction),
        optional_u32_label(intent.score_delta),
        intent.score_x,
        intent.score_y,
        intent.live_enemy_mutated,
    )
}

fn projected_fireball_enemy_hit_snapshot_label(
    snapshot: LegacyRuntimeProjectedFireballEnemyHitSnapshot,
) -> String {
    format!(
        "index={};target={};enemy_index={};active_after={};shot_after={};removed_from_future_queries={};live_enemy_mutated={}",
        snapshot.intent.projectile_index,
        fireball_collision_target_label(snapshot.intent.target),
        snapshot.enemy.index,
        snapshot.active_after,
        snapshot.shot_after,
        snapshot.removed_from_future_queries,
        snapshot.live_enemy_mutated,
    )
}

fn projected_fireball_projectile_collision_snapshot_label(
    snapshot: LegacyRuntimeProjectedFireballProjectileCollisionSnapshot,
) -> String {
    format!(
        "index={};source={};axis={};target={};before_active={};after_active={};after_destroy_soon={};after_speed=({:.6},{:.6});removed_from_future_queries={};live_projectile_queue_mutated={}",
        snapshot.projectile_index,
        fireball_collision_probe_source_label(snapshot.source),
        fireball_collision_axis_label(snapshot.axis),
        fireball_collision_target_label(snapshot.target),
        snapshot.state_before.active,
        snapshot.state_after.active,
        snapshot.state_after.destroy_soon,
        snapshot.state_after.speed_x,
        snapshot.state_after.speed_y,
        snapshot.removed_from_future_collision_queries,
        snapshot.live_projectile_queue_mutated,
    )
}

fn fireball_render_source_label(source: LegacyRuntimeFireballRenderSource) -> &'static str {
    match source {
        LegacyRuntimeFireballRenderSource::LiveProjectile => "live_projectile",
        LegacyRuntimeFireballRenderSource::ProjectedProjectileCollision => {
            "projected_projectile_collision"
        }
    }
}

fn fireball_render_frame_kind_label(kind: LegacyRuntimeFireballRenderFrameKind) -> &'static str {
    match kind {
        LegacyRuntimeFireballRenderFrameKind::Flying => "flying",
        LegacyRuntimeFireballRenderFrameKind::Explosion => "explosion",
    }
}

fn fireball_render_quad_label(preview: LegacyRuntimeFireballRenderIntentPreview) -> String {
    format!(
        "{},{},{},{}",
        preview.quad.x_px, preview.quad.y_px, preview.quad.width_px, preview.quad.height_px,
    )
}

fn fireball_render_preview_label(preview: LegacyRuntimeFireballRenderIntentPreview) -> String {
    format!(
        "index={};source={};kind={};frame={};image={};quad={};draw=({:.6},{:.6});rotation={:.6};scale={:.6};live_rendering_executed={};live_projectile_queue_mutated={}",
        preview.projectile_index,
        fireball_render_source_label(preview.source),
        fireball_render_frame_kind_label(preview.frame_kind),
        fireball_frame_label(preview.frame),
        preview.image_path,
        fireball_render_quad_label(preview),
        preview.draw_x_px,
        preview.draw_y_px,
        preview.rotation,
        preview.scale,
        preview.live_rendering_executed,
        preview.live_projectile_queue_mutated,
    )
}

fn player_power_up_label(power_up: LegacyRuntimePlayerPowerUp) -> &'static str {
    match power_up {
        LegacyRuntimePlayerPowerUp::Small => "small",
        LegacyRuntimePlayerPowerUp::Big => "big",
        LegacyRuntimePlayerPowerUp::Fire => "fire",
    }
}

fn player_render_frame_label(frame: LegacyRuntimePlayerRenderFrame) -> &'static str {
    match frame {
        LegacyRuntimePlayerRenderFrame::SmallRun => "small_run",
        LegacyRuntimePlayerRenderFrame::SmallIdle => "small_idle",
        LegacyRuntimePlayerRenderFrame::SmallSlide => "small_slide",
        LegacyRuntimePlayerRenderFrame::SmallJump => "small_jump",
        LegacyRuntimePlayerRenderFrame::SmallSwim => "small_swim",
        LegacyRuntimePlayerRenderFrame::SmallClimb => "small_climb",
        LegacyRuntimePlayerRenderFrame::SmallDead => "small_dead",
        LegacyRuntimePlayerRenderFrame::BigRun => "big_run",
        LegacyRuntimePlayerRenderFrame::BigIdle => "big_idle",
        LegacyRuntimePlayerRenderFrame::BigSlide => "big_slide",
        LegacyRuntimePlayerRenderFrame::BigJump => "big_jump",
        LegacyRuntimePlayerRenderFrame::BigSwim => "big_swim",
        LegacyRuntimePlayerRenderFrame::BigClimb => "big_climb",
        LegacyRuntimePlayerRenderFrame::BigDuck => "big_duck",
        LegacyRuntimePlayerRenderFrame::BigFire => "big_fire",
    }
}

fn player_render_quad_label(quad: LegacyRuntimePlayerRenderQuad) -> String {
    format!(
        "{},{},{},{}@{}x{}",
        quad.x_px,
        quad.y_px,
        quad.width_px,
        quad.height_px,
        quad.atlas_width_px,
        quad.atlas_height_px,
    )
}

fn player_render_tint_source_label(source: LegacyRuntimePlayerRenderTintSource) -> &'static str {
    match source {
        LegacyRuntimePlayerRenderTintSource::PlayerColor => "player_color",
        LegacyRuntimePlayerRenderTintSource::FlowerColor => "flower_color",
        LegacyRuntimePlayerRenderTintSource::White => "white",
    }
}

fn player_render_color_layer_label(layer: LegacyRuntimePlayerRenderColorLayerPreview) -> String {
    format!(
        "order={},layer={},image={},tint={:.6}/{:.6}/{:.6},tint_source={},quad={},draw=({:.6},{:.6}),rotation={:.6},scale={:.6},live_rendering_executed={}",
        layer.draw_order,
        layer.graphic_layer_index,
        layer.image_path,
        layer.tint.r,
        layer.tint.g,
        layer.tint.b,
        player_render_tint_source_label(layer.tint_source),
        player_render_quad_label(layer.quad),
        layer.draw_x_px,
        layer.draw_y_px,
        layer.rotation,
        layer.scale,
        layer.live_rendering_executed,
    )
}

fn player_render_color_layers_label(
    layers: &[LegacyRuntimePlayerRenderColorLayerPreview; 4],
) -> String {
    layers
        .iter()
        .map(|layer| player_render_color_layer_label(*layer))
        .collect::<Vec<_>>()
        .join("|")
}

fn player_render_hat_size_label(size: LegacyRuntimePlayerRenderHatSize) -> &'static str {
    match size {
        LegacyRuntimePlayerRenderHatSize::Small => "small",
        LegacyRuntimePlayerRenderHatSize::Big => "big",
    }
}

fn player_render_hat_draw_label(hat: LegacyRuntimePlayerRenderHatPreview) -> String {
    format!(
        "order={},slot={},hat={},size={},image={},tint={:.6}/{:.6}/{:.6},tint_source={},config=({},{},{}),offset=({},{}),stack_y={},after_layer={},before_layer={},draw=({:.6},{:.6}),origin=({},{}),rotation={:.6},direction_scale={:.6},vertical_scale={:.6},live_rendering_executed={}",
        hat.draw_order,
        hat.hat_slot_index,
        hat.hat_id,
        player_render_hat_size_label(hat.size),
        hat.image_path,
        hat.tint.r,
        hat.tint.g,
        hat.tint.b,
        player_render_tint_source_label(hat.tint_source),
        hat.hat_config_x_px,
        hat.hat_config_y_px,
        hat.hat_height_px,
        hat.offset_x_px,
        hat.offset_y_px,
        hat.stack_y_px,
        hat.follows_graphic_layer_index,
        hat.precedes_graphic_layer_index,
        hat.draw_x_px,
        hat.draw_y_px,
        hat.origin_x_px,
        hat.origin_y_px,
        hat.rotation,
        hat.direction_scale,
        hat.vertical_scale,
        hat.live_rendering_executed,
    )
}

fn player_render_hat_draws_label(
    hats: &[LegacyRuntimePlayerRenderHatPreview; 4],
    hat_count: u8,
) -> String {
    if hat_count == 0 {
        return "none".to_owned();
    }

    hats.iter()
        .take(usize::from(hat_count))
        .filter(|hat| hat.drawn)
        .map(|hat| player_render_hat_draw_label(*hat))
        .collect::<Vec<_>>()
        .join("|")
}

fn player_render_preview_label(preview: LegacyRuntimePlayerRenderIntentPreview) -> String {
    format!(
        "index={};power_up={};size={};render_frame={};animation_state={:?};facing={};run_frame={};swim_frame={};ducking={};fire_animation_timer={:.6};fire_animation_active={};image={};quad={};color_layers={};hat_draws={};draw=({:.6},{:.6});rotation={:.6};scale={:.6};live_rendering_executed={};live_player_mutated={}",
        preview.player_index,
        player_power_up_label(preview.power_up),
        preview.size,
        player_render_frame_label(preview.render_frame),
        preview.animation_state,
        horizontal_direction_label(preview.facing),
        preview.run_frame,
        preview.swim_frame,
        bool_label(preview.ducking),
        preview.fire_animation_timer,
        bool_label(preview.fire_animation_active),
        preview.image_path,
        player_render_quad_label(preview.quad),
        player_render_color_layers_label(&preview.color_layers),
        player_render_hat_draws_label(&preview.hat_draws, preview.hat_draw_count),
        preview.draw_x_px,
        preview.draw_y_px,
        preview.rotation,
        preview.scale,
        preview.live_rendering_executed,
        preview.live_player_mutated,
    )
}

fn fireball_map_target_probe_label(probe: LegacyRuntimeFireballMapTargetProbe) -> String {
    format!(
        "index={};coord={};tile={};axis={};target={};state=({:.6},{:.6});speed=({:.6},{:.6});collides={};invisible={};breakable={};coin_block={};block_hit_sound={};live_collision_mutated={}",
        probe.projectile_index,
        coord_label(probe.coord),
        probe.tile_id.0,
        fireball_collision_axis_label(probe.axis),
        fireball_collision_target_label(probe.target),
        probe.state.x,
        probe.state.y,
        probe.state.speed_x,
        probe.state.speed_y,
        probe.collides,
        probe.invisible,
        probe.breakable,
        probe.coin_block,
        probe.play_block_hit_sound,
        probe.live_projectile_collision_mutated,
    )
}

fn fireball_release_summary_label(
    summary: LegacyRuntimeFireballProjectileReleaseSummary,
) -> String {
    format!(
        "index={};source={};callback={};count_delta={};live_queue_mutated={};live_counter_mutated={}",
        summary.projectile_index,
        fireball_release_source_label(summary.source),
        fireball_callback_label(summary.callback.callback),
        summary.callback.fireball_count_delta,
        summary.live_projectile_queue_mutated,
        summary.live_fireball_counter_mutated,
    )
}

fn projected_fireball_count_snapshot_label(
    snapshot: LegacyRuntimeProjectedFireballCountSnapshot,
) -> String {
    format!(
        "source={};count_before={};delta={};count_after={};live_counter_mutated={}",
        projected_fireball_count_source_label(snapshot.source),
        snapshot.active_fireball_count_before,
        snapshot.fireball_count_delta,
        snapshot.active_fireball_count_after,
        snapshot.live_fireball_counter_mutated,
    )
}

fn projected_fireball_count_source_label(
    source: LegacyRuntimeProjectedFireballCountSource,
) -> String {
    match source {
        LegacyRuntimeProjectedFireballCountSource::LaunchIntent => "launch".to_owned(),
        LegacyRuntimeProjectedFireballCountSource::ProjectileUpdateReleaseSummary {
            projectile_index,
        } => format!("projectile_update_release:index={projectile_index}"),
        LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
            projectile_index,
            collision_source,
            axis,
            target,
        } => format!(
            "collision_release:index={};source={};axis={};target={}",
            projectile_index,
            fireball_collision_probe_source_label(collision_source),
            fireball_collision_axis_label(axis),
            fireball_collision_target_label(target),
        ),
    }
}

fn fireball_release_source_label(source: LegacyRuntimeFireballProjectileReleaseSource) -> String {
    match source {
        LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate => {
            "projectile_update".to_owned()
        }
        LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
            source,
            axis,
            target,
        } => format!(
            "collision_probe:source={};axis={};target={}",
            fireball_collision_probe_source_label(source),
            fireball_collision_axis_label(axis),
            fireball_collision_target_label(target),
        ),
    }
}

fn fireball_release_axis_label(source: LegacyRuntimeFireballProjectileReleaseSource) -> String {
    match source {
        LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate => "update".to_owned(),
        LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe { axis, .. } => {
            fireball_collision_axis_label(axis).to_owned()
        }
    }
}

fn fireball_release_target_label(source: LegacyRuntimeFireballProjectileReleaseSource) -> String {
    match source {
        LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate => "offscreen".to_owned(),
        LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe { target, .. } => {
            fireball_collision_target_label(target).to_owned()
        }
    }
}

const fn fireball_callback_label(callback: LegacyRuntimeFireballCallback) -> &'static str {
    match callback {
        LegacyRuntimeFireballCallback::MarioFireballCallback => "mario:fireballcallback",
    }
}

fn fireball_collision_axis_label(axis: LegacyRuntimeFireballCollisionAxis) -> &'static str {
    match axis {
        LegacyRuntimeFireballCollisionAxis::Left => "left",
        LegacyRuntimeFireballCollisionAxis::Right => "right",
        LegacyRuntimeFireballCollisionAxis::Floor => "floor",
        LegacyRuntimeFireballCollisionAxis::Ceiling => "ceiling",
        LegacyRuntimeFireballCollisionAxis::Passive => "passive",
    }
}

fn fireball_collision_target_label(target: LegacyFireballCollisionTarget) -> &'static str {
    match target {
        LegacyFireballCollisionTarget::Tile => "tile",
        LegacyFireballCollisionTarget::BulletBill => "bulletbill",
        LegacyFireballCollisionTarget::PortalWall => "portalwall",
        LegacyFireballCollisionTarget::Spring => "spring",
        LegacyFireballCollisionTarget::Goomba => "goomba",
        LegacyFireballCollisionTarget::Koopa { beetle: false } => "koopa",
        LegacyFireballCollisionTarget::Koopa { beetle: true } => "beetle",
        LegacyFireballCollisionTarget::HammerBro => "hammerbro",
        LegacyFireballCollisionTarget::Plant => "plant",
        LegacyFireballCollisionTarget::Cheep => "cheep",
        LegacyFireballCollisionTarget::Bowser => "bowser",
        LegacyFireballCollisionTarget::Squid => "squid",
        LegacyFireballCollisionTarget::FlyingFish => "flyingfish",
        LegacyFireballCollisionTarget::Lakito => "lakito",
        LegacyFireballCollisionTarget::Other => "other",
    }
}

fn block_bounce_content_label(content: LegacyBlockBounceContentKind) -> &'static str {
    match content {
        LegacyBlockBounceContentKind::Mushroom => "mushroom",
        LegacyBlockBounceContentKind::OneUp => "one-up",
        LegacyBlockBounceContentKind::Star => "star",
        LegacyBlockBounceContentKind::ManyCoins => "many-coins",
        LegacyBlockBounceContentKind::Vine => "vine",
    }
}

fn block_bounce_spawn_content_label(content: Option<LegacyBlockBounceSpawnKind>) -> &'static str {
    match content {
        Some(LegacyBlockBounceSpawnKind::Mushroom) => "mushroom",
        Some(LegacyBlockBounceSpawnKind::OneUp) => "one-up",
        Some(LegacyBlockBounceSpawnKind::Star) => "star",
        Some(LegacyBlockBounceSpawnKind::Vine) => "vine",
        None => "none",
    }
}

fn block_bounce_optional_replay_spawn_label(spawn: Option<LegacyBlockBounceReplaySpawn>) -> String {
    match spawn {
        Some(spawn) => block_bounce_replay_spawn_label(spawn),
        None => "none".to_owned(),
    }
}

fn block_bounce_replay_spawn_label(spawn: LegacyBlockBounceReplaySpawn) -> String {
    format!(
        "kind={};x={:.6};y={:.6}",
        block_bounce_replay_kind_label(spawn.kind),
        spawn.x,
        spawn.y,
    )
}

fn block_bounce_replay_kind_label(kind: LegacyBlockBounceReplayKind) -> &'static str {
    match kind {
        LegacyBlockBounceReplayKind::Mushroom => "mushroom",
        LegacyBlockBounceReplayKind::Flower => "flower",
        LegacyBlockBounceReplayKind::OneUp => "one-up",
        LegacyBlockBounceReplayKind::Star => "star",
        LegacyBlockBounceReplayKind::Vine => "vine",
    }
}

fn block_bounce_schedule_label(schedule: LegacyRuntimePlayerBlockBounceSchedule) -> String {
    format!(
        "block={};queue={};timer={:.9};spawn_content={};hitter_size={};regenerate_sprite_batch={}",
        coord_label(schedule.coord),
        tile_coord_label(schedule.schedule.coord),
        schedule.schedule.timer,
        block_bounce_spawn_content_label(schedule.schedule.spawn_content),
        schedule.schedule.hitter_size,
        bool_label(schedule.schedule.regenerate_sprite_batch),
    )
}

fn block_bounce_completion_label(completion: LegacyRuntimeBlockBounceCompletionReport) -> String {
    format!(
        "index={};tile={};timer={:.6};remove={};suppressed_replay_spawn={}",
        completion.index,
        tile_coord_label(completion.coord),
        completion.timer,
        bool_label(completion.remove),
        block_bounce_optional_replay_spawn_label(completion.suppressed_replay_spawn),
    )
}

fn block_bounce_item_spawn_intent_label(intent: LegacyRuntimeBlockBounceItemSpawnIntent) -> String {
    format!(
        "source_index={};source_tile={};spawn={}",
        intent.source_index,
        tile_coord_label(intent.source_coord),
        block_bounce_replay_spawn_label(intent.spawn),
    )
}

fn contained_reward_reveal_intent_label(
    intent: LegacyRuntimeBlockContainedRewardRevealIntent,
) -> String {
    format!(
        "block={};content={};tile={};tile_id={};sound={}",
        coord_label(intent.coord),
        block_bounce_content_label(intent.content),
        tile_coord_label(intent.outcome.tile_change.coord),
        intent.outcome.tile_change.tile.0,
        block_reveal_sound_label(intent.outcome.sound),
    )
}

fn block_reveal_sound_label(sound: LegacyBlockRevealSound) -> &'static str {
    match sound {
        LegacyBlockRevealSound::MushroomAppear => "mushroomappear",
        LegacyBlockRevealSound::Vine => "vine",
    }
}

fn breakable_cleanup_projections_label(
    projections: &[LegacyRuntimeBreakableBlockCleanupProjection],
) -> String {
    if projections.is_empty() {
        return "[]".to_owned();
    }

    format!(
        "[{}]",
        projections
            .iter()
            .copied()
            .map(breakable_cleanup_projection_label)
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn breakable_cleanup_projection_label(
    projection: LegacyRuntimeBreakableBlockCleanupProjection,
) -> String {
    format!(
        "{}->{}",
        breakable_cleanup_source_label(projection.source),
        breakable_cleanup_action_label(projection.action),
    )
}

fn breakable_cleanup_source_label(source: LegacyRuntimeBreakableBlockCleanupSource) -> String {
    match source {
        LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy { coord } => {
            format!("empty_breakable_block_destroy:{}", coord_label(coord))
        }
    }
}

fn breakable_cleanup_action_label(action: LegacyRuntimeBreakableBlockCleanupAction) -> String {
    match action {
        LegacyRuntimeBreakableBlockCleanupAction::RemoveTileCollisionObject => {
            "remove_tile_collision_object".to_owned()
        }
        LegacyRuntimeBreakableBlockCleanupAction::ClearGels => "clear_gels".to_owned(),
        LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris { index, debris } => format!(
            "spawn_debris:index={index};x={:.6};y={:.6};speed_x={:.6};speed_y={:.6}",
            debris.x, debris.y, debris.speed_x, debris.speed_y,
        ),
        LegacyRuntimeBreakableBlockCleanupAction::RegenerateSpriteBatch => {
            "regenerate_sprite_batch".to_owned()
        }
    }
}

fn scrolling_score_label(label: LegacyScrollingScoreLabel) -> String {
    match label {
        LegacyScrollingScoreLabel::Points(points) => points.to_string(),
        LegacyScrollingScoreLabel::OneUp => "1up".to_owned(),
    }
}

fn coin_block_animation_score_label(score: Option<LegacyCoinBlockAnimationScore>) -> String {
    match score {
        Some(score) => format!(
            "delta={};floating={};x={:.6};y={:.6}",
            score.score_delta, score.floating_score, score.x, score.y,
        ),
        None => "none".to_owned(),
    }
}

fn optional_scrolling_score_state_label(score: Option<LegacyScrollingScoreState>) -> String {
    match score {
        Some(score) => scrolling_score_state_label(score),
        None => "none".to_owned(),
    }
}

fn scrolling_score_state_label(score: LegacyScrollingScoreState) -> String {
    format!(
        "label={};x={:.6};y={:.6};timer={:.6}",
        scrolling_score_label(score.label),
        score.x,
        score.y,
        score.timer,
    )
}

fn coin_block_animation_update_label(
    report: LegacyRuntimeCoinBlockAnimationUpdateReport,
) -> String {
    format!(
        "index={};x={:.6};y={:.6};timer={:.6};frame={};remove={};score={};scrolling_score={}",
        report.index,
        report.state.x,
        report.state.y,
        report.state.timer,
        report.state.frame,
        bool_label(report.remove),
        coin_block_animation_score_label(report.score),
        optional_scrolling_score_state_label(report.scrolling_score),
    )
}

fn block_debris_animation_update_label(
    report: LegacyRuntimeBlockDebrisAnimationUpdateReport,
) -> String {
    format!(
        "index={};source={};debris_index={};x={:.6};y={:.6};speed_x={:.6};speed_y={:.6};timer={:.6};frame={};remove={}",
        report.index,
        breakable_cleanup_source_label(report.source),
        report.debris_index,
        report.state.x,
        report.state.y,
        report.state.speed_x,
        report.state.speed_y,
        report.state.timer,
        report.state.frame,
        bool_label(report.remove),
    )
}

fn scrolling_score_animation_update_label(
    report: LegacyRuntimeScrollingScoreAnimationUpdateReport,
) -> String {
    format!(
        "index={};source={};state={};presentation_x={:.6};presentation_y={:.6};remove={}",
        report.index,
        score_source_label(report.source),
        scrolling_score_state_label(report.state),
        report.presentation.x,
        report.presentation.y,
        bool_label(report.remove),
    )
}

fn optional_frame_detail_label(detail: Option<(usize, String)>) -> String {
    match detail {
        Some((frame_index, label)) => format!("frame={frame_index};{label}"),
        None => "none".to_owned(),
    }
}

fn portal_trace_hit_side_label(hit: Option<LegacyRuntimePortalTraceHit>) -> &'static str {
    match hit {
        Some(hit) => facing_label(hit.side),
        None => "none",
    }
}

fn portal_trace_hit_tendency_label(hit: Option<LegacyRuntimePortalTraceHit>) -> String {
    match hit {
        Some(hit) => hit.tendency.to_string(),
        None => "none".to_owned(),
    }
}

fn portal_outcome_intent_label(intent: LegacyRuntimePortalOutcomeIntent) -> String {
    format!(
        "slot={};kind={};placement_tile={};placement_side={};sound={}",
        portal_slot_label(Some(intent.requested_slot)),
        portal_outcome_kind_label(intent.kind),
        portal_placement_coord_label(intent.placement),
        portal_placement_side_label(intent.placement),
        sound_effect_label(intent.sound),
    )
}

fn portal_transit_audio_intent_label(intent: LegacyRuntimePortalTransitAudioIntent) -> String {
    format!(
        "outcome={};entry_slot={};exit_slot={};sound={}",
        portal_transit_outcome_kind_label(intent.outcome_kind),
        portal_slot_label(Some(intent.entry_slot)),
        portal_slot_label(Some(intent.exit_slot)),
        sound_effect_label(intent.sound),
    )
}

fn portal_placement_coord_label(placement: Option<LegacyRuntimePortalPlacement>) -> String {
    match placement {
        Some(placement) => coord_label(placement.coord),
        None => "none".to_owned(),
    }
}

fn portal_placement_side_label(placement: Option<LegacyRuntimePortalPlacement>) -> &'static str {
    match placement {
        Some(placement) => facing_label(placement.side),
        None => "none",
    }
}

fn tile_reservations_label(reservations: [LegacyMapTileCoord; 2]) -> String {
    format!(
        "[{}|{}]",
        coord_label(reservations[0]),
        coord_label(reservations[1])
    )
}

fn wall_reservations_label(reservations: [LegacyRuntimePortalWallReservation; 3]) -> String {
    format!(
        "[{}|{}|{}]",
        wall_reservation_label(reservations[0]),
        wall_reservation_label(reservations[1]),
        wall_reservation_label(reservations[2])
    )
}

fn wall_reservation_label(reservation: LegacyRuntimePortalWallReservation) -> String {
    format!(
        "({},{};{}x{})",
        reservation.x, reservation.y, reservation.width, reservation.height
    )
}

fn projected_portal_slot_label(portal: Option<LegacyRuntimeProjectedPortal>) -> &'static str {
    match portal {
        Some(portal) => portal_slot_label(Some(portal.requested_slot)),
        None => "none",
    }
}

fn projected_portal_coord_label(portal: Option<LegacyRuntimeProjectedPortal>) -> String {
    match portal {
        Some(portal) => portal_placement_coord_label(Some(portal.placement)),
        None => "none".to_owned(),
    }
}

fn projected_portal_side_label(portal: Option<LegacyRuntimeProjectedPortal>) -> &'static str {
    match portal {
        Some(portal) => portal_placement_side_label(Some(portal.placement)),
        None => "none",
    }
}

fn portal_pairing_label(pairing: Option<LegacyRuntimePortalPairing>) -> String {
    match pairing {
        Some(pairing) => format!(
            "{}->{}",
            portal_slot_label(Some(pairing.entry_slot)),
            portal_slot_label(Some(pairing.exit_slot))
        ),
        None => "none".to_owned(),
    }
}

fn portal_pairing_entry_slot_label(pairing: Option<LegacyRuntimePortalPairing>) -> &'static str {
    match pairing {
        Some(pairing) => portal_slot_label(Some(pairing.entry_slot)),
        None => "none",
    }
}

fn portal_pairing_exit_slot_label(pairing: Option<LegacyRuntimePortalPairing>) -> &'static str {
    match pairing {
        Some(pairing) => portal_slot_label(Some(pairing.exit_slot)),
        None => "none",
    }
}

fn optional_coord_label(coord: Option<LegacyMapTileCoord>) -> String {
    match coord {
        Some(coord) => coord_label(coord),
        None => "none".to_owned(),
    }
}

fn player_coin_pickup_label(pickup: LegacyRuntimePlayerCoinPickup) -> String {
    format!(
        "pickup_tile={};tile_id={};clear_tile={};clear_tile_id={};score_delta={};sound={}",
        coord_label(pickup.coord),
        pickup.tile_id.0,
        coord_label(pickup.coord),
        pickup.clear_tile_id.0,
        pickup.score_delta,
        sound_effect_label(pickup.sound),
    )
}

fn audio_command_sequence_label(commands: &[LegacyAudioCommand]) -> String {
    if commands.is_empty() {
        return "none".to_owned();
    }
    commands
        .iter()
        .map(|command| match *command {
            LegacyAudioCommand::StopSound(sound) => format!("stop:{}", sound_effect_label(sound)),
            LegacyAudioCommand::PlaySound(sound) => format!("play:{}", sound_effect_label(sound)),
            LegacyAudioCommand::StopAll => "stop_all".to_owned(),
            LegacyAudioCommand::PauseAll => "pause_all".to_owned(),
            LegacyAudioCommand::SetMasterVolume(volume) => format!("master_volume:{volume:.6}"),
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn audio_command_sequence_detail_label(commands: &[LegacyAudioCommand]) -> String {
    if commands.is_empty() {
        return "none".to_owned();
    }
    commands
        .iter()
        .enumerate()
        .map(|(index, command)| format!("{index}:{}", audio_command_sequence_label(&[*command])))
        .collect::<Vec<_>>()
        .join("|")
}

fn player_tile_collision_label(collision: LegacyRuntimePlayerTileCollision) -> String {
    format!(
        "tile={};tile_id={};axis={}",
        coord_label(collision.coord),
        collision.tile_id.0,
        player_collision_axis_label(collision.axis),
    )
}

fn ceiling_block_hit_label(block_hit: LegacyRuntimePlayerCeilingBlockHit) -> String {
    format!(
        "tile={};tile_id={};breakable={};coin_block={};play_hit_sound={};portal_guard={}",
        coord_label(block_hit.coord),
        block_hit.tile_id.0,
        bool_label(block_hit.breakable),
        bool_label(block_hit.coin_block),
        bool_label(block_hit.play_hit_sound),
        optional_portal_guard_source_label(block_hit.portal_guard.map(|guard| guard.source)),
    )
}

fn player_collision_axis_label(axis: LegacyRuntimePlayerCollisionAxis) -> &'static str {
    match axis {
        LegacyRuntimePlayerCollisionAxis::Horizontal => "horizontal",
        LegacyRuntimePlayerCollisionAxis::Vertical => "vertical",
    }
}

fn tile_coord_label(coord: TileCoord) -> String {
    format!("({},{})", coord.x, coord.y)
}

fn portal_block_guard_source_label(source: LegacyRuntimePortalBlockGuardSource) -> &'static str {
    match source {
        LegacyRuntimePortalBlockGuardSource::ExplicitReservation => "explicit_reservation",
        LegacyRuntimePortalBlockGuardSource::ProjectedPortalState => "projected_portal_state",
    }
}

fn optional_portal_guard_source_label(
    source: Option<LegacyRuntimePortalBlockGuardSource>,
) -> &'static str {
    match source {
        Some(source) => portal_block_guard_source_label(source),
        None => "none",
    }
}

fn blocked_exit_probe_label(probe: Option<LegacyRuntimePortalBlockedExitProbe>) -> String {
    match probe {
        Some(probe) => format!(
            "{};{};{:.6},{:.6}",
            coord_label(probe.blocking_coord),
            blocked_exit_bounce_axis_label(probe.bounce_axis),
            probe.bounced_speed_x,
            probe.bounced_speed_y
        ),
        None => "none".to_owned(),
    }
}

fn blocked_exit_bounce_axis_label(axis: LegacyRuntimePortalBlockedExitBounceAxis) -> &'static str {
    match axis {
        LegacyRuntimePortalBlockedExitBounceAxis::Horizontal => "horizontal",
        LegacyRuntimePortalBlockedExitBounceAxis::Vertical => "vertical",
    }
}

fn coord_label(coord: LegacyMapTileCoord) -> String {
    format!("({},{})", coord.x, coord.y)
}

fn sound_effect_label(sound: LegacySoundEffect) -> &'static str {
    sound.lua_global()
}

fn facing_label(facing: Facing) -> &'static str {
    match facing {
        Facing::Up => "up",
        Facing::Down => "down",
        Facing::Left => "left",
        Facing::Right => "right",
    }
}

fn portal_outcome_kind_label(kind: LegacyRuntimePortalOutcomeKind) -> &'static str {
    match kind {
        LegacyRuntimePortalOutcomeKind::Open => "open",
        LegacyRuntimePortalOutcomeKind::Fizzle => "fizzle",
    }
}

fn portal_transit_outcome_kind_label(kind: LegacyRuntimePortalTransitOutcomeKind) -> &'static str {
    match kind {
        LegacyRuntimePortalTransitOutcomeKind::TeleportPreview => "teleport_preview",
        LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview => {
            "blocked_exit_bounce_preview"
        }
    }
}

fn projected_player_state_source_label(
    source: LegacyRuntimeProjectedPlayerStateSource,
) -> &'static str {
    match source {
        LegacyRuntimeProjectedPlayerStateSource::PortalTransitTeleportPreview => {
            "portal_transit_teleport_preview"
        }
        LegacyRuntimeProjectedPlayerStateSource::PortalTransitBlockedExitBouncePreview => {
            "portal_transit_blocked_exit_bounce_preview"
        }
    }
}

fn horizontal_direction_label(direction: HorizontalDirection) -> &'static str {
    match direction {
        HorizontalDirection::Left => "left",
        HorizontalDirection::Right => "right",
    }
}

fn enemy_direction_label(direction: LegacyEnemyDirection) -> &'static str {
    match direction {
        LegacyEnemyDirection::Left => "left",
        LegacyEnemyDirection::Right => "right",
    }
}

fn item_kind_label(kind: LegacyBlockJumpItemKind) -> &'static str {
    match kind {
        LegacyBlockJumpItemKind::Mushroom => "mushroom",
        LegacyBlockJumpItemKind::OneUp => "one-up",
    }
}

fn bool_label(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn portal_target_player_source_label(
    source: LegacyRuntimePortalTargetPlayerSource,
) -> &'static str {
    match source {
        LegacyRuntimePortalTargetPlayerSource::LivePlayer => "live_player",
        LegacyRuntimePortalTargetPlayerSource::ProjectedPortalTransit => "projected_portal_transit",
    }
}

fn portal_slot_label(slot: Option<LegacyRuntimePortalSlot>) -> &'static str {
    match slot {
        Some(LegacyRuntimePortalSlot::Portal1) => "portal1",
        Some(LegacyRuntimePortalSlot::Portal2) => "portal2",
        None => "none",
    }
}

fn parse_args(args: Vec<String>) -> Result<(PathBuf, LegacyRuntimeHarnessConfig), Box<dyn Error>> {
    let mut repo_root = PathBuf::from(".");
    let mut config = LegacyRuntimeHarnessConfig::default();
    let mut input = LegacyRuntimeHarnessInput::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--repo-root" => {
                index += 1;
                repo_root = PathBuf::from(next_value(&args, index, "--repo-root")?);
            }
            "--mappack" => {
                index += 1;
                config.selection.mappack = next_value(&args, index, "--mappack")?;
            }
            "--level" => {
                index += 1;
                config.selection.filename = next_value(&args, index, "--level")?;
            }
            "--world" => {
                index += 1;
                config.selection.world = parse_value(&args, index, "--world")?;
            }
            "--level-index" => {
                index += 1;
                config.selection.level = parse_value(&args, index, "--level-index")?;
            }
            "--sublevel" => {
                index += 1;
                config.selection.sublevel = parse_value(&args, index, "--sublevel")?;
            }
            "--frames" => {
                index += 1;
                config.frames = parse_value(&args, index, "--frames")?;
            }
            "--raw-dt" => {
                index += 1;
                config.raw_dt = parse_value(&args, index, "--raw-dt")?;
            }
            "--left" => input.left = true,
            "--right" => input.right = true,
            "--run" => input.run = true,
            "--fire" => input.fire = true,
            "--fire-flower" => input.fire_flower_power = true,
            "--fire-ducking" => input.fire_ducking = true,
            "--fireball-count" => {
                index += 1;
                input.active_fireball_count = parse_value(&args, index, "--fireball-count")?;
            }
            "--seed-fireball" => {
                index += 1;
                config
                    .initial_fireball_projectiles
                    .push(parse_fireball_projectile(&args, index, "--seed-fireball")?);
            }
            "--probe-fireball-collision" => {
                index += 1;
                config.fireball_collision_probe =
                    Some(parse_fireball_collision_probe(&args, index)?);
            }
            "--seed-fireball-enemy" => {
                index += 1;
                config.fireball_enemies.push(parse_fireball_enemy_snapshot(
                    &args,
                    index,
                    "--seed-fireball-enemy",
                )?);
            }
            "--portal1" => input.portal_1 = true,
            "--portal2" => input.portal_2 = true,
            "--aim" => {
                index += 1;
                input.pointing_angle = parse_value(&args, index, "--aim")?;
            }
            "--seed-portal1" => {
                index += 1;
                let placement = parse_portal_placement(&args, index, "--seed-portal1")?;
                seed_projected_portal(&mut config, LegacyRuntimePortalSlot::Portal1, placement)?;
            }
            "--seed-portal2" => {
                index += 1;
                let placement = parse_portal_placement(&args, index, "--seed-portal2")?;
                seed_projected_portal(&mut config, LegacyRuntimePortalSlot::Portal2, placement)?;
            }
            "--player-body" => {
                index += 1;
                let body = parse_player_body(&args, index, "--player-body")?;
                config.initial_player.body = body;
                config.force_initial_player_seed = true;
            }
            "--player-speed" => {
                index += 1;
                let (speed_x, speed_y) = parse_player_speed(&args, index, "--player-speed")?;
                config.initial_player.movement.speed_x = speed_x;
                config.initial_player.movement.speed_y = speed_y;
                config.force_initial_player_seed = true;
            }
            "--seed-jump-item" => {
                index += 1;
                config
                    .jump_items
                    .push(parse_jump_item_snapshot(&args, index, "--seed-jump-item")?);
            }
            "--seed-top-enemy" => {
                index += 1;
                config.top_enemies.push(parse_top_enemy_snapshot(
                    &args,
                    index,
                    "--seed-top-enemy",
                )?);
            }
            "--seed-many-coins-timer" => {
                index += 1;
                config.many_coins_timers.push(parse_many_coins_timer(
                    &args,
                    index,
                    "--seed-many-coins-timer",
                )?);
            }
            arg => return Err(format!("unknown runtime harness argument: {arg}").into()),
        }

        index += 1;
    }

    config.input = input;
    Ok((repo_root, config))
}

fn seed_projected_portal(
    config: &mut LegacyRuntimeHarnessConfig,
    requested_slot: LegacyRuntimePortalSlot,
    placement: LegacyRuntimePortalPlacement,
) -> Result<(), Box<dyn Error>> {
    let Some(projection) =
        legacy_runtime_portal_reservation_projection(LegacyRuntimePortalOutcomeIntent {
            requested_slot,
            kind: LegacyRuntimePortalOutcomeKind::Open,
            placement: Some(placement),
            sound: match requested_slot {
                LegacyRuntimePortalSlot::Portal1 => LegacySoundEffect::Portal1Open,
                LegacyRuntimePortalSlot::Portal2 => LegacySoundEffect::Portal2Open,
            },
        })
    else {
        return Err("projected portal seed must describe an open placement".into());
    };

    config
        .initial_projected_portal_state
        .apply_projection(projection);
    Ok(())
}

fn parse_portal_placement(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyRuntimePortalPlacement, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 3)?;
    Ok(LegacyRuntimePortalPlacement {
        coord: LegacyMapTileCoord::new(
            parse_csv_value(parts[0], flag, "tile x")?,
            parse_csv_value(parts[1], flag, "tile y")?,
        ),
        side: parse_facing(parts[2], flag)?,
    })
}

fn parse_player_body(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<PlayerBodyBounds, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 4)?;
    Ok(PlayerBodyBounds::new(
        parse_csv_value(parts[0], flag, "x")?,
        parse_csv_value(parts[1], flag, "y")?,
        parse_csv_value(parts[2], flag, "width")?,
        parse_csv_value(parts[3], flag, "height")?,
    ))
}

fn parse_player_speed(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<(f32, f32), Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 2)?;
    Ok((
        parse_csv_value(parts[0], flag, "speed x")?,
        parse_csv_value(parts[1], flag, "speed y")?,
    ))
}

fn parse_jump_item_snapshot(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyRuntimeBlockJumpItemSnapshot, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 7)?;
    Ok(LegacyRuntimeBlockJumpItemSnapshot::new(
        parse_jump_item_kind(parts[0], flag)?,
        parse_csv_value(parts[1], flag, "index")?,
        parse_csv_value(parts[2], flag, "x")?,
        parse_csv_value(parts[3], flag, "y")?,
        parse_csv_value(parts[4], flag, "width")?,
        parse_csv_value(parts[5], flag, "height")?,
        parse_csv_value(parts[6], flag, "has jump handler")?,
    ))
}

fn parse_top_enemy_snapshot(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyRuntimeBlockTopEnemySnapshot, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 6)?;
    Ok(LegacyRuntimeBlockTopEnemySnapshot::new(
        parse_csv_value(parts[0], flag, "index")?,
        parse_csv_value(parts[1], flag, "x")?,
        parse_csv_value(parts[2], flag, "y")?,
        parse_csv_value(parts[3], flag, "width")?,
        parse_csv_value(parts[4], flag, "height")?,
        parse_csv_value(parts[5], flag, "has shotted handler")?,
    ))
}

fn parse_many_coins_timer(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyManyCoinsTimerEntry, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 3)?;
    Ok(LegacyManyCoinsTimerEntry {
        coord: TileCoord::new(
            parse_csv_value(parts[0], flag, "tile x")?,
            parse_csv_value(parts[1], flag, "tile y")?,
        ),
        remaining: parse_csv_value(parts[2], flag, "remaining")?,
    })
}

fn parse_fireball_projectile(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyFireballState, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 3)?;
    Ok(LegacyFireballState::spawn(
        parse_csv_value(parts[0], flag, "source x")?,
        parse_csv_value(parts[1], flag, "source y")?,
        parse_enemy_direction(parts[2], flag)?,
        iw2wth_core::LegacyFireballConstants::default(),
    ))
}

fn parse_fireball_collision_probe(
    args: &[String],
    index: usize,
) -> Result<LegacyRuntimeFireballCollisionProbeRequest, Box<dyn Error>> {
    let value = next_value(args, index, "--probe-fireball-collision")?;
    let parts = split_csv(&value, "--probe-fireball-collision", 3)?;
    Ok(LegacyRuntimeFireballCollisionProbeRequest::new(
        parse_csv_value(parts[0], "--probe-fireball-collision", "projectile index")?,
        parse_fireball_collision_axis(parts[1], "--probe-fireball-collision")?,
        parse_fireball_collision_target(parts[2], "--probe-fireball-collision")?,
    ))
}

fn parse_fireball_enemy_snapshot(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<LegacyRuntimeFireballEnemySnapshot, Box<dyn Error>> {
    let value = next_value(args, index, flag)?;
    let parts = split_csv(&value, flag, 7)?;
    Ok(LegacyRuntimeFireballEnemySnapshot::new(
        parse_fireball_collision_target(parts[0], flag)?,
        parse_csv_value(parts[1], flag, "index")?,
        parse_csv_value(parts[2], flag, "x")?,
        parse_csv_value(parts[3], flag, "y")?,
        parse_csv_value(parts[4], flag, "width")?,
        parse_csv_value(parts[5], flag, "height")?,
        parse_csv_value(parts[6], flag, "has shotted handler")?,
    ))
}

fn parse_fireball_collision_axis(
    value: &str,
    flag: &str,
) -> Result<LegacyRuntimeFireballCollisionAxis, Box<dyn Error>> {
    match value {
        "left" => Ok(LegacyRuntimeFireballCollisionAxis::Left),
        "right" => Ok(LegacyRuntimeFireballCollisionAxis::Right),
        "floor" => Ok(LegacyRuntimeFireballCollisionAxis::Floor),
        "ceiling" | "ceil" => Ok(LegacyRuntimeFireballCollisionAxis::Ceiling),
        "passive" => Ok(LegacyRuntimeFireballCollisionAxis::Passive),
        _ => Err(format!(
            "invalid axis in {flag}: expected left, right, floor, ceiling, or passive"
        )
        .into()),
    }
}

fn parse_fireball_collision_target(
    value: &str,
    flag: &str,
) -> Result<LegacyFireballCollisionTarget, Box<dyn Error>> {
    match value {
        "tile" => Ok(LegacyFireballCollisionTarget::Tile),
        "bulletbill" | "bullet_bill" => Ok(LegacyFireballCollisionTarget::BulletBill),
        "portalwall" | "portal_wall" => Ok(LegacyFireballCollisionTarget::PortalWall),
        "spring" => Ok(LegacyFireballCollisionTarget::Spring),
        "goomba" => Ok(LegacyFireballCollisionTarget::Goomba),
        "koopa" => Ok(LegacyFireballCollisionTarget::Koopa { beetle: false }),
        "beetle" => Ok(LegacyFireballCollisionTarget::Koopa { beetle: true }),
        "hammerbro" | "hammer_bro" => Ok(LegacyFireballCollisionTarget::HammerBro),
        "plant" => Ok(LegacyFireballCollisionTarget::Plant),
        "cheep" => Ok(LegacyFireballCollisionTarget::Cheep),
        "bowser" => Ok(LegacyFireballCollisionTarget::Bowser),
        "squid" => Ok(LegacyFireballCollisionTarget::Squid),
        "flyingfish" | "flying_fish" => Ok(LegacyFireballCollisionTarget::FlyingFish),
        "lakito" => Ok(LegacyFireballCollisionTarget::Lakito),
        "other" => Ok(LegacyFireballCollisionTarget::Other),
        _ => Err(
            format!("invalid target in {flag}: expected a fireball collision target label").into(),
        ),
    }
}

fn parse_jump_item_kind(
    value: &str,
    flag: &str,
) -> Result<LegacyBlockJumpItemKind, Box<dyn Error>> {
    match value {
        "mushroom" => Ok(LegacyBlockJumpItemKind::Mushroom),
        "oneup" | "one-up" => Ok(LegacyBlockJumpItemKind::OneUp),
        _ => Err(format!("invalid item kind in {flag}: expected mushroom or oneup").into()),
    }
}

fn parse_enemy_direction(value: &str, flag: &str) -> Result<LegacyEnemyDirection, Box<dyn Error>> {
    match value {
        "left" => Ok(LegacyEnemyDirection::Left),
        "right" => Ok(LegacyEnemyDirection::Right),
        _ => Err(format!("invalid direction in {flag}: expected left or right").into()),
    }
}

fn split_csv<'a>(
    value: &'a str,
    flag: &str,
    expected_len: usize,
) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let parts = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != expected_len {
        return Err(format!("{flag} requires {expected_len} comma-separated values").into());
    }
    Ok(parts)
}

fn parse_csv_value<T>(value: &str, flag: &str, label: &str) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    value
        .parse::<T>()
        .map_err(|source| format!("invalid {label} in {flag}: {source}").into())
}

fn parse_facing(value: &str, flag: &str) -> Result<Facing, Box<dyn Error>> {
    match value {
        "up" => Ok(Facing::Up),
        "down" => Ok(Facing::Down),
        "left" => Ok(Facing::Left),
        "right" => Ok(Facing::Right),
        _ => Err(format!("invalid side in {flag}: expected up, down, left, or right").into()),
    }
}

fn next_value(args: &[String], index: usize, flag: &str) -> Result<String, Box<dyn Error>> {
    args.get(index)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value").into())
}

fn parse_value<T>(args: &[String], index: usize, flag: &str) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    let value = next_value(args, index, flag)?;
    value
        .parse::<T>()
        .map_err(|source| format!("invalid value for {flag}: {source}").into())
}

fn print_usage() {
    println!(
        "usage: cargo run --manifest-path rust/Cargo.toml -p iw2wth_runtime --bin iw2wth-runtime-harness -- [options]"
    );
    println!(
        "options: --repo-root PATH --mappack NAME --level FILE --world N --level-index N --sublevel N --frames N --raw-dt SECONDS --left --right --run --fire --fire-flower --fire-ducking --fireball-count N --seed-fireball X,Y,DIRECTION --probe-fireball-collision INDEX,AXIS,TARGET --seed-fireball-enemy TARGET,INDEX,X,Y,W,H,HAS_SHOTTED_HANDLER --portal1 --portal2 --aim RADIANS --seed-portal1 X,Y,SIDE --seed-portal2 X,Y,SIDE --player-body X,Y,W,H --player-speed X,Y --seed-jump-item KIND,INDEX,X,Y,W,H,HAS_JUMP_HANDLER --seed-top-enemy INDEX,X,Y,W,H,HAS_SHOTTED_HANDLER --seed-many-coins-timer X,Y,REMAINING"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use iw2wth_core::{PlayerAnimationState, PlayerMovementState};
    use iw2wth_runtime::shell::{
        LegacyRuntimeFireballCallbackMetadata, LegacyRuntimeFireballRenderQuad,
        LegacyRuntimePlayer, LegacyRuntimePlayerRenderTint,
    };

    #[test]
    fn parse_args_seeds_projected_portals_and_player_state() {
        let (repo_root, config) = parse_args(vec![
            "--repo-root".to_owned(),
            "/tmp/iw2wth".to_owned(),
            "--seed-portal1".to_owned(),
            "2,3,up".to_owned(),
            "--seed-portal2".to_owned(),
            "9,4,right".to_owned(),
            "--player-body".to_owned(),
            "1,3.2,0.75,0.75".to_owned(),
            "--player-speed".to_owned(),
            "4,-80".to_owned(),
        ])
        .expect("seed flags should parse");

        assert_eq!(repo_root, PathBuf::from("/tmp/iw2wth"));
        assert!(config.force_initial_player_seed);
        assert_eq!(
            config.initial_player.body,
            PlayerBodyBounds::new(1.0, 3.2, 0.75, 0.75),
        );
        assert_eq!(config.initial_player.movement.speed_x, 4.0);
        assert_eq!(config.initial_player.movement.speed_y, -80.0);

        let portal_1 = config
            .initial_projected_portal_state
            .slot(LegacyRuntimePortalSlot::Portal1)
            .expect("portal1 should be seeded");
        assert_eq!(
            portal_1.placement,
            LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(2, 3),
                side: Facing::Up,
            },
        );
        assert_eq!(
            portal_1.tile_reservations,
            [LegacyMapTileCoord::new(2, 3), LegacyMapTileCoord::new(3, 3)],
        );
        assert_eq!(
            portal_1.wall_reservations,
            [
                LegacyRuntimePortalWallReservation::new(1, 3, 2, 0),
                LegacyRuntimePortalWallReservation::new(1, 2, 0, 1),
                LegacyRuntimePortalWallReservation::new(3, 2, 0, 1),
            ],
        );

        let portal_2 = config
            .initial_projected_portal_state
            .slot(LegacyRuntimePortalSlot::Portal2)
            .expect("portal2 should be seeded");
        assert_eq!(
            portal_2.placement,
            LegacyRuntimePortalPlacement {
                coord: LegacyMapTileCoord::new(9, 4),
                side: Facing::Right,
            },
        );
        assert_eq!(
            portal_2.tile_reservations,
            [LegacyMapTileCoord::new(9, 4), LegacyMapTileCoord::new(9, 5)],
        );
    }

    #[test]
    fn parse_args_configures_report_only_fireball_launch_input() {
        let (_, config) = parse_args(vec![
            "--fire".to_owned(),
            "--fire-flower".to_owned(),
            "--fireball-count".to_owned(),
            "1".to_owned(),
            "--aim".to_owned(),
            "0.25".to_owned(),
        ])
        .expect("fireball launch flags should parse");

        assert!(config.input.fire);
        assert!(config.input.fire_flower_power);
        assert!(!config.input.fire_ducking);
        assert_eq!(config.input.active_fireball_count, 1);
        assert_eq!(config.input.pointing_angle, 0.25);
    }

    #[test]
    fn parse_args_configures_seeded_fireball_projectile_progress_input() {
        let (_, config) = parse_args(vec!["--seed-fireball".to_owned(), "-2,4,left".to_owned()])
            .expect("seeded fireball flag should parse");

        assert_eq!(config.initial_fireball_projectiles.len(), 1);
        let projectile = config.initial_fireball_projectiles[0];
        assert_eq!(projectile.x, -2.0);
        assert_eq!(projectile.y, 4.25);
        assert_eq!(projectile.speed_x, -15.0);
        assert!(projectile.active);
        assert!(!projectile.destroy);
    }

    #[test]
    fn parse_args_configures_report_only_fireball_collision_probe_input() {
        let (_, config) = parse_args(vec![
            "--probe-fireball-collision".to_owned(),
            "1,floor,beetle".to_owned(),
            "--seed-fireball-enemy".to_owned(),
            "goomba,7,3.5,4,1,1,true".to_owned(),
        ])
        .expect("fireball collision probe flag should parse");

        assert_eq!(
            config.fireball_collision_probe,
            Some(LegacyRuntimeFireballCollisionProbeRequest::new(
                1,
                LegacyRuntimeFireballCollisionAxis::Floor,
                LegacyFireballCollisionTarget::Koopa { beetle: true },
            )),
        );
        assert_eq!(
            config.fireball_enemies,
            vec![LegacyRuntimeFireballEnemySnapshot::new(
                LegacyFireballCollisionTarget::Goomba,
                7,
                3.5,
                4.0,
                1.0,
                1.0,
                true,
            )],
        );
    }

    #[test]
    fn cli_labels_fireball_map_target_probe_metadata() {
        let state = LegacyFireballState::spawn(
            1.0,
            3.5,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        let label = fireball_map_target_probe_label(LegacyRuntimeFireballMapTargetProbe {
            projectile_index: 0,
            state,
            coord: LegacyMapTileCoord::new(3, 4),
            tile_id: iw2wth_core::TileId(5),
            axis: LegacyRuntimeFireballCollisionAxis::Right,
            target: LegacyFireballCollisionTarget::Tile,
            collides: true,
            invisible: false,
            breakable: true,
            coin_block: true,
            play_block_hit_sound: true,
            live_projectile_collision_mutated: false,
        });

        assert_eq!(
            label,
            "index=0;coord=(3,4);tile=5;axis=right;target=tile;state=(1.375000,3.750000);speed=(15.000000,0.000000);collides=true;invisible=false;breakable=true;coin_block=true;block_hit_sound=true;live_collision_mutated=false",
        );
    }

    #[test]
    fn cli_labels_fireball_enemy_hit_intent_metadata() {
        let intent = LegacyRuntimeFireballEnemyHitIntent {
            projectile_index: 2,
            source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
            axis: LegacyRuntimeFireballCollisionAxis::Passive,
            target: LegacyFireballCollisionTarget::Goomba,
            enemy: LegacyRuntimeFireballEnemySnapshot::new(
                LegacyFireballCollisionTarget::Goomba,
                7,
                3.5,
                4.0,
                1.0,
                1.0,
                true,
            ),
            shot_direction: LegacyEnemyDirection::Right,
            score_delta: Some(100),
            score_x: 3.375,
            score_y: 4.25,
            live_enemy_mutated: false,
        };
        let label = fireball_enemy_hit_intent_label(intent);

        assert_eq!(
            label,
            "index=2;source=explicit_request;axis=passive;target=goomba;enemy_index=7;enemy_target=goomba;enemy=(3.500000,4.000000,1.000000,1.000000);shot_direction=right;score_delta=100;score=(3.375000,4.250000);live_enemy_mutated=false",
        );

        let projected_label = projected_fireball_enemy_hit_snapshot_label(
            LegacyRuntimeProjectedFireballEnemyHitSnapshot::from_intent(intent),
        );
        assert_eq!(
            projected_label,
            "index=2;target=goomba;enemy_index=7;active_after=false;shot_after=true;removed_from_future_queries=true;live_enemy_mutated=false",
        );

        let overlap_intent = LegacyRuntimeFireballEnemyHitIntent {
            source: LegacyRuntimeFireballCollisionProbeSource::EnemyOverlapProbe { enemy_index: 7 },
            ..intent
        };
        assert_eq!(
            fireball_enemy_hit_intent_label(overlap_intent),
            "index=2;source=enemy_overlap_probe:enemy_index=7;axis=passive;target=goomba;enemy_index=7;enemy_target=goomba;enemy=(3.500000,4.000000,1.000000,1.000000);shot_direction=right;score_delta=100;score=(3.375000,4.250000);live_enemy_mutated=false",
        );
    }

    #[test]
    fn cli_labels_fireball_collision_release_callback_metadata() {
        let summary =
            fireball_release_summary_label(LegacyRuntimeFireballProjectileReleaseSummary {
                projectile_index: 2,
                source: LegacyRuntimeFireballProjectileReleaseSource::CollisionProbe {
                    source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    axis: LegacyRuntimeFireballCollisionAxis::Passive,
                    target: LegacyFireballCollisionTarget::Goomba,
                },
                callback: LegacyRuntimeFireballCallbackMetadata {
                    callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
                    fireball_count_delta: -1,
                },
                live_projectile_queue_mutated: false,
                live_fireball_counter_mutated: false,
            });

        assert_eq!(
            summary,
            "index=2;source=collision_probe:source=explicit_request;axis=passive;target=goomba;callback=mario:fireballcallback;count_delta=-1;live_queue_mutated=false;live_counter_mutated=false",
        );

        let update_summary =
            fireball_release_summary_label(LegacyRuntimeFireballProjectileReleaseSummary {
                projectile_index: 1,
                source: LegacyRuntimeFireballProjectileReleaseSource::ProjectileUpdate,
                callback: LegacyRuntimeFireballCallbackMetadata {
                    callback: LegacyRuntimeFireballCallback::MarioFireballCallback,
                    fireball_count_delta: -1,
                },
                live_projectile_queue_mutated: false,
                live_fireball_counter_mutated: false,
            });

        assert_eq!(
            update_summary,
            "index=1;source=projectile_update;callback=mario:fireballcallback;count_delta=-1;live_queue_mutated=false;live_counter_mutated=false",
        );
    }

    #[test]
    fn cli_labels_projected_fireball_count_snapshot_metadata() {
        let launch_summary =
            projected_fireball_count_snapshot_label(LegacyRuntimeProjectedFireballCountSnapshot {
                source: LegacyRuntimeProjectedFireballCountSource::LaunchIntent,
                active_fireball_count_before: 1,
                fireball_count_delta: 1,
                active_fireball_count_after: 2,
                live_fireball_counter_mutated: false,
            });

        assert_eq!(
            launch_summary,
            "source=launch;count_before=1;delta=1;count_after=2;live_counter_mutated=false",
        );

        let summary =
            projected_fireball_count_snapshot_label(LegacyRuntimeProjectedFireballCountSnapshot {
                source: LegacyRuntimeProjectedFireballCountSource::CollisionReleaseSummary {
                    projectile_index: 2,
                    collision_source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                    axis: LegacyRuntimeFireballCollisionAxis::Passive,
                    target: LegacyFireballCollisionTarget::Goomba,
                },
                active_fireball_count_before: 2,
                fireball_count_delta: -1,
                active_fireball_count_after: 1,
                live_fireball_counter_mutated: false,
            });

        assert_eq!(
            summary,
            "source=collision_release:index=2;source=explicit_request;axis=passive;target=goomba;count_before=2;delta=-1;count_after=1;live_counter_mutated=false",
        );

        let update_summary =
            projected_fireball_count_snapshot_label(LegacyRuntimeProjectedFireballCountSnapshot {
                source: LegacyRuntimeProjectedFireballCountSource::ProjectileUpdateReleaseSummary {
                    projectile_index: 1,
                },
                active_fireball_count_before: 2,
                fireball_count_delta: -1,
                active_fireball_count_after: 1,
                live_fireball_counter_mutated: false,
            });

        assert_eq!(
            update_summary,
            "source=projectile_update_release:index=1;count_before=2;delta=-1;count_after=1;live_counter_mutated=false",
        );
    }

    #[test]
    fn cli_labels_fireball_render_preview_metadata() {
        let state = LegacyFireballState::spawn(
            2.0,
            3.0,
            LegacyEnemyDirection::Right,
            iw2wth_core::LegacyFireballConstants::default(),
        );
        let preview = LegacyRuntimeFireballRenderIntentPreview {
            projectile_index: 0,
            source: LegacyRuntimeFireballRenderSource::ProjectedProjectileCollision,
            state,
            frame: LegacyFireballFrame::ExplosionOne,
            frame_kind: LegacyRuntimeFireballRenderFrameKind::Explosion,
            image_path: "graphics/SMB/fireball.png",
            quad: LegacyRuntimeFireballRenderQuad {
                x_px: 32,
                y_px: 0,
                width_px: 16,
                height_px: 16,
            },
            draw_x_px: 44.0,
            draw_y_px: 48.0,
            rotation: 0.0,
            scale: 2.0,
            live_rendering_executed: false,
            live_projectile_queue_mutated: false,
        };

        assert_eq!(
            fireball_render_preview_label(preview),
            "index=0;source=projected_projectile_collision;kind=explosion;frame=explosion_1;image=graphics/SMB/fireball.png;quad=32,0,16,16;draw=(44.000000,48.000000);rotation=0.000000;scale=2.000000;live_rendering_executed=false;live_projectile_queue_mutated=false",
        );
    }

    #[test]
    fn cli_labels_player_render_preview_metadata() {
        let movement = PlayerMovementState {
            run_frame: 2,
            swim_frame: 1,
            ducking: true,
            animation_state: PlayerAnimationState::Running,
            animation_direction: HorizontalDirection::Left,
            ..PlayerMovementState::default()
        };
        let player = LegacyRuntimePlayer::new(
            PlayerBodyBounds::new(2.0, 3.0, 12.0 / 16.0, 24.0 / 16.0),
            movement,
        )
        .with_power_up(LegacyRuntimePlayerPowerUp::Fire)
        .with_fire_animation_timer(0.05);
        let quad = LegacyRuntimePlayerRenderQuad {
            x_px: 260,
            y_px: 72,
            width_px: 20,
            height_px: 36,
            atlas_width_px: 512,
            atlas_height_px: 256,
        };
        let color_layers = [
            LegacyRuntimePlayerRenderColorLayerPreview {
                draw_order: 0,
                graphic_layer_index: 1,
                image_path: "graphics/SMB/player/bigmarioanimations1.png",
                tint: LegacyRuntimePlayerRenderTint {
                    r: 252.0 / 255.0,
                    g: 216.0 / 255.0,
                    b: 168.0 / 255.0,
                },
                tint_source: LegacyRuntimePlayerRenderTintSource::FlowerColor,
                quad,
                draw_x_px: 36.0,
                draw_y_px: 102.0,
                rotation: 0.0,
                scale: 2.0,
                live_rendering_executed: false,
            },
            LegacyRuntimePlayerRenderColorLayerPreview {
                draw_order: 1,
                graphic_layer_index: 2,
                image_path: "graphics/SMB/player/bigmarioanimations2.png",
                tint: LegacyRuntimePlayerRenderTint {
                    r: 216.0 / 255.0,
                    g: 40.0 / 255.0,
                    b: 0.0,
                },
                tint_source: LegacyRuntimePlayerRenderTintSource::FlowerColor,
                quad,
                draw_x_px: 36.0,
                draw_y_px: 102.0,
                rotation: 0.0,
                scale: 2.0,
                live_rendering_executed: false,
            },
            LegacyRuntimePlayerRenderColorLayerPreview {
                draw_order: 2,
                graphic_layer_index: 3,
                image_path: "graphics/SMB/player/bigmarioanimations3.png",
                tint: LegacyRuntimePlayerRenderTint {
                    r: 252.0 / 255.0,
                    g: 152.0 / 255.0,
                    b: 56.0 / 255.0,
                },
                tint_source: LegacyRuntimePlayerRenderTintSource::FlowerColor,
                quad,
                draw_x_px: 36.0,
                draw_y_px: 102.0,
                rotation: 0.0,
                scale: 2.0,
                live_rendering_executed: false,
            },
            LegacyRuntimePlayerRenderColorLayerPreview {
                draw_order: 3,
                graphic_layer_index: 0,
                image_path: "graphics/SMB/player/bigmarioanimations0.png",
                tint: LegacyRuntimePlayerRenderTint {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                tint_source: LegacyRuntimePlayerRenderTintSource::White,
                quad,
                draw_x_px: 36.0,
                draw_y_px: 102.0,
                rotation: 0.0,
                scale: 2.0,
                live_rendering_executed: false,
            },
        ];
        let empty_hat_draw = LegacyRuntimePlayerRenderHatPreview {
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
            direction_scale: 0.0,
            vertical_scale: 0.0,
            live_rendering_executed: false,
        };
        let hat_draws = [
            LegacyRuntimePlayerRenderHatPreview {
                drawn: true,
                draw_order: 0,
                hat_slot_index: 0,
                hat_id: 1,
                size: LegacyRuntimePlayerRenderHatSize::Big,
                image_path: "graphics/SMB/bighats/standard.png",
                tint: LegacyRuntimePlayerRenderTint {
                    r: 252.0 / 255.0,
                    g: 216.0 / 255.0,
                    b: 168.0 / 255.0,
                },
                tint_source: LegacyRuntimePlayerRenderTintSource::FlowerColor,
                hat_config_x_px: 0,
                hat_config_y_px: 0,
                hat_height_px: 4,
                offset_x_px: -5,
                offset_y_px: -4,
                stack_y_px: 0,
                follows_graphic_layer_index: 3,
                precedes_graphic_layer_index: 0,
                draw_x_px: 36.0,
                draw_y_px: 102.0,
                origin_x_px: 4,
                origin_y_px: 16,
                rotation: 0.0,
                direction_scale: -2.0,
                vertical_scale: 2.0,
                live_rendering_executed: false,
            },
            empty_hat_draw,
            empty_hat_draw,
            empty_hat_draw,
        ];
        let preview = LegacyRuntimePlayerRenderIntentPreview {
            player_index: 0,
            player,
            body: player.body,
            facing: HorizontalDirection::Left,
            animation_state: PlayerAnimationState::Running,
            render_frame: LegacyRuntimePlayerRenderFrame::BigDuck,
            run_frame: 2,
            swim_frame: 1,
            size: 3,
            power_up: LegacyRuntimePlayerPowerUp::Fire,
            ducking: true,
            fire_animation_timer: 0.05,
            fire_animation_active: true,
            image_path: "graphics/SMB/player/bigmarioanimations.png",
            quad,
            color_layers,
            hat_draw_count: 1,
            hat_draws,
            draw_x_px: 36.0,
            draw_y_px: 102.0,
            rotation: 0.0,
            scale: 2.0,
            live_rendering_executed: false,
            live_player_mutated: false,
        };

        assert_eq!(
            player_render_preview_label(preview),
            "index=0;power_up=fire;size=3;render_frame=big_duck;animation_state=Running;facing=left;run_frame=2;swim_frame=1;ducking=true;fire_animation_timer=0.050000;fire_animation_active=true;image=graphics/SMB/player/bigmarioanimations.png;quad=260,72,20,36@512x256;color_layers=order=0,layer=1,image=graphics/SMB/player/bigmarioanimations1.png,tint=0.988235/0.847059/0.658824,tint_source=flower_color,quad=260,72,20,36@512x256,draw=(36.000000,102.000000),rotation=0.000000,scale=2.000000,live_rendering_executed=false|order=1,layer=2,image=graphics/SMB/player/bigmarioanimations2.png,tint=0.847059/0.156863/0.000000,tint_source=flower_color,quad=260,72,20,36@512x256,draw=(36.000000,102.000000),rotation=0.000000,scale=2.000000,live_rendering_executed=false|order=2,layer=3,image=graphics/SMB/player/bigmarioanimations3.png,tint=0.988235/0.596078/0.219608,tint_source=flower_color,quad=260,72,20,36@512x256,draw=(36.000000,102.000000),rotation=0.000000,scale=2.000000,live_rendering_executed=false|order=3,layer=0,image=graphics/SMB/player/bigmarioanimations0.png,tint=1.000000/1.000000/1.000000,tint_source=white,quad=260,72,20,36@512x256,draw=(36.000000,102.000000),rotation=0.000000,scale=2.000000,live_rendering_executed=false;hat_draws=order=0,slot=0,hat=1,size=big,image=graphics/SMB/bighats/standard.png,tint=0.988235/0.847059/0.658824,tint_source=flower_color,config=(0,0,4),offset=(-5,-4),stack_y=0,after_layer=3,before_layer=0,draw=(36.000000,102.000000),origin=(4,16),rotation=0.000000,direction_scale=-2.000000,vertical_scale=2.000000,live_rendering_executed=false;draw=(36.000000,102.000000);rotation=0.000000;scale=2.000000;live_rendering_executed=false;live_player_mutated=false",
        );
    }

    #[test]
    fn cli_labels_counter_sources_for_structured_report_details() {
        assert_eq!(
            player_coin_pickup_label(LegacyRuntimePlayerCoinPickup {
                coord: LegacyMapTileCoord::new(2, 4),
                tile_id: iw2wth_core::TileId(3),
                clear_tile_id: iw2wth_core::TileId(1),
                score_delta: 200,
                sound: LegacySoundEffect::Coin,
            }),
            "pickup_tile=(2,4);tile_id=3;clear_tile=(2,4);clear_tile_id=1;score_delta=200;sound=coinsound",
        );
        assert_eq!(
            audio_command_sequence_label(&[
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ]),
            "stop:coinsound|play:coinsound",
        );
        assert_eq!(
            audio_command_sequence_detail_label(&[
                LegacyAudioCommand::StopSound(LegacySoundEffect::Coin),
                LegacyAudioCommand::PlaySound(LegacySoundEffect::Coin),
            ]),
            "0:stop:coinsound|1:play:coinsound",
        );
        assert_eq!(sound_effect_label(LegacySoundEffect::Coin), "coinsound");
        assert_eq!(
            portal_outcome_intent_label(LegacyRuntimePortalOutcomeIntent {
                requested_slot: LegacyRuntimePortalSlot::Portal2,
                kind: LegacyRuntimePortalOutcomeKind::Fizzle,
                placement: None,
                sound: LegacySoundEffect::PortalFizzle,
            }),
            "slot=portal2;kind=fizzle;placement_tile=none;placement_side=none;sound=portalfizzlesound",
        );
        assert_eq!(
            portal_transit_audio_intent_label(LegacyRuntimePortalTransitAudioIntent {
                outcome_kind: LegacyRuntimePortalTransitOutcomeKind::BlockedExitBouncePreview,
                entry_slot: LegacyRuntimePortalSlot::Portal1,
                exit_slot: LegacyRuntimePortalSlot::Portal2,
                sound: LegacySoundEffect::PortalEnter,
            }),
            "outcome=blocked_exit_bounce_preview;entry_slot=portal1;exit_slot=portal2;sound=portalentersound",
        );
        assert_eq!(
            coin_counter_source_label(LegacyRuntimeCoinCounterSource::TopCoinCollection {
                block_coord: LegacyMapTileCoord::new(2, 3),
                coin_coord: LegacyMapTileCoord::new(2, 2),
            }),
            "top_coin_collection:block=(2,3);coin=(2,2)",
        );
        assert_eq!(
            score_source_label(LegacyRuntimeScoreSource::EnemyShotRequest {
                block_coord: LegacyMapTileCoord::new(2, 3),
                enemy_index: 9,
            }),
            "enemy_shot_request:block=(2,3);enemy_index=9",
        );
        assert_eq!(
            score_source_label(LegacyRuntimeScoreSource::FireballCollisionProbe {
                projectile_index: 2,
                source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                axis: LegacyRuntimeFireballCollisionAxis::Passive,
                target: LegacyFireballCollisionTarget::Goomba,
            }),
            "fireball_collision_probe:index=2;source=explicit_request;axis=passive;target=goomba",
        );
        assert_eq!(
            scrolling_score_label(LegacyScrollingScoreLabel::Points(100)),
            "100"
        );
        assert_eq!(
            scrolling_score_label(LegacyScrollingScoreLabel::OneUp),
            "1up"
        );
    }

    #[test]
    fn cli_labels_effect_animation_details_for_structured_reports() {
        let score = LegacyCoinBlockAnimationScore {
            score_delta: 0,
            floating_score: 200,
            x: 1.5,
            y: 2.5,
        };
        let scrolling_score = LegacyScrollingScoreState::spawn(
            LegacyScrollingScoreLabel::Points(200),
            1.5,
            2.5,
            0.25,
        );
        assert_eq!(
            coin_block_animation_score_label(Some(score)),
            "delta=0;floating=200;x=1.500000;y=2.500000",
        );
        assert_eq!(
            optional_scrolling_score_state_label(Some(scrolling_score)),
            "label=200;x=1.250000;y=2.500000;timer=0.000000",
        );
        assert_eq!(
            coin_block_animation_update_label(LegacyRuntimeCoinBlockAnimationUpdateReport {
                index: 0,
                state: iw2wth_core::LegacyCoinBlockAnimationState {
                    x: 1.5,
                    y: 2.5,
                    timer: 0.01,
                    frame: 31,
                },
                remove: true,
                score: Some(score),
                scrolling_score: Some(scrolling_score),
            }),
            "index=0;x=1.500000;y=2.500000;timer=0.010000;frame=31;remove=true;score=delta=0;floating=200;x=1.500000;y=2.500000;scrolling_score=label=200;x=1.250000;y=2.500000;timer=0.000000",
        );

        let debris_report = LegacyRuntimeBlockDebrisAnimationUpdateReport {
            index: 3,
            source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                coord: LegacyMapTileCoord::new(2, 3),
            },
            debris_index: 3,
            state: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 15.25, -3.5, 2.0),
            remove: true,
        };
        assert_eq!(
            block_debris_animation_update_label(debris_report),
            "index=3;source=empty_breakable_block_destroy:(2,3);debris_index=3;x=1.500000;y=15.250000;speed_x=-3.500000;speed_y=2.000000;timer=0.000000;frame=1;remove=true",
        );

        assert_eq!(
            scrolling_score_animation_update_label(
                LegacyRuntimeScrollingScoreAnimationUpdateReport {
                    index: 1,
                    source: LegacyRuntimeScoreSource::EnemyShotRequest {
                        block_coord: LegacyMapTileCoord::new(2, 3),
                        enemy_index: 9,
                    },
                    state: LegacyScrollingScoreState {
                        x: 1.25,
                        y: 2.5,
                        label: LegacyScrollingScoreLabel::Points(100),
                        timer: 0.401,
                    },
                    presentation: iw2wth_core::LegacyScrollingScorePresentation {
                        x: 1.0,
                        y: 1.25,
                        label: LegacyScrollingScoreLabel::Points(100),
                    },
                    remove: true,
                }
            ),
            "index=1;source=enemy_shot_request:block=(2,3);enemy_index=9;state=label=100;x=1.250000;y=2.500000;timer=0.401000;presentation_x=1.000000;presentation_y=1.250000;remove=true",
        );
        assert_eq!(
            scrolling_score_animation_update_label(
                LegacyRuntimeScrollingScoreAnimationUpdateReport {
                    index: 0,
                    source: LegacyRuntimeScoreSource::FireballCollisionProbe {
                        projectile_index: 2,
                        source: LegacyRuntimeFireballCollisionProbeSource::ExplicitRequest,
                        axis: LegacyRuntimeFireballCollisionAxis::Passive,
                        target: LegacyFireballCollisionTarget::Goomba,
                    },
                    state: LegacyScrollingScoreState {
                        x: 1.25,
                        y: 2.5,
                        label: LegacyScrollingScoreLabel::Points(100),
                        timer: 0.011,
                    },
                    presentation: iw2wth_core::LegacyScrollingScorePresentation {
                        x: 1.0,
                        y: 1.25,
                        label: LegacyScrollingScoreLabel::Points(100),
                    },
                    remove: false,
                }
            ),
            "index=0;source=fireball_collision_probe:index=2;source=explicit_request;axis=passive;target=goomba;state=label=100;x=1.250000;y=2.500000;timer=0.011000;presentation_x=1.000000;presentation_y=1.250000;remove=false",
        );
        assert_eq!(
            optional_frame_detail_label(Some((
                12,
                block_debris_animation_update_label(debris_report),
            ))),
            "frame=12;index=3;source=empty_breakable_block_destroy:(2,3);debris_index=3;x=1.500000;y=15.250000;speed_x=-3.500000;speed_y=2.000000;timer=0.000000;frame=1;remove=true",
        );
        assert_eq!(optional_frame_detail_label(None), "none");
    }

    #[test]
    fn cli_labels_projection_details_for_structured_reports() {
        let tile_change = LegacyRuntimeTileChangeProjection {
            source: LegacyRuntimeTileChangeSource::TopCoinCollection {
                block_coord: LegacyMapTileCoord::new(2, 3),
                coin_coord: LegacyMapTileCoord::new(2, 2),
            },
            tile_change: iw2wth_core::LegacyTileChange {
                coord: TileCoord::new(2, 2),
                tile: iw2wth_core::TileId(1),
            },
        };
        assert_eq!(
            tile_change_projection_label(tile_change),
            "top_coin_collection:block=(2,3);coin=(2,2)->(2,2):1",
        );
        assert_eq!(
            tile_change_projections_label(&[tile_change]),
            "[top_coin_collection:block=(2,3);coin=(2,2)->(2,2):1]",
        );

        let cleanup = LegacyRuntimeBreakableBlockCleanupProjection {
            source: LegacyRuntimeBreakableBlockCleanupSource::EmptyBreakableBlockDestroy {
                coord: LegacyMapTileCoord::new(2, 3),
            },
            action: LegacyRuntimeBreakableBlockCleanupAction::SpawnDebris {
                index: 0,
                debris: iw2wth_core::LegacyBlockDebrisState::spawn(1.5, 2.5, 3.5, -23.0),
            },
        };
        assert_eq!(
            breakable_cleanup_projection_label(cleanup),
            "empty_breakable_block_destroy:(2,3)->spawn_debris:index=0;x=1.500000;y=2.500000;speed_x=3.500000;speed_y=-23.000000",
        );
        assert_eq!(
            breakable_cleanup_projections_label(&[cleanup]),
            "[empty_breakable_block_destroy:(2,3)->spawn_debris:index=0;x=1.500000;y=2.500000;speed_x=3.500000;speed_y=-23.000000]",
        );
    }

    #[test]
    fn cli_labels_player_tile_collision_details_for_structured_reports() {
        let horizontal = LegacyRuntimePlayerTileCollision {
            coord: LegacyMapTileCoord::new(2, 4),
            tile_id: iw2wth_core::TileId(7),
            axis: LegacyRuntimePlayerCollisionAxis::Horizontal,
        };
        assert_eq!(
            player_tile_collision_label(horizontal),
            "tile=(2,4);tile_id=7;axis=horizontal",
        );
        assert_eq!(
            optional_frame_detail_label(Some((3, player_tile_collision_label(horizontal)))),
            "frame=3;tile=(2,4);tile_id=7;axis=horizontal",
        );

        assert_eq!(
            ceiling_block_hit_label(LegacyRuntimePlayerCeilingBlockHit {
                coord: LegacyMapTileCoord::new(2, 3),
                tile_id: iw2wth_core::TileId(113),
                breakable: true,
                coin_block: true,
                play_hit_sound: true,
                portal_guard: None,
            }),
            "tile=(2,3);tile_id=113;breakable=true;coin_block=true;play_hit_sound=true;portal_guard=none",
        );
    }

    #[test]
    fn cli_labels_block_bounce_and_reward_reveal_details_for_structured_reports() {
        let schedule = LegacyRuntimePlayerBlockBounceSchedule {
            coord: LegacyMapTileCoord::new(2, 3),
            schedule: iw2wth_core::LegacyBlockBounceSchedule {
                timer: 0.000_000_001,
                coord: TileCoord::new(2, 3),
                spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
        };
        assert_eq!(
            block_bounce_schedule_label(schedule),
            "block=(2,3);queue=(2,3);timer=0.000000001;spawn_content=mushroom;hitter_size=1;regenerate_sprite_batch=true",
        );

        let spawn = LegacyBlockBounceReplaySpawn {
            kind: LegacyBlockBounceReplayKind::Mushroom,
            x: 1.5,
            y: 2.875,
        };
        assert_eq!(
            block_bounce_replay_spawn_label(spawn),
            "kind=mushroom;x=1.500000;y=2.875000",
        );
        assert_eq!(
            block_bounce_completion_label(LegacyRuntimeBlockBounceCompletionReport {
                index: 0,
                coord: TileCoord::new(2, 3),
                timer: 0.21,
                remove: true,
                suppressed_replay_spawn: Some(spawn),
                item_spawn_intent: Some(LegacyRuntimeBlockBounceItemSpawnIntent {
                    source_index: 0,
                    source_coord: TileCoord::new(2, 3),
                    spawn,
                }),
            }),
            "index=0;tile=(2,3);timer=0.210000;remove=true;suppressed_replay_spawn=kind=mushroom;x=1.500000;y=2.875000",
        );
        assert_eq!(
            block_bounce_item_spawn_intent_label(LegacyRuntimeBlockBounceItemSpawnIntent {
                source_index: 0,
                source_coord: TileCoord::new(2, 3),
                spawn,
            }),
            "source_index=0;source_tile=(2,3);spawn=kind=mushroom;x=1.500000;y=2.875000",
        );
        assert_eq!(
            contained_reward_reveal_intent_label(LegacyRuntimeBlockContainedRewardRevealIntent {
                coord: LegacyMapTileCoord::new(2, 3),
                content: LegacyBlockBounceContentKind::Mushroom,
                outcome: iw2wth_core::LegacyBlockContainedRewardRevealOutcome {
                    tile_change: iw2wth_core::LegacyTileChange {
                        coord: TileCoord::new(2, 3),
                        tile: iw2wth_core::TileId(113),
                    },
                    sound: LegacyBlockRevealSound::MushroomAppear,
                },
            }),
            "block=(2,3);content=mushroom;tile=(2,3);tile_id=113;sound=mushroomappear",
        );

        let coin_reward = LegacyRuntimeCoinBlockRewardIntent {
            coord: LegacyMapTileCoord::new(2, 3),
            outcome: iw2wth_core::LegacyCoinBlockRewardOutcome {
                play_coin_sound: true,
                animation: LegacyCoinBlockAnimationState::spawn(1.5, 2.0),
                score_delta: 200,
                coin_count: 99,
                life_reward: Some(LegacyCoinLifeReward {
                    grant_lives_to_players: 1,
                    respawn_players: true,
                    play_sound: true,
                }),
                tile_change: Some(LegacyTileChange {
                    coord: TileCoord::new(2, 3),
                    tile: iw2wth_core::TileId(113),
                }),
                start_many_coins_timer: Some(LegacyCoinBlockTimerSpawn {
                    coord: TileCoord::new(2, 3),
                    duration: 4.0,
                }),
            },
        };
        assert_eq!(
            coin_block_reward_intent_label(coin_reward),
            "block=(2,3);coin_sound=true;animation=x=1.500000;y=2.000000;timer=0.000000;frame=1;score_delta=200;coin_count=99;life_reward=grant_lives=1;respawn_players=true;play_sound=true;tile_change=(2,3):113;many_coins_timer=coord=(2,3);duration=4.000000",
        );

        let top_coin = LegacyRuntimeBlockTopCoinCollectionIntent {
            block_coord: LegacyMapTileCoord::new(2, 3),
            coin_coord: LegacyMapTileCoord::new(2, 2),
            outcome: iw2wth_core::LegacyBlockTopCoinCollectionOutcome {
                tile_change: LegacyTileChange {
                    coord: TileCoord::new(2, 2),
                    tile: iw2wth_core::TileId(1),
                },
                play_coin_sound: true,
                animation: LegacyCoinBlockAnimationState::spawn(1.5, 1.0),
                score_delta: 200,
                coin_count: 0,
                life_reward: None,
            },
        };
        assert_eq!(
            top_coin_collection_intent_label(top_coin),
            "block=(2,3);coin=(2,2);coin_sound=true;animation=x=1.500000;y=1.000000;timer=0.000000;frame=1;score_delta=200;coin_count=0;life_reward=none;tile_change=(2,2):1",
        );
    }

    #[test]
    fn parse_args_seeds_block_hit_adapter_snapshots() {
        let (_, config) = parse_args(vec![
            "--seed-jump-item".to_owned(),
            "mushroom,9,0.5,1.5,1,0.5,true".to_owned(),
            "--seed-jump-item".to_owned(),
            "one-up,10,2.5,1.5,1,0.5,false".to_owned(),
            "--seed-top-enemy".to_owned(),
            "11,0.5,1.5,1,0.5,true".to_owned(),
            "--seed-fireball-enemy".to_owned(),
            "goomba,12,3.5,4,1,1,true".to_owned(),
            "--seed-many-coins-timer".to_owned(),
            "2,3,0.005".to_owned(),
            "--seed-many-coins-timer".to_owned(),
            "2,3,-0.25".to_owned(),
        ])
        .expect("block-hit snapshot seed flags should parse");

        assert_eq!(
            config.jump_items,
            vec![
                LegacyRuntimeBlockJumpItemSnapshot::new(
                    LegacyBlockJumpItemKind::Mushroom,
                    9,
                    0.5,
                    1.5,
                    1.0,
                    0.5,
                    true,
                ),
                LegacyRuntimeBlockJumpItemSnapshot::new(
                    LegacyBlockJumpItemKind::OneUp,
                    10,
                    2.5,
                    1.5,
                    1.0,
                    0.5,
                    false,
                ),
            ],
        );
        assert_eq!(
            config.top_enemies,
            vec![LegacyRuntimeBlockTopEnemySnapshot::new(
                11, 0.5, 1.5, 1.0, 0.5, true,
            )],
        );
        assert_eq!(
            config.fireball_enemies,
            vec![LegacyRuntimeFireballEnemySnapshot::new(
                LegacyFireballCollisionTarget::Goomba,
                12,
                3.5,
                4.0,
                1.0,
                1.0,
                true,
            )],
        );
        assert_eq!(
            config.many_coins_timers,
            vec![
                LegacyManyCoinsTimerEntry {
                    coord: TileCoord::new(2, 3),
                    remaining: 0.005,
                },
                LegacyManyCoinsTimerEntry {
                    coord: TileCoord::new(2, 3),
                    remaining: -0.25,
                },
            ],
        );
    }
}

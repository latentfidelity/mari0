//! Engine-neutral gameplay core for the Lua-to-Rust conversion.
//!
//! This crate starts intentionally small. Code only moves here when it can be
//! expressed as deterministic data and rules without direct rendering, audio,
//! input, filesystem, or windowing calls.

pub mod block;
pub mod collision;
pub mod config;
pub mod content;
pub mod effects;
pub mod enemy;
pub mod hazard;
pub mod item;
pub mod level;
pub mod math;
pub mod player;
pub mod projectile;
pub mod spring;
pub mod vine;
pub mod wormhole;

pub use block::{
    LEGACY_BLOCK_BOUNCE_DURATION, LEGACY_BLOCK_BOUNCE_TIMER_START,
    LegacyBlockBounceCompletionUpdate, LegacyBlockBounceContentKind, LegacyBlockBounceContext,
    LegacyBlockBounceQueuePruneUpdate, LegacyBlockBounceReplayKind, LegacyBlockBounceReplaySpawn,
    LegacyBlockBounceSchedule, LegacyBlockBounceSpawnKind, LegacyBlockContainedRewardRevealContext,
    LegacyBlockContainedRewardRevealOutcome, LegacyBlockEnemyShotRequest,
    LegacyBlockHitSoundContext, LegacyBlockItemJumpRequest, LegacyBlockJumpItem,
    LegacyBlockJumpItemKind, LegacyBlockPortalReservation, LegacyBlockRevealSound,
    LegacyBlockSpriteset, LegacyBlockTopCoinCollectionContext, LegacyBlockTopCoinCollectionOutcome,
    LegacyBlockTopEnemy, LegacyBreakableBlockOutcome, LegacyBrokenBlockEffects,
    LegacyCoinBlockRewardContext, LegacyCoinBlockRewardKind, LegacyCoinBlockRewardOutcome,
    LegacyCoinBlockTimerSpawn, LegacyCoinLifeReward, LegacyEmptyBreakableBlockDestroyContext,
    LegacyManyCoinsTimer, LegacyManyCoinsTimerEntry, LegacyTileChange,
    legacy_block_bounce_schedule, legacy_block_contained_reward_reveal,
    legacy_block_enemy_shot_requests, legacy_block_hit_sound_requested,
    legacy_block_item_jump_requests, legacy_block_top_coin_collection, legacy_breakable_block_hit,
    legacy_coin_block_reward, legacy_empty_breakable_block_destroy, legacy_many_coins_timer_state,
    legacy_used_coin_block_tile, prune_legacy_completed_block_bounces,
    update_legacy_block_bounce_completion, update_legacy_many_coins_timer,
};
pub use collision::{
    CollisionBody, CollisionKind, LegacyCollisionActor, LegacyCollisionHandlerResult,
    LegacyCollisionResponse, LegacyCollisionTarget, LegacyPassiveCollisionResponse, collision_kind,
    in_range, legacy_horizontal_collision_response, legacy_passive_collision_response,
    legacy_vertical_collision_response, lua_aabb_overlap,
};
pub use config::{
    BlueGelBounceConstants, LegacyBlockDebrisConstants, LegacyBulletBillConstants,
    LegacyCastleFireConstants, LegacyCheepCheepConstants, LegacyCoinBlockAnimationConstants,
    LegacyCoinBlockRewardConstants, LegacyFireConstants, LegacyFireballConstants,
    LegacyFireworkConstants, LegacyFlyingFishConstants, LegacyGoombaConstants,
    LegacyHammerBroConstants, LegacyHammerConstants, LegacyKoopaConstants, LegacyLakitoConstants,
    LegacyPlantConstants, LegacyPowerUpConstants, LegacyScrollingScoreConstants,
    LegacySquidConstants, LegacyUpFireConstants, LegacyVineConstants, OrangeGelMovementConstants,
    PhysicsConstants, PlayerAnimationConstants, PlayerMovementConstants, SpringConstants,
    UnderwaterMovementConstants,
};
pub use content::{
    CellValue, LegacyBoxEntity, LegacyEnemyEntity, LegacyEntityKind, LegacyEntityPlacement,
    LegacyEntitySurface, LegacyFaithPlateEntity, LegacyGelDispenserEntity, LegacyGoalEntity,
    LegacyGroundLightEntity, LegacyHazardEntity, LegacyLaserEntity, LegacyLevelControlEntity,
    LegacyLightBridgeEntity, LegacyLinkedEntity, LegacyPlatformEntity, LegacyPowerUpEntity,
    LegacyWarpEntity, LevelCell, LevelProperties, LinkTarget, Lives, MappackSettings, Mari0Level,
    ParseError,
};
pub use effects::{
    LegacyBlockDebrisState, LegacyBlockDebrisUpdate, LegacyCoinBlockAnimationScore,
    LegacyCoinBlockAnimationState, LegacyCoinBlockAnimationUpdate, LegacyFireworkBoomSpawn,
    LegacyFireworkBoomState, LegacyFireworkBoomUpdate, LegacyFireworkFrame,
    LegacyScrollingScoreLabel, LegacyScrollingScorePresentation, LegacyScrollingScoreState,
    LegacyScrollingScoreUpdate, legacy_firework_boom_frame, legacy_scrolling_score_presentation,
    update_legacy_block_debris, update_legacy_coin_block_animation, update_legacy_firework_boom,
    update_legacy_scrolling_score,
};
pub use enemy::{
    LegacyBulletBillLauncherState, LegacyBulletBillLauncherUpdate,
    LegacyBulletBillLauncherViewport, LegacyBulletBillLifecycle, LegacyBulletBillState,
    LegacyCheepCheepColor, LegacyCheepCheepFrame, LegacyCheepCheepLifecycle, LegacyCheepCheepState,
    LegacyCheepCheepVerticalDirection, LegacyEnemyCollisionResponse, LegacyEnemyDirection,
    LegacyEnemyTileProbe, LegacyEnemyUpdate, LegacyFlyingFishFrame, LegacyFlyingFishLifecycle,
    LegacyFlyingFishState, LegacyGoombaFrame, LegacyGoombaLifecycle, LegacyGoombaState,
    LegacyGoombaVariant, LegacyHammerBroCollisionActor, LegacyHammerBroFrame,
    LegacyHammerBroJumpDecision, LegacyHammerBroJumpState, LegacyHammerBroLifecycle,
    LegacyHammerBroState, LegacyHammerBroUpdate, LegacyHammerFrame, LegacyHammerState,
    LegacyKoopaEdgeTurn, LegacyKoopaFrame, LegacyKoopaLifecycle, LegacyKoopaSideCollisionResponse,
    LegacyKoopaSideCollisionTarget, LegacyKoopaState, LegacyKoopaStompOutcome, LegacyKoopaVariant,
    LegacyLakitoLifecycle, LegacyLakitoPlayerTarget, LegacyLakitoState, LegacyLakitoUpdate,
    LegacyPlantFrame, LegacyPlantState, LegacySquidFrame, LegacySquidLifecycle, LegacySquidMotion,
    LegacySquidPlayerTarget, LegacySquidState, apply_legacy_red_koopa_edge_turn,
    emancipate_legacy_hammer_bro, fire_legacy_bullet_bill_launcher, legacy_cheep_cheep_collision,
    legacy_flying_fish_collision, legacy_goomba_left_collision, legacy_goomba_right_collision,
    legacy_hammer_bro_ceil_collision, legacy_hammer_bro_floor_collision,
    legacy_hammer_bro_left_collision, legacy_hammer_bro_right_collision, legacy_hammer_collision,
    legacy_koopa_floor_collision, legacy_koopa_left_collision, legacy_koopa_resists_fireball,
    legacy_koopa_right_collision, legacy_koopa_start_fall, legacy_lakito_collision,
    legacy_lakito_spikeyfall_collision, legacy_spikey_falling_floor_collision,
    legacy_spikey_falling_suppresses_lakito_collision, legacy_squid_collision,
    portal_legacy_bullet_bill, portal_legacy_hammer, portal_legacy_hammer_bro,
    shoot_legacy_bullet_bill, shoot_legacy_cheep_cheep, shoot_legacy_flying_fish,
    shoot_legacy_goomba, shoot_legacy_hammer_bro, shoot_legacy_koopa, shoot_legacy_lakito,
    shoot_legacy_plant, shoot_legacy_squid, stomp_legacy_bullet_bill, stomp_legacy_flying_fish,
    stomp_legacy_goomba, stomp_legacy_hammer_bro, stomp_legacy_koopa, stomp_legacy_lakito,
    update_legacy_bullet_bill, update_legacy_bullet_bill_launcher, update_legacy_cheep_cheep,
    update_legacy_flying_fish, update_legacy_goomba, update_legacy_hammer,
    update_legacy_hammer_bro_active, update_legacy_hammer_bro_shot, update_legacy_koopa,
    update_legacy_lakito, update_legacy_plant, update_legacy_squid,
};
pub use hazard::{
    LegacyCastleFireDirection, LegacyCastleFireFrame, LegacyCastleFireSegment,
    LegacyCastleFireState, LegacyCastleFireUpdate, LegacyFireCollisionResponse, LegacyFireFrame,
    LegacyFireSource, LegacyFireState, LegacyFireUpdate, LegacyUpFireCollisionResponse,
    LegacyUpFireState, LegacyUpFireUpdate, legacy_castle_fire_segments, legacy_fire_collision,
    legacy_up_fire_collision, update_legacy_castle_fire, update_legacy_fire, update_legacy_up_fire,
};
pub use item::{
    LegacyFlowerCollection, LegacyFlowerCollision, LegacyFlowerState, LegacyMushroomCollection,
    LegacyMushroomCollision, LegacyMushroomState, LegacyOneUpCollection, LegacyOneUpCollision,
    LegacyOneUpReward, LegacyOneUpState, LegacyOneUpViewport, LegacyPowerUpCollisionActor,
    LegacyPowerUpUpdate, LegacyStarCollection, LegacyStarCollision, LegacyStarState,
    apply_legacy_flower_jump, apply_legacy_mushroom_jump, apply_legacy_one_up_jump,
    apply_legacy_star_jump, legacy_flower_ceiling_collision, legacy_flower_floor_collision,
    legacy_flower_left_collision, legacy_flower_right_collision, legacy_mushroom_ceiling_collision,
    legacy_mushroom_floor_collision, legacy_mushroom_left_collision,
    legacy_mushroom_right_collision, legacy_one_up_ceiling_collision,
    legacy_one_up_floor_collision, legacy_one_up_left_collision, legacy_one_up_right_collision,
    legacy_star_ceiling_collision, legacy_star_floor_collision, legacy_star_left_collision,
    legacy_star_right_collision, update_legacy_flower, update_legacy_mushroom,
    update_legacy_one_up, update_legacy_star,
};
pub use level::{LevelGrid, TileCoord, TileId};
pub use math::{Aabb, Vec2};
pub use player::{
    HorizontalDirection, LegacyCeilingTileContext, LegacyCeilingTileResponse,
    LegacyDropVineContext, LegacyDropVineOutcome, LegacyGelKind, LegacyGrabVineContext,
    LegacyGrabVineOutcome, LegacyHeldSpringContext, LegacyHeldSpringUpdate, LegacyMapBounds,
    LegacyMapTileCoord, LegacyOnVineAttachmentLossContext, LegacyOnVineContext,
    LegacyOnVineDirection, LegacyOnVineHorizontalPortalContext,
    LegacyOnVineHorizontalPortalOutcome, LegacyOnVineHorizontalPortalPair, LegacyOnVineUpdate,
    LegacyPipeCandidate, LegacyPipeDirection, LegacyPipeEntry, LegacyPlayerCollisionSnapshot,
    LegacyPlayerDeathCause, LegacyPlayerHazard, LegacyPlayerHazardCollision,
    LegacyPlayerHazardOutcome, LegacySideBoxResponse, LegacySurfaceMovementContext, LegacyVineSide,
    PlayerAnimationState, PlayerBodyBounds, PlayerEnvironment, PlayerMovementInput,
    PlayerMovementState, PlayerSideCollision, PlayerVerticalBounds,
    advance_legacy_player_animation, advance_legacy_underwater_animation, apply_legacy_drop_vine,
    apply_legacy_enemy_stomp_bounce_state, apply_legacy_faithplate_state,
    apply_legacy_floor_blue_gel_bounce, apply_legacy_floor_invisible_tile_suppression,
    apply_legacy_floor_landing_state, apply_legacy_grab_vine, apply_legacy_head_bump_state,
    apply_legacy_hit_spring_state, apply_legacy_leave_spring_state,
    apply_legacy_non_invisible_ceiling_tile_response, apply_legacy_on_vine_attachment_loss,
    apply_legacy_on_vine_horizontal_portal, apply_legacy_player_gravity_selection,
    apply_legacy_player_gravity_velocity, apply_legacy_player_movement,
    apply_legacy_player_movement_with_surface_query, apply_legacy_side_blue_gel_bounce,
    apply_legacy_side_box_response, apply_legacy_side_button_response,
    apply_legacy_side_tile_gap_run, apply_legacy_spring_high_request,
    apply_legacy_start_fall_after_vertical_move, apply_legacy_underwater_jump,
    apply_legacy_underwater_movement, legacy_ceiling_invisible_tile_suppresses_default,
    legacy_enemy_stomp_bounce_speed, legacy_jump_velocity, legacy_leave_spring_y,
    legacy_player_hazard_collision_outcome, legacy_right_side_pipe_entry,
    legacy_side_invisible_tile_suppresses_default, legacy_surface_movement_constants,
    legacy_top_ground_probe, legacy_underwater_jump_velocity, stop_legacy_jump, try_legacy_jump,
    update_legacy_held_spring, update_legacy_on_vine_motion,
};
pub use projectile::{
    LegacyFireballCollisionOutcome, LegacyFireballCollisionTarget, LegacyFireballExplosion,
    LegacyFireballFrame, LegacyFireballState, LegacyFireballUpdate, LegacyFireballViewport,
    explode_legacy_fireball, legacy_fireball_ceil_collision, legacy_fireball_floor_collision,
    legacy_fireball_left_collision, legacy_fireball_passive_collision,
    legacy_fireball_right_collision, update_legacy_fireball,
};
pub use spring::{LegacySpringState, apply_legacy_spring_hit, update_legacy_spring};
pub use vine::{LegacyVineState, LegacyVineVariant, update_legacy_vine};
pub use wormhole::{
    AnimationDirection, Facing, LegacyPortalEndpoint, LegacyPortalTransit,
    LegacyPortalTransitInput, WormholeEndpoint, WormholePair, WormholeTransit,
    legacy_portal_coords,
};

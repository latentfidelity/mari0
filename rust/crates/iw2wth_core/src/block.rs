//! Engine-neutral block reward side-effect contracts ported from `mario.lua`.

use crate::{
    collision::in_range,
    config::LegacyCoinBlockRewardConstants,
    effects::{LegacyBlockDebrisState, LegacyCoinBlockAnimationState},
    enemy::LegacyEnemyDirection,
    level::{TileCoord, TileId},
    wormhole::Facing,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockSpriteset {
    One,
    Two,
    Other,
}

impl LegacyBlockSpriteset {
    #[must_use]
    pub const fn from_legacy_index(index: u8) -> Self {
        match index {
            1 => Self::One,
            2 => Self::Two,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyCoinBlockRewardKind {
    Single { invisible: bool },
    ManyCoins { timer: LegacyManyCoinsTimer },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyManyCoinsTimer {
    Missing,
    Existing { remaining: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyManyCoinsTimerEntry {
    pub coord: TileCoord,
    pub remaining: f32,
}

#[must_use]
pub fn update_legacy_many_coins_timer(remaining: f32, dt: f32) -> f32 {
    if remaining > 0.0 {
        remaining - dt
    } else {
        remaining
    }
}

#[must_use]
pub fn legacy_many_coins_timer_state(
    coord: TileCoord,
    timers: &[LegacyManyCoinsTimerEntry],
) -> LegacyManyCoinsTimer {
    timers
        .iter()
        .rev()
        .find(|timer| timer.coord == coord)
        .map_or(LegacyManyCoinsTimer::Missing, |timer| {
            LegacyManyCoinsTimer::Existing {
                remaining: timer.remaining,
            }
        })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockRewardContext {
    pub coord: TileCoord,
    pub spriteset: LegacyBlockSpriteset,
    pub kind: LegacyCoinBlockRewardKind,
    pub coin_count: u32,
    pub life_count_enabled: bool,
    pub player_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyCoinLifeReward {
    pub grant_lives_to_players: usize,
    pub respawn_players: bool,
    pub play_sound: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockTimerSpawn {
    pub coord: TileCoord,
    pub duration: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyTileChange {
    pub coord: TileCoord,
    pub tile: TileId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockRewardOutcome {
    pub play_coin_sound: bool,
    pub animation: LegacyCoinBlockAnimationState,
    pub score_delta: u32,
    pub coin_count: u32,
    pub life_reward: Option<LegacyCoinLifeReward>,
    pub tile_change: Option<LegacyTileChange>,
    pub start_many_coins_timer: Option<LegacyCoinBlockTimerSpawn>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockTopCoinCollectionContext {
    pub top_coin_coord: Option<TileCoord>,
    pub coin_count: u32,
    pub life_count_enabled: bool,
    pub player_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockTopCoinCollectionOutcome {
    pub tile_change: LegacyTileChange,
    pub play_coin_sound: bool,
    pub animation: LegacyCoinBlockAnimationState,
    pub score_delta: u32,
    pub coin_count: u32,
    pub life_reward: Option<LegacyCoinLifeReward>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyBlockPortalReservation {
    pub coord: TileCoord,
    pub facing: Facing,
}

impl LegacyBlockPortalReservation {
    #[must_use]
    pub const fn new(coord: TileCoord, facing: Facing) -> Self {
        Self { coord, facing }
    }

    #[must_use]
    pub const fn adjacent_coord(self) -> TileCoord {
        match self.facing {
            Facing::Up => TileCoord::new(self.coord.x, self.coord.y - 1),
            Facing::Right => TileCoord::new(self.coord.x + 1, self.coord.y),
            Facing::Down => TileCoord::new(self.coord.x, self.coord.y + 1),
            Facing::Left => TileCoord::new(self.coord.x - 1, self.coord.y),
        }
    }

    #[must_use]
    pub const fn protects(self, coord: TileCoord) -> bool {
        coord.x == self.coord.x && coord.y == self.coord.y || {
            let adjacent = self.adjacent_coord();
            coord.x == adjacent.x && coord.y == adjacent.y
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LegacyBreakableBlockOutcome {
    ProtectedByPortal,
    Broken(LegacyBrokenBlockEffects),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyBrokenBlockEffects {
    pub tile_change: LegacyTileChange,
    pub remove_tile_collision_object: bool,
    pub clear_gels: bool,
    pub play_break_sound: bool,
    pub score_delta: u32,
    pub debris: [LegacyBlockDebrisState; 4],
    pub regenerate_sprite_batch: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockJumpItemKind {
    Mushroom,
    OneUp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockJumpItem {
    pub kind: LegacyBlockJumpItemKind,
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub has_jump_handler: bool,
}

impl LegacyBlockJumpItem {
    #[must_use]
    pub fn center_x(self) -> f32 {
        self.x + self.width / 2.0
    }

    #[must_use]
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockItemJumpRequest {
    pub kind: LegacyBlockJumpItemKind,
    pub index: usize,
    pub source_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockTopEnemy {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub has_shotted_handler: bool,
}

impl LegacyBlockTopEnemy {
    #[must_use]
    pub fn center_x(self) -> f32 {
        self.x + self.width / 2.0
    }

    #[must_use]
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockEnemyShotRequest {
    pub index: usize,
    pub direction: LegacyEnemyDirection,
    pub score_delta: u32,
    pub score_x: f32,
    pub score_y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyBlockHitSoundContext {
    pub blocked_by_portal_guard: bool,
    pub editor_mode: bool,
    pub in_map: bool,
}

pub const LEGACY_BLOCK_BOUNCE_TIMER_START: f32 = 0.000_000_001;
pub const LEGACY_BLOCK_BOUNCE_DURATION: f32 = 0.2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockBounceContentKind {
    Mushroom,
    OneUp,
    Star,
    ManyCoins,
    Vine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockBounceContext {
    pub coord: TileCoord,
    pub hitter_size: u8,
    pub content: Option<LegacyBlockBounceContentKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockBounceSpawnKind {
    Mushroom,
    OneUp,
    Star,
    Vine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockBounceSchedule {
    pub timer: f32,
    pub coord: TileCoord,
    pub spawn_content: Option<LegacyBlockBounceSpawnKind>,
    pub hitter_size: u8,
    pub regenerate_sprite_batch: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockBounceReplayKind {
    Mushroom,
    Flower,
    OneUp,
    Star,
    Vine,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockBounceReplaySpawn {
    pub kind: LegacyBlockBounceReplayKind,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockBounceCompletionUpdate {
    pub remove: bool,
    pub replay_spawn: Option<LegacyBlockBounceReplaySpawn>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyBlockBounceQueuePruneUpdate {
    pub regenerate_sprite_batch: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockContainedRewardRevealContext {
    pub coord: TileCoord,
    pub spriteset: LegacyBlockSpriteset,
    pub invisible: bool,
    pub content: Option<LegacyBlockBounceContentKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBlockRevealSound {
    MushroomAppear,
    Vine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyBlockContainedRewardRevealOutcome {
    pub tile_change: LegacyTileChange,
    pub sound: LegacyBlockRevealSound,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyEmptyBreakableBlockDestroyContext {
    pub coord: TileCoord,
    pub hitter_size: Option<u8>,
    pub is_coin_block: bool,
    pub content: Option<LegacyBlockBounceContentKind>,
}

#[must_use]
pub fn legacy_coin_block_reward(
    context: LegacyCoinBlockRewardContext,
    constants: LegacyCoinBlockRewardConstants,
) -> LegacyCoinBlockRewardOutcome {
    let incremented_coin_count = context.coin_count + 1;
    let (coin_count, life_reward) = legacy_coin_count_after_reward(
        incremented_coin_count,
        context.life_count_enabled,
        context.player_count,
        constants,
    );

    let tile_change = match context.kind {
        LegacyCoinBlockRewardKind::Single { invisible } => Some(LegacyTileChange {
            coord: context.coord,
            tile: legacy_used_coin_block_tile(context.spriteset, invisible),
        }),
        LegacyCoinBlockRewardKind::ManyCoins {
            timer: LegacyManyCoinsTimer::Existing { remaining },
        } if remaining <= 0.0 => Some(LegacyTileChange {
            coord: context.coord,
            tile: legacy_used_coin_block_tile(context.spriteset, false),
        }),
        LegacyCoinBlockRewardKind::ManyCoins { .. } => None,
    };

    let start_many_coins_timer = match context.kind {
        LegacyCoinBlockRewardKind::ManyCoins {
            timer: LegacyManyCoinsTimer::Missing,
        } => Some(LegacyCoinBlockTimerSpawn {
            coord: context.coord,
            duration: constants.many_coins_timer_duration,
        }),
        LegacyCoinBlockRewardKind::Single { .. }
        | LegacyCoinBlockRewardKind::ManyCoins {
            timer: LegacyManyCoinsTimer::Existing { .. },
        } => None,
    };

    LegacyCoinBlockRewardOutcome {
        play_coin_sound: true,
        animation: LegacyCoinBlockAnimationState::spawn(
            context.coord.x as f32 - 0.5,
            context.coord.y as f32 - 1.0,
        ),
        score_delta: constants.score_delta,
        coin_count,
        life_reward,
        tile_change,
        start_many_coins_timer,
    }
}

#[must_use]
pub fn legacy_block_top_coin_collection(
    context: LegacyBlockTopCoinCollectionContext,
    constants: LegacyCoinBlockRewardConstants,
) -> Option<LegacyBlockTopCoinCollectionOutcome> {
    let coord = context.top_coin_coord?;
    let (coin_count, life_reward) = legacy_coin_count_after_reward(
        context.coin_count + 1,
        context.life_count_enabled,
        context.player_count,
        constants,
    );

    Some(LegacyBlockTopCoinCollectionOutcome {
        tile_change: LegacyTileChange {
            coord,
            tile: TileId(1),
        },
        play_coin_sound: true,
        animation: LegacyCoinBlockAnimationState::spawn(coord.x as f32 - 0.5, coord.y as f32),
        score_delta: constants.score_delta,
        coin_count,
        life_reward,
    })
}

#[must_use]
pub fn legacy_block_bounce_schedule(
    context: LegacyBlockBounceContext,
) -> LegacyBlockBounceSchedule {
    let spawn_content = match context.content {
        Some(LegacyBlockBounceContentKind::Mushroom) => Some(LegacyBlockBounceSpawnKind::Mushroom),
        Some(LegacyBlockBounceContentKind::OneUp) => Some(LegacyBlockBounceSpawnKind::OneUp),
        Some(LegacyBlockBounceContentKind::Star) => Some(LegacyBlockBounceSpawnKind::Star),
        Some(LegacyBlockBounceContentKind::Vine) => Some(LegacyBlockBounceSpawnKind::Vine),
        Some(LegacyBlockBounceContentKind::ManyCoins) | None => None,
    };

    LegacyBlockBounceSchedule {
        timer: LEGACY_BLOCK_BOUNCE_TIMER_START,
        coord: context.coord,
        spawn_content,
        hitter_size: context.hitter_size,
        regenerate_sprite_batch: true,
    }
}

#[must_use]
pub fn legacy_block_contained_reward_reveal(
    context: LegacyBlockContainedRewardRevealContext,
) -> Option<LegacyBlockContainedRewardRevealOutcome> {
    let content = context.content?;
    if content == LegacyBlockBounceContentKind::ManyCoins {
        return None;
    }

    Some(LegacyBlockContainedRewardRevealOutcome {
        tile_change: LegacyTileChange {
            coord: context.coord,
            tile: legacy_used_coin_block_tile(context.spriteset, context.invisible),
        },
        sound: if content == LegacyBlockBounceContentKind::Vine {
            LegacyBlockRevealSound::Vine
        } else {
            LegacyBlockRevealSound::MushroomAppear
        },
    })
}

#[must_use]
pub fn legacy_empty_breakable_block_destroy(
    context: LegacyEmptyBreakableBlockDestroyContext,
    portal_reservations: &[LegacyBlockPortalReservation],
) -> Option<LegacyBreakableBlockOutcome> {
    let hitter_size = context.hitter_size?;

    if hitter_size <= 1 || context.is_coin_block || context.content.is_some() {
        return None;
    }

    Some(legacy_breakable_block_hit(
        context.coord,
        portal_reservations,
    ))
}

#[must_use]
pub fn legacy_breakable_block_hit(
    coord: TileCoord,
    portal_reservations: &[LegacyBlockPortalReservation],
) -> LegacyBreakableBlockOutcome {
    if portal_reservations
        .iter()
        .any(|reservation| reservation.protects(coord))
    {
        return LegacyBreakableBlockOutcome::ProtectedByPortal;
    }

    let x = coord.x as f32;
    let y = coord.y as f32;
    let debris_x = x - 0.5;
    let debris_y = y - 0.5;

    LegacyBreakableBlockOutcome::Broken(LegacyBrokenBlockEffects {
        tile_change: LegacyTileChange {
            coord,
            tile: TileId(1),
        },
        remove_tile_collision_object: true,
        clear_gels: true,
        play_break_sound: true,
        score_delta: 50,
        debris: [
            LegacyBlockDebrisState::spawn(debris_x, debris_y, 3.5, -23.0),
            LegacyBlockDebrisState::spawn(debris_x, debris_y, -3.5, -23.0),
            LegacyBlockDebrisState::spawn(debris_x, debris_y, 3.5, -14.0),
            LegacyBlockDebrisState::spawn(debris_x, debris_y, -3.5, -14.0),
        ],
        regenerate_sprite_batch: true,
    })
}

#[must_use]
pub fn legacy_block_enemy_shot_requests(
    coord: TileCoord,
    enemies: &[LegacyBlockTopEnemy],
) -> Vec<LegacyBlockEnemyShotRequest> {
    let block_x = coord.x as f32;
    let block_y = coord.y as f32;
    let source_center_x = block_x - 0.5;

    enemies
        .iter()
        .copied()
        .filter(|enemy| {
            enemy.has_shotted_handler
                && in_range(enemy.center_x(), block_x - 1.0, block_x, true)
                && block_y - 1.0 == enemy.bottom()
        })
        .map(|enemy| {
            let center_x = enemy.center_x();
            LegacyBlockEnemyShotRequest {
                index: enemy.index,
                direction: if center_x < source_center_x {
                    LegacyEnemyDirection::Left
                } else {
                    LegacyEnemyDirection::Right
                },
                score_delta: 100,
                score_x: center_x,
                score_y: enemy.y,
            }
        })
        .collect()
}

#[must_use]
pub const fn legacy_block_hit_sound_requested(context: LegacyBlockHitSoundContext) -> bool {
    !context.blocked_by_portal_guard && !context.editor_mode && context.in_map
}

#[must_use]
pub fn update_legacy_block_bounce_completion(
    state: &mut LegacyBlockBounceSchedule,
    dt: f32,
) -> LegacyBlockBounceCompletionUpdate {
    if state.timer < LEGACY_BLOCK_BOUNCE_DURATION {
        state.timer += dt;
        if state.timer > LEGACY_BLOCK_BOUNCE_DURATION {
            state.timer = LEGACY_BLOCK_BOUNCE_DURATION;
            return LegacyBlockBounceCompletionUpdate {
                remove: true,
                replay_spawn: legacy_block_bounce_replay_spawn(state),
            };
        }
    }

    LegacyBlockBounceCompletionUpdate {
        remove: false,
        replay_spawn: None,
    }
}

#[must_use]
pub fn prune_legacy_completed_block_bounces(
    queue: &mut Vec<LegacyBlockBounceSchedule>,
    completed_indices: &[usize],
) -> LegacyBlockBounceQueuePruneUpdate {
    if completed_indices.is_empty() {
        return LegacyBlockBounceQueuePruneUpdate {
            regenerate_sprite_batch: false,
        };
    }

    let mut sorted_indices = completed_indices.to_vec();
    sorted_indices.sort_unstable_by(|left, right| right.cmp(left));

    let mut removed_any = false;
    let mut previous = None;

    for index in sorted_indices {
        if previous == Some(index) {
            continue;
        }
        previous = Some(index);

        if index < queue.len() {
            queue.remove(index);
            removed_any = true;
        }
    }

    LegacyBlockBounceQueuePruneUpdate {
        regenerate_sprite_batch: removed_any,
    }
}

fn legacy_block_bounce_replay_spawn(
    state: &LegacyBlockBounceSchedule,
) -> Option<LegacyBlockBounceReplaySpawn> {
    let x = state.coord.x as f32;
    let y = state.coord.y as f32;

    match state.spawn_content {
        Some(LegacyBlockBounceSpawnKind::Mushroom) => Some(LegacyBlockBounceReplaySpawn {
            kind: if state.hitter_size > 1 {
                LegacyBlockBounceReplayKind::Flower
            } else {
                LegacyBlockBounceReplayKind::Mushroom
            },
            x: x - 0.5,
            y: y - 2.0 / 16.0,
        }),
        Some(LegacyBlockBounceSpawnKind::OneUp) => Some(LegacyBlockBounceReplaySpawn {
            kind: LegacyBlockBounceReplayKind::OneUp,
            x: x - 0.5,
            y: y - 2.0 / 16.0,
        }),
        Some(LegacyBlockBounceSpawnKind::Star) => Some(LegacyBlockBounceReplaySpawn {
            kind: LegacyBlockBounceReplayKind::Star,
            x: x - 0.5,
            y: y - 2.0 / 16.0,
        }),
        Some(LegacyBlockBounceSpawnKind::Vine) => Some(LegacyBlockBounceReplaySpawn {
            kind: LegacyBlockBounceReplayKind::Vine,
            x,
            y,
        }),
        None => None,
    }
}

#[must_use]
pub fn legacy_block_item_jump_requests(
    coord: TileCoord,
    items: &[LegacyBlockJumpItem],
) -> Vec<LegacyBlockItemJumpRequest> {
    let block_x = coord.x as f32;
    let block_y = coord.y as f32;

    items
        .iter()
        .copied()
        .filter(|item| {
            item.has_jump_handler
                && in_range(item.center_x(), block_x - 1.0, block_x, true)
                && block_y - 1.0 == item.bottom()
        })
        .map(|item| LegacyBlockItemJumpRequest {
            kind: item.kind,
            index: item.index,
            source_x: block_x,
        })
        .collect()
}

#[must_use]
pub const fn legacy_used_coin_block_tile(
    spriteset: LegacyBlockSpriteset,
    invisible: bool,
) -> TileId {
    match (spriteset, invisible) {
        (LegacyBlockSpriteset::One, _) => TileId(113),
        (LegacyBlockSpriteset::Two, true) => TileId(118),
        (LegacyBlockSpriteset::Two, false) => TileId(114),
        (LegacyBlockSpriteset::Other, true) => TileId(112),
        (LegacyBlockSpriteset::Other, false) => TileId(117),
    }
}

fn legacy_coin_count_after_reward(
    incremented_coin_count: u32,
    life_count_enabled: bool,
    player_count: usize,
    constants: LegacyCoinBlockRewardConstants,
) -> (u32, Option<LegacyCoinLifeReward>) {
    if incremented_coin_count != constants.coin_life_threshold {
        return (incremented_coin_count, None);
    }

    (
        0,
        Some(LegacyCoinLifeReward {
            grant_lives_to_players: if life_count_enabled { player_count } else { 0 },
            respawn_players: life_count_enabled,
            play_sound: true,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        LEGACY_BLOCK_BOUNCE_DURATION, LEGACY_BLOCK_BOUNCE_TIMER_START,
        LegacyBlockBounceCompletionUpdate, LegacyBlockBounceContentKind, LegacyBlockBounceContext,
        LegacyBlockBounceQueuePruneUpdate, LegacyBlockBounceReplayKind,
        LegacyBlockBounceReplaySpawn, LegacyBlockBounceSchedule, LegacyBlockBounceSpawnKind,
        LegacyBlockContainedRewardRevealContext, LegacyBlockContainedRewardRevealOutcome,
        LegacyBlockEnemyShotRequest, LegacyBlockHitSoundContext, LegacyBlockItemJumpRequest,
        LegacyBlockJumpItem, LegacyBlockJumpItemKind, LegacyBlockPortalReservation,
        LegacyBlockRevealSound, LegacyBlockSpriteset, LegacyBlockTopCoinCollectionContext,
        LegacyBlockTopCoinCollectionOutcome, LegacyBlockTopEnemy, LegacyBreakableBlockOutcome,
        LegacyBrokenBlockEffects, LegacyCoinBlockRewardContext, LegacyCoinBlockRewardKind,
        LegacyCoinBlockRewardOutcome, LegacyCoinBlockTimerSpawn, LegacyCoinLifeReward,
        LegacyEmptyBreakableBlockDestroyContext, LegacyManyCoinsTimer, LegacyManyCoinsTimerEntry,
        LegacyTileChange, legacy_block_bounce_schedule, legacy_block_contained_reward_reveal,
        legacy_block_enemy_shot_requests, legacy_block_hit_sound_requested,
        legacy_block_item_jump_requests, legacy_block_top_coin_collection,
        legacy_breakable_block_hit, legacy_coin_block_reward, legacy_empty_breakable_block_destroy,
        legacy_many_coins_timer_state, legacy_used_coin_block_tile,
        prune_legacy_completed_block_bounces, update_legacy_block_bounce_completion,
        update_legacy_many_coins_timer,
    };
    use crate::{
        config::LegacyCoinBlockRewardConstants,
        effects::{LegacyBlockDebrisState, LegacyCoinBlockAnimationState},
        enemy::LegacyEnemyDirection,
        level::{TileCoord, TileId},
        wormhole::Facing,
    };

    #[test]
    fn breakable_block_hit_is_no_op_when_coord_matches_portal_endpoint() {
        let coord = TileCoord::new(7, 9);
        let portals = [LegacyBlockPortalReservation::new(coord, Facing::Right)];

        assert_eq!(
            legacy_breakable_block_hit(coord, &portals),
            LegacyBreakableBlockOutcome::ProtectedByPortal
        );
    }

    #[test]
    fn breakable_block_hit_is_no_op_when_coord_matches_portal_adjacent_tile() {
        let base = TileCoord::new(7, 9);

        for (facing, protected) in [
            (Facing::Up, TileCoord::new(7, 8)),
            (Facing::Right, TileCoord::new(8, 9)),
            (Facing::Down, TileCoord::new(7, 10)),
            (Facing::Left, TileCoord::new(6, 9)),
        ] {
            let portals = [LegacyBlockPortalReservation::new(base, facing)];
            assert_eq!(
                legacy_breakable_block_hit(protected, &portals),
                LegacyBreakableBlockOutcome::ProtectedByPortal
            );
        }
    }

    #[test]
    fn breakable_block_hit_emits_legacy_break_effects_when_not_portal_protected() {
        let coord = TileCoord::new(7, 9);
        let portals = [LegacyBlockPortalReservation::new(
            TileCoord::new(3, 4),
            Facing::Down,
        )];

        assert_eq!(
            legacy_breakable_block_hit(coord, &portals),
            LegacyBreakableBlockOutcome::Broken(LegacyBrokenBlockEffects {
                tile_change: LegacyTileChange {
                    coord,
                    tile: TileId(1),
                },
                remove_tile_collision_object: true,
                clear_gels: true,
                play_break_sound: true,
                score_delta: 50,
                debris: [
                    LegacyBlockDebrisState::spawn(6.5, 8.5, 3.5, -23.0),
                    LegacyBlockDebrisState::spawn(6.5, 8.5, -3.5, -23.0),
                    LegacyBlockDebrisState::spawn(6.5, 8.5, 3.5, -14.0),
                    LegacyBlockDebrisState::spawn(6.5, 8.5, -3.5, -14.0),
                ],
                regenerate_sprite_batch: true,
            })
        );
    }

    #[test]
    fn breakable_block_hit_is_not_blocked_by_unrelated_adjacent_portal_tiles() {
        let coord = TileCoord::new(7, 9);
        let portals = [LegacyBlockPortalReservation::new(
            TileCoord::new(7, 10),
            Facing::Down,
        )];

        assert!(matches!(
            legacy_breakable_block_hit(coord, &portals),
            LegacyBreakableBlockOutcome::Broken(_)
        ));
    }

    #[test]
    fn empty_breakable_block_destroy_requires_big_hitter_non_coin_and_empty_content() {
        let coord = TileCoord::new(7, 9);

        for context in [
            LegacyEmptyBreakableBlockDestroyContext {
                coord,
                hitter_size: None,
                is_coin_block: false,
                content: None,
            },
            LegacyEmptyBreakableBlockDestroyContext {
                coord,
                hitter_size: Some(1),
                is_coin_block: false,
                content: None,
            },
            LegacyEmptyBreakableBlockDestroyContext {
                coord,
                hitter_size: Some(2),
                is_coin_block: true,
                content: None,
            },
            LegacyEmptyBreakableBlockDestroyContext {
                coord,
                hitter_size: Some(2),
                is_coin_block: false,
                content: Some(LegacyBlockBounceContentKind::ManyCoins),
            },
            LegacyEmptyBreakableBlockDestroyContext {
                coord,
                hitter_size: Some(2),
                is_coin_block: false,
                content: Some(LegacyBlockBounceContentKind::Mushroom),
            },
        ] {
            assert_eq!(legacy_empty_breakable_block_destroy(context, &[]), None);
        }
    }

    #[test]
    fn empty_breakable_block_destroy_delegates_to_breakable_block_effects_for_big_empty_hits() {
        let coord = TileCoord::new(7, 9);
        let portals = [LegacyBlockPortalReservation::new(
            TileCoord::new(3, 4),
            Facing::Down,
        )];

        assert_eq!(
            legacy_empty_breakable_block_destroy(
                LegacyEmptyBreakableBlockDestroyContext {
                    coord,
                    hitter_size: Some(2),
                    is_coin_block: false,
                    content: None,
                },
                &portals,
            ),
            Some(legacy_breakable_block_hit(coord, &portals))
        );
    }

    #[test]
    fn empty_breakable_block_destroy_preserves_portal_protected_no_op() {
        let coord = TileCoord::new(7, 9);
        let portals = [LegacyBlockPortalReservation::new(coord, Facing::Right)];

        assert_eq!(
            legacy_empty_breakable_block_destroy(
                LegacyEmptyBreakableBlockDestroyContext {
                    coord,
                    hitter_size: Some(2),
                    is_coin_block: false,
                    content: None,
                },
                &portals,
            ),
            Some(LegacyBreakableBlockOutcome::ProtectedByPortal)
        );
    }

    #[test]
    fn block_hit_sound_is_requested_for_in_map_non_editor_non_portal_hits() {
        assert!(legacy_block_hit_sound_requested(
            LegacyBlockHitSoundContext {
                blocked_by_portal_guard: false,
                editor_mode: false,
                in_map: true,
            }
        ));
    }

    #[test]
    fn block_hit_sound_is_suppressed_by_portal_guard_and_editor_mode() {
        assert!(!legacy_block_hit_sound_requested(
            LegacyBlockHitSoundContext {
                blocked_by_portal_guard: true,
                editor_mode: false,
                in_map: true,
            }
        ));

        assert!(!legacy_block_hit_sound_requested(
            LegacyBlockHitSoundContext {
                blocked_by_portal_guard: false,
                editor_mode: true,
                in_map: true,
            }
        ));
    }

    #[test]
    fn block_hit_sound_is_suppressed_when_target_coord_is_out_of_map() {
        assert!(!legacy_block_hit_sound_requested(
            LegacyBlockHitSoundContext {
                blocked_by_portal_guard: false,
                editor_mode: false,
                in_map: false,
            }
        ));
    }

    #[test]
    fn block_item_jump_requests_match_legacy_center_and_bottom_probe() {
        let coord = TileCoord::new(7, 9);
        let items = [
            LegacyBlockJumpItem {
                kind: LegacyBlockJumpItemKind::Mushroom,
                index: 3,
                x: 5.5,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_jump_handler: true,
            },
            LegacyBlockJumpItem {
                kind: LegacyBlockJumpItemKind::OneUp,
                index: 4,
                x: 6.5,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_jump_handler: true,
            },
        ];

        assert_eq!(
            legacy_block_item_jump_requests(coord, &items),
            vec![
                LegacyBlockItemJumpRequest {
                    kind: LegacyBlockJumpItemKind::Mushroom,
                    index: 3,
                    source_x: 7.0,
                },
                LegacyBlockItemJumpRequest {
                    kind: LegacyBlockJumpItemKind::OneUp,
                    index: 4,
                    source_x: 7.0,
                },
            ]
        );
    }

    #[test]
    fn block_item_jump_requests_require_jump_handler_horizontal_range_and_exact_bottom() {
        let coord = TileCoord::new(7, 9);
        let items = [
            LegacyBlockJumpItem {
                kind: LegacyBlockJumpItemKind::Mushroom,
                index: 1,
                x: 5.499,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_jump_handler: true,
            },
            LegacyBlockJumpItem {
                kind: LegacyBlockJumpItemKind::Mushroom,
                index: 2,
                x: 5.5,
                y: 7.499,
                width: 1.0,
                height: 0.5,
                has_jump_handler: true,
            },
            LegacyBlockJumpItem {
                kind: LegacyBlockJumpItemKind::OneUp,
                index: 3,
                x: 5.5,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_jump_handler: false,
            },
        ];

        assert!(legacy_block_item_jump_requests(coord, &items).is_empty());
    }

    #[test]
    fn block_enemy_shot_requests_match_legacy_top_probe_direction_and_score_event() {
        let coord = TileCoord::new(7, 9);
        let enemies = [
            LegacyBlockTopEnemy {
                index: 2,
                x: 5.5,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_shotted_handler: true,
            },
            LegacyBlockTopEnemy {
                index: 3,
                x: 6.0,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_shotted_handler: true,
            },
        ];

        assert_eq!(
            legacy_block_enemy_shot_requests(coord, &enemies),
            vec![
                LegacyBlockEnemyShotRequest {
                    index: 2,
                    direction: LegacyEnemyDirection::Left,
                    score_delta: 100,
                    score_x: 6.0,
                    score_y: 7.5,
                },
                LegacyBlockEnemyShotRequest {
                    index: 3,
                    direction: LegacyEnemyDirection::Right,
                    score_delta: 100,
                    score_x: 6.5,
                    score_y: 7.5,
                },
            ]
        );
    }

    #[test]
    fn block_enemy_shot_requests_require_shot_handler_horizontal_range_and_exact_bottom() {
        let coord = TileCoord::new(7, 9);
        let enemies = [
            LegacyBlockTopEnemy {
                index: 1,
                x: 5.499,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_shotted_handler: true,
            },
            LegacyBlockTopEnemy {
                index: 2,
                x: 5.5,
                y: 7.499,
                width: 1.0,
                height: 0.5,
                has_shotted_handler: true,
            },
            LegacyBlockTopEnemy {
                index: 3,
                x: 5.5,
                y: 7.5,
                width: 1.0,
                height: 0.5,
                has_shotted_handler: false,
            },
        ];

        assert!(legacy_block_enemy_shot_requests(coord, &enemies).is_empty());
    }

    #[test]
    fn block_enemy_shot_requests_preserve_duplicate_probe_entries() {
        let coord = TileCoord::new(7, 9);
        let enemy = LegacyBlockTopEnemy {
            index: 7,
            x: 5.5,
            y: 7.5,
            width: 1.0,
            height: 0.5,
            has_shotted_handler: true,
        };

        assert_eq!(
            legacy_block_enemy_shot_requests(coord, &[enemy, enemy]),
            vec![
                LegacyBlockEnemyShotRequest {
                    index: 7,
                    direction: LegacyEnemyDirection::Left,
                    score_delta: 100,
                    score_x: 6.0,
                    score_y: 7.5,
                },
                LegacyBlockEnemyShotRequest {
                    index: 7,
                    direction: LegacyEnemyDirection::Left,
                    score_delta: 100,
                    score_x: 6.0,
                    score_y: 7.5,
                },
            ]
        );
    }

    #[test]
    fn block_bounce_schedule_preserves_empty_hit_coord_timer_size_and_spritebatch_flag() {
        let coord = TileCoord::new(7, 9);

        assert_eq!(
            legacy_block_bounce_schedule(LegacyBlockBounceContext {
                coord,
                hitter_size: 2,
                content: None,
            }),
            LegacyBlockBounceSchedule {
                timer: LEGACY_BLOCK_BOUNCE_TIMER_START,
                coord,
                spawn_content: None,
                hitter_size: 2,
                regenerate_sprite_batch: true,
            }
        );
    }

    #[test]
    fn block_bounce_schedule_maps_spawnable_contents_to_item_replay_kinds() {
        for (content, spawn_content) in [
            (
                LegacyBlockBounceContentKind::Mushroom,
                LegacyBlockBounceSpawnKind::Mushroom,
            ),
            (
                LegacyBlockBounceContentKind::OneUp,
                LegacyBlockBounceSpawnKind::OneUp,
            ),
            (
                LegacyBlockBounceContentKind::Star,
                LegacyBlockBounceSpawnKind::Star,
            ),
            (
                LegacyBlockBounceContentKind::Vine,
                LegacyBlockBounceSpawnKind::Vine,
            ),
        ] {
            assert_eq!(
                legacy_block_bounce_schedule(LegacyBlockBounceContext {
                    coord: TileCoord::new(7, 9),
                    hitter_size: 1,
                    content: Some(content),
                })
                .spawn_content,
                Some(spawn_content)
            );
        }
    }

    #[test]
    fn block_bounce_schedule_treats_many_coins_as_empty_content_but_preserves_size() {
        let schedule = legacy_block_bounce_schedule(LegacyBlockBounceContext {
            coord: TileCoord::new(7, 9),
            hitter_size: 2,
            content: Some(LegacyBlockBounceContentKind::ManyCoins),
        });

        assert_eq!(schedule.spawn_content, None);
        assert_eq!(schedule.hitter_size, 2);
    }

    #[test]
    fn block_bounce_completion_crossing_duration_removes_entry_and_replays_small_mushroom_spawn() {
        let mut state = LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_TIMER_START,
            coord: TileCoord::new(7, 9),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, LEGACY_BLOCK_BOUNCE_DURATION + 0.01),
            LegacyBlockBounceCompletionUpdate {
                remove: true,
                replay_spawn: Some(LegacyBlockBounceReplaySpawn {
                    kind: LegacyBlockBounceReplayKind::Mushroom,
                    x: 6.5,
                    y: 8.875,
                }),
            }
        );
        assert_eq!(state.timer, LEGACY_BLOCK_BOUNCE_DURATION);
    }

    #[test]
    fn block_bounce_completion_upgrades_big_hitter_mushroom_replay_to_flower() {
        let mut state = LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(7, 9),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
            hitter_size: 2,
            regenerate_sprite_batch: true,
        };

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, 0.02),
            LegacyBlockBounceCompletionUpdate {
                remove: true,
                replay_spawn: Some(LegacyBlockBounceReplaySpawn {
                    kind: LegacyBlockBounceReplayKind::Flower,
                    x: 6.5,
                    y: 8.875,
                }),
            }
        );
    }

    #[test]
    fn block_bounce_completion_replays_vine_at_block_coord() {
        let mut state = LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(7, 9),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Vine),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, 0.02),
            LegacyBlockBounceCompletionUpdate {
                remove: true,
                replay_spawn: Some(LegacyBlockBounceReplaySpawn {
                    kind: LegacyBlockBounceReplayKind::Vine,
                    x: 7.0,
                    y: 9.0,
                }),
            }
        );
    }

    #[test]
    fn block_bounce_completion_removes_empty_entries_without_replay() {
        let mut state = LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(7, 9),
            spawn_content: None,
            hitter_size: 2,
            regenerate_sprite_batch: true,
        };

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, 0.02),
            LegacyBlockBounceCompletionUpdate {
                remove: true,
                replay_spawn: None,
            }
        );
    }

    #[test]
    fn block_bounce_completion_requires_strict_threshold_crossing_to_replay_and_remove() {
        let mut state = LegacyBlockBounceSchedule {
            timer: LEGACY_BLOCK_BOUNCE_DURATION - 0.01,
            coord: TileCoord::new(7, 9),
            spawn_content: Some(LegacyBlockBounceSpawnKind::OneUp),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, 0.01),
            LegacyBlockBounceCompletionUpdate {
                remove: false,
                replay_spawn: None,
            }
        );
        assert_eq!(state.timer, LEGACY_BLOCK_BOUNCE_DURATION);

        assert_eq!(
            update_legacy_block_bounce_completion(&mut state, 0.01),
            LegacyBlockBounceCompletionUpdate {
                remove: false,
                replay_spawn: None,
            }
        );
        assert_eq!(state.timer, LEGACY_BLOCK_BOUNCE_DURATION);
    }

    #[test]
    fn block_bounce_queue_prune_removes_completed_entries_in_descending_index_order() {
        let mut queue = vec![
            LegacyBlockBounceSchedule {
                timer: 0.05,
                coord: TileCoord::new(1, 1),
                spawn_content: None,
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
            LegacyBlockBounceSchedule {
                timer: 0.1,
                coord: TileCoord::new(2, 2),
                spawn_content: Some(LegacyBlockBounceSpawnKind::Mushroom),
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
            LegacyBlockBounceSchedule {
                timer: 0.15,
                coord: TileCoord::new(3, 3),
                spawn_content: Some(LegacyBlockBounceSpawnKind::Vine),
                hitter_size: 2,
                regenerate_sprite_batch: true,
            },
            LegacyBlockBounceSchedule {
                timer: LEGACY_BLOCK_BOUNCE_DURATION,
                coord: TileCoord::new(4, 4),
                spawn_content: None,
                hitter_size: 1,
                regenerate_sprite_batch: true,
            },
        ];

        assert_eq!(
            prune_legacy_completed_block_bounces(&mut queue, &[1, 3]),
            LegacyBlockBounceQueuePruneUpdate {
                regenerate_sprite_batch: true,
            }
        );

        assert_eq!(
            queue,
            vec![
                LegacyBlockBounceSchedule {
                    timer: 0.05,
                    coord: TileCoord::new(1, 1),
                    spawn_content: None,
                    hitter_size: 1,
                    regenerate_sprite_batch: true,
                },
                LegacyBlockBounceSchedule {
                    timer: 0.15,
                    coord: TileCoord::new(3, 3),
                    spawn_content: Some(LegacyBlockBounceSpawnKind::Vine),
                    hitter_size: 2,
                    regenerate_sprite_batch: true,
                },
            ]
        );
    }

    #[test]
    fn block_bounce_queue_prune_is_no_op_without_completed_entries() {
        let original = LegacyBlockBounceSchedule {
            timer: 0.05,
            coord: TileCoord::new(7, 9),
            spawn_content: Some(LegacyBlockBounceSpawnKind::Star),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };
        let mut queue = vec![original];

        assert_eq!(
            prune_legacy_completed_block_bounces(&mut queue, &[]),
            LegacyBlockBounceQueuePruneUpdate {
                regenerate_sprite_batch: false,
            }
        );
        assert_eq!(queue, vec![original]);
    }

    #[test]
    fn block_bounce_queue_prune_ignores_duplicate_and_out_of_bounds_indices() {
        let first = LegacyBlockBounceSchedule {
            timer: 0.05,
            coord: TileCoord::new(5, 5),
            spawn_content: None,
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };
        let second = LegacyBlockBounceSchedule {
            timer: 0.1,
            coord: TileCoord::new(6, 6),
            spawn_content: Some(LegacyBlockBounceSpawnKind::OneUp),
            hitter_size: 1,
            regenerate_sprite_batch: true,
        };
        let mut queue = vec![first, second];

        assert_eq!(
            prune_legacy_completed_block_bounces(&mut queue, &[1, 1, 9]),
            LegacyBlockBounceQueuePruneUpdate {
                regenerate_sprite_batch: true,
            }
        );
        assert_eq!(queue, vec![first]);
    }

    #[test]
    fn many_coins_timer_countdown_decrements_while_remaining_is_positive() {
        assert_eq!(update_legacy_many_coins_timer(4.0, 0.25), 3.75);
    }

    #[test]
    fn many_coins_timer_countdown_preserves_negative_overshoot_after_positive_update() {
        assert_eq!(update_legacy_many_coins_timer(0.1, 0.25), -0.15);
    }

    #[test]
    fn many_coins_timer_countdown_is_no_op_for_zero_or_negative_remaining() {
        assert_eq!(update_legacy_many_coins_timer(0.0, 0.25), 0.0);
        assert_eq!(update_legacy_many_coins_timer(-0.15, 0.25), -0.15);
    }

    #[test]
    fn many_coins_timer_state_is_missing_when_block_has_no_matching_timer() {
        let timers = [
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(6, 9),
                remaining: 4.0,
            },
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(8, 9),
                remaining: 1.5,
            },
        ];

        assert_eq!(
            legacy_many_coins_timer_state(TileCoord::new(7, 9), &timers),
            LegacyManyCoinsTimer::Missing
        );
    }

    #[test]
    fn many_coins_timer_state_returns_matching_timer_remaining_value() {
        let timers = [LegacyManyCoinsTimerEntry {
            coord: TileCoord::new(7, 9),
            remaining: 3.75,
        }];

        assert_eq!(
            legacy_many_coins_timer_state(TileCoord::new(7, 9), &timers),
            LegacyManyCoinsTimer::Existing { remaining: 3.75 }
        );
    }

    #[test]
    fn many_coins_timer_state_uses_last_matching_duplicate_entry() {
        let timers = [
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(7, 9),
                remaining: 4.0,
            },
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(5, 5),
                remaining: 2.0,
            },
            LegacyManyCoinsTimerEntry {
                coord: TileCoord::new(7, 9),
                remaining: -0.25,
            },
        ];

        assert_eq!(
            legacy_many_coins_timer_state(TileCoord::new(7, 9), &timers),
            LegacyManyCoinsTimer::Existing { remaining: -0.25 }
        );
    }

    #[test]
    fn block_contained_reward_reveal_is_no_op_for_empty_or_many_coin_content() {
        let coord = TileCoord::new(7, 9);

        assert_eq!(
            legacy_block_contained_reward_reveal(LegacyBlockContainedRewardRevealContext {
                coord,
                spriteset: LegacyBlockSpriteset::Two,
                invisible: false,
                content: None,
            }),
            None
        );

        assert_eq!(
            legacy_block_contained_reward_reveal(LegacyBlockContainedRewardRevealContext {
                coord,
                spriteset: LegacyBlockSpriteset::Two,
                invisible: false,
                content: Some(LegacyBlockBounceContentKind::ManyCoins),
            }),
            None
        );
    }

    #[test]
    fn block_contained_reward_reveal_uses_used_tile_mapping_and_mushroom_sound() {
        let coord = TileCoord::new(7, 9);

        assert_eq!(
            legacy_block_contained_reward_reveal(LegacyBlockContainedRewardRevealContext {
                coord,
                spriteset: LegacyBlockSpriteset::Other,
                invisible: true,
                content: Some(LegacyBlockBounceContentKind::Star),
            }),
            Some(LegacyBlockContainedRewardRevealOutcome {
                tile_change: LegacyTileChange {
                    coord,
                    tile: TileId(112),
                },
                sound: LegacyBlockRevealSound::MushroomAppear,
            })
        );
    }

    #[test]
    fn block_contained_reward_reveal_uses_vine_sound_for_vine_content() {
        let coord = TileCoord::new(7, 9);

        assert_eq!(
            legacy_block_contained_reward_reveal(LegacyBlockContainedRewardRevealContext {
                coord,
                spriteset: LegacyBlockSpriteset::Two,
                invisible: false,
                content: Some(LegacyBlockBounceContentKind::Vine),
            }),
            Some(LegacyBlockContainedRewardRevealOutcome {
                tile_change: LegacyTileChange {
                    coord,
                    tile: TileId(114),
                },
                sound: LegacyBlockRevealSound::Vine,
            })
        );
    }

    #[test]
    fn block_top_coin_collection_is_no_op_without_a_coin_coord() {
        let constants = LegacyCoinBlockRewardConstants::default();

        assert_eq!(
            legacy_block_top_coin_collection(
                LegacyBlockTopCoinCollectionContext {
                    top_coin_coord: None,
                    coin_count: 41,
                    life_count_enabled: true,
                    player_count: 2,
                },
                constants,
            ),
            None
        );
    }

    #[test]
    fn block_top_coin_collection_matches_legacy_tile_clear_score_sound_and_animation() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 8);

        assert_eq!(
            legacy_block_top_coin_collection(
                LegacyBlockTopCoinCollectionContext {
                    top_coin_coord: Some(coord),
                    coin_count: 41,
                    life_count_enabled: true,
                    player_count: 2,
                },
                constants,
            ),
            Some(LegacyBlockTopCoinCollectionOutcome {
                tile_change: LegacyTileChange {
                    coord,
                    tile: TileId(1),
                },
                play_coin_sound: true,
                animation: LegacyCoinBlockAnimationState::spawn(6.5, 8.0),
                score_delta: constants.score_delta,
                coin_count: 42,
                life_reward: None,
            })
        );
    }

    #[test]
    fn block_top_coin_collection_resets_at_exact_hundred_and_emits_life_reward() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 8);

        let outcome = legacy_block_top_coin_collection(
            LegacyBlockTopCoinCollectionContext {
                top_coin_coord: Some(coord),
                coin_count: 99,
                life_count_enabled: false,
                player_count: 3,
            },
            constants,
        )
        .expect("top coin should collect");

        assert_eq!(
            outcome,
            LegacyBlockTopCoinCollectionOutcome {
                tile_change: LegacyTileChange {
                    coord,
                    tile: TileId(1),
                },
                play_coin_sound: true,
                animation: LegacyCoinBlockAnimationState::spawn(6.5, 8.0),
                score_delta: constants.score_delta,
                coin_count: 0,
                life_reward: Some(LegacyCoinLifeReward {
                    grant_lives_to_players: 0,
                    respawn_players: false,
                    play_sound: true,
                }),
            }
        );
    }

    #[test]
    fn used_coin_block_tile_matches_legacy_spriteset_and_invisible_mapping() {
        assert_eq!(
            LegacyBlockSpriteset::from_legacy_index(1),
            LegacyBlockSpriteset::One
        );
        assert_eq!(
            LegacyBlockSpriteset::from_legacy_index(2),
            LegacyBlockSpriteset::Two
        );
        assert_eq!(
            LegacyBlockSpriteset::from_legacy_index(3),
            LegacyBlockSpriteset::Other
        );

        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::One, false),
            TileId(113)
        );
        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::One, true),
            TileId(113)
        );
        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::Two, false),
            TileId(114)
        );
        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::Two, true),
            TileId(118)
        );
        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::Other, false),
            TileId(117)
        );
        assert_eq!(
            legacy_used_coin_block_tile(LegacyBlockSpriteset::Other, true),
            TileId(112)
        );
    }

    #[test]
    fn single_coin_block_reward_sets_used_tile_and_emits_coin_score_and_animation() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        assert_eq!(
            legacy_coin_block_reward(
                LegacyCoinBlockRewardContext {
                    coord,
                    spriteset: LegacyBlockSpriteset::Two,
                    kind: LegacyCoinBlockRewardKind::Single { invisible: false },
                    coin_count: 41,
                    life_count_enabled: true,
                    player_count: 2,
                },
                constants,
            ),
            LegacyCoinBlockRewardOutcome {
                play_coin_sound: true,
                animation: LegacyCoinBlockAnimationState::spawn(6.5, 8.0),
                score_delta: 200,
                coin_count: 42,
                life_reward: None,
                tile_change: Some(LegacyTileChange {
                    coord,
                    tile: TileId(114),
                }),
                start_many_coins_timer: None,
            }
        );
    }

    #[test]
    fn single_invisible_coin_block_uses_invisible_used_tile_mapping() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);
        let outcome = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::Other,
                kind: LegacyCoinBlockRewardKind::Single { invisible: true },
                coin_count: 0,
                life_count_enabled: true,
                player_count: 1,
            },
            constants,
        );

        assert_eq!(
            outcome.tile_change,
            Some(LegacyTileChange {
                coord,
                tile: TileId(112)
            })
        );
    }

    #[test]
    fn coin_reward_resets_at_exact_hundred_and_emits_life_reward_even_when_lives_disabled() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        let enabled = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::One,
                kind: LegacyCoinBlockRewardKind::Single { invisible: false },
                coin_count: 99,
                life_count_enabled: true,
                player_count: 3,
            },
            constants,
        );
        assert_eq!(enabled.coin_count, 0);
        assert_eq!(
            enabled.life_reward,
            Some(LegacyCoinLifeReward {
                grant_lives_to_players: 3,
                respawn_players: true,
                play_sound: true,
            })
        );

        let disabled = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::One,
                kind: LegacyCoinBlockRewardKind::Single { invisible: false },
                coin_count: 99,
                life_count_enabled: false,
                player_count: 3,
            },
            constants,
        );
        assert_eq!(disabled.coin_count, 0);
        assert_eq!(
            disabled.life_reward,
            Some(LegacyCoinLifeReward {
                grant_lives_to_players: 0,
                respawn_players: false,
                play_sound: true,
            })
        );
    }

    #[test]
    fn coin_reward_only_resets_when_increment_lands_exactly_on_hundred() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        let outcome = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::One,
                kind: LegacyCoinBlockRewardKind::Single { invisible: false },
                coin_count: 100,
                life_count_enabled: true,
                player_count: 1,
            },
            constants,
        );

        assert_eq!(outcome.coin_count, 101);
        assert_eq!(outcome.life_reward, None);
    }

    #[test]
    fn many_coin_block_without_timer_starts_timer_but_does_not_change_tile() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        let outcome = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::Two,
                kind: LegacyCoinBlockRewardKind::ManyCoins {
                    timer: LegacyManyCoinsTimer::Missing,
                },
                coin_count: 3,
                life_count_enabled: true,
                player_count: 2,
            },
            constants,
        );

        assert_eq!(outcome.tile_change, None);
        assert_eq!(
            outcome.start_many_coins_timer,
            Some(LegacyCoinBlockTimerSpawn {
                coord,
                duration: constants.many_coins_timer_duration,
            })
        );
        assert_eq!(
            outcome.animation,
            LegacyCoinBlockAnimationState::spawn(6.5, 8.0)
        );
        assert_eq!(outcome.score_delta, constants.score_delta);
        assert_eq!(outcome.coin_count, 4);
    }

    #[test]
    fn many_coin_block_with_active_timer_keeps_tile_and_timer() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        let outcome = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::Two,
                kind: LegacyCoinBlockRewardKind::ManyCoins {
                    timer: LegacyManyCoinsTimer::Existing { remaining: 0.25 },
                },
                coin_count: 3,
                life_count_enabled: true,
                player_count: 2,
            },
            constants,
        );

        assert_eq!(outcome.tile_change, None);
        assert_eq!(outcome.start_many_coins_timer, None);
    }

    #[test]
    fn many_coin_block_with_expired_timer_uses_non_invisible_used_tile_mapping() {
        let constants = LegacyCoinBlockRewardConstants::default();
        let coord = TileCoord::new(7, 9);

        let outcome = legacy_coin_block_reward(
            LegacyCoinBlockRewardContext {
                coord,
                spriteset: LegacyBlockSpriteset::Other,
                kind: LegacyCoinBlockRewardKind::ManyCoins {
                    timer: LegacyManyCoinsTimer::Existing { remaining: 0.0 },
                },
                coin_count: 3,
                life_count_enabled: true,
                player_count: 2,
            },
            constants,
        );

        assert_eq!(
            outcome.tile_change,
            Some(LegacyTileChange {
                coord,
                tile: TileId(117)
            })
        );
        assert_eq!(outcome.start_many_coins_timer, None);
    }
}

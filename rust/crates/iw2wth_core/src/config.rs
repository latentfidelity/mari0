//! Typed constants ported from `variables.lua`.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsConstants {
    pub epsilon: f32,
    pub gravity: f32,
    pub jumping_gravity: f32,
    pub max_y_speed: f32,
    pub jump_force: f32,
    pub jump_force_add: f32,
    pub head_force: f32,
    pub bounce_height: f32,
    pub passive_speed: f32,
    pub space_run_room: f32,
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            epsilon: 0.000_000_001,
            gravity: 80.0,
            jumping_gravity: 30.0,
            max_y_speed: 100.0,
            jump_force: 16.0,
            jump_force_add: 1.9,
            head_force: 2.0,
            bounce_height: 14.0 / 16.0,
            passive_speed: 4.0,
            space_run_room: 1.2 / 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerMovementConstants {
    pub walk_acceleration: f32,
    pub run_acceleration: f32,
    pub air_walk_acceleration: f32,
    pub air_run_acceleration: f32,
    pub min_speed: f32,
    pub max_walk_speed: f32,
    pub max_run_speed: f32,
    pub friction: f32,
    pub super_friction: f32,
    pub air_friction: f32,
    pub air_slide_factor: f32,
}

impl Default for PlayerMovementConstants {
    fn default() -> Self {
        Self {
            walk_acceleration: 8.0,
            run_acceleration: 16.0,
            air_walk_acceleration: 8.0,
            air_run_acceleration: 16.0,
            min_speed: 0.7,
            max_walk_speed: 6.4,
            max_run_speed: 9.0,
            friction: 14.0,
            super_friction: 100.0,
            air_friction: 0.0,
            air_slide_factor: 0.8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrangeGelMovementConstants {
    pub max_run_speed: f32,
    pub max_walk_speed: f32,
    pub run_acceleration: f32,
    pub walk_acceleration: f32,
}

impl Default for OrangeGelMovementConstants {
    fn default() -> Self {
        Self {
            max_run_speed: 50.0,
            max_walk_speed: 25.0,
            run_acceleration: 25.0,
            walk_acceleration: 12.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlueGelBounceConstants {
    pub floor_threshold_gravity_multiplier: f32,
    pub horizontal_multiplier: f32,
    pub horizontal_vertical_speed: f32,
    pub horizontal_max_speed_x: f32,
    pub horizontal_min_speed_x: f32,
}

impl Default for BlueGelBounceConstants {
    fn default() -> Self {
        Self {
            floor_threshold_gravity_multiplier: 10.0,
            horizontal_multiplier: 1.5,
            horizontal_vertical_speed: 20.0,
            horizontal_max_speed_x: 15.0,
            horizontal_min_speed_x: 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringConstants {
    pub duration: f32,
    pub high_force: f32,
    pub force: f32,
    pub player_y_offset: f32,
}

impl Default for SpringConstants {
    fn default() -> Self {
        Self {
            duration: 0.2,
            high_force: 41.0,
            force: 24.0,
            player_y_offset: 31.0 / 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyVineConstants {
    pub speed: f32,
    pub move_speed: f32,
    pub move_down_speed: f32,
    pub frame_delay: f32,
    pub frame_delay_down: f32,
    pub animation_start_y: f32,
    pub drop_x_offset: f32,
    pub start_limit: f32,
    pub regular_limit: f32,
    pub width: f32,
    pub spawn_y_offset: f32,
    pub growth_height_offset: f32,
}

impl Default for LegacyVineConstants {
    fn default() -> Self {
        Self {
            speed: 2.13,
            move_speed: 3.21,
            move_down_speed: 6.42,
            frame_delay: 0.15,
            frame_delay_down: 0.075,
            animation_start_y: 4.0,
            drop_x_offset: 7.0 / 16.0,
            start_limit: 9.0 + 1.0 / 16.0,
            regular_limit: -1.0,
            width: 10.0 / 16.0,
            spawn_y_offset: 1.0,
            growth_height_offset: 1.7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPowerUpConstants {
    pub mushroom_speed: f32,
    pub emergence_time: f32,
    pub mushroom_jump_force: f32,
    pub star_jump_force: f32,
    pub star_gravity: f32,
    pub star_animation_delay: f32,
    pub flower_gravity: f32,
    pub flower_emerged_y_offset: f32,
    pub rotation_alignment_speed: f32,
    pub one_up_offscreen_y: f32,
    pub one_up_offscreen_left_margin: f32,
}

impl Default for LegacyPowerUpConstants {
    fn default() -> Self {
        Self {
            mushroom_speed: 3.6,
            emergence_time: 0.7,
            mushroom_jump_force: 13.0,
            star_jump_force: 13.0,
            star_gravity: 40.0,
            star_animation_delay: 0.04,
            flower_gravity: 0.0,
            flower_emerged_y_offset: 27.0 / 16.0,
            rotation_alignment_speed: 15.0,
            one_up_offscreen_y: 18.0,
            one_up_offscreen_left_margin: 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerAnimationConstants {
    pub run_animation_speed: f32,
    pub swim_animation_speed: f32,
}

impl Default for PlayerAnimationConstants {
    fn default() -> Self {
        Self {
            run_animation_speed: 10.0,
            swim_animation_speed: 10.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnderwaterMovementConstants {
    pub walk_acceleration: f32,
    pub run_acceleration: f32,
    pub air_walk_acceleration: f32,
    pub max_air_walk_speed: f32,
    pub max_walk_speed: f32,
    pub max_run_speed: f32,
    pub friction: f32,
    pub super_friction: f32,
    pub air_friction: f32,
    pub air_slide_factor: f32,
    pub jump_force: f32,
    pub jump_force_add: f32,
    pub gravity: f32,
    pub jumping_gravity: f32,
    pub max_height: f32,
    pub push_down_speed: f32,
}

impl Default for UnderwaterMovementConstants {
    fn default() -> Self {
        Self {
            walk_acceleration: 8.0,
            run_acceleration: 16.0,
            air_walk_acceleration: 8.0,
            max_air_walk_speed: 5.0,
            max_walk_speed: 3.6,
            max_run_speed: 5.0,
            friction: 14.0,
            super_friction: 100.0,
            air_friction: 0.0,
            air_slide_factor: 0.8,
            jump_force: 5.9,
            jump_force_add: 0.0,
            gravity: 9.0,
            jumping_gravity: 12.0,
            max_height: 2.5,
            push_down_speed: 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyGoombaConstants {
    pub speed: f32,
    pub animation_speed: f32,
    pub death_time: f32,
    pub friction: f32,
    pub shot_speed_x: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacyGoombaConstants {
    fn default() -> Self {
        Self {
            speed: 2.0,
            animation_speed: 0.2,
            death_time: 0.5,
            friction: 14.0,
            shot_speed_x: 4.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCheepCheepConstants {
    pub white_speed: f32,
    pub red_speed: f32,
    pub vertical_speed: f32,
    pub vertical_range: f32,
    pub animation_speed: f32,
    pub shot_speed_x: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacyCheepCheepConstants {
    fn default() -> Self {
        Self {
            white_speed: 1.0,
            red_speed: 1.8,
            vertical_speed: 0.3,
            vertical_range: 1.0,
            animation_speed: 0.35,
            shot_speed_x: 4.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySquidConstants {
    pub fall_speed: f32,
    pub horizontal_speed: f32,
    pub up_speed: f32,
    pub acceleration: f32,
    pub down_distance: f32,
    pub shot_speed_x: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacySquidConstants {
    fn default() -> Self {
        Self {
            fall_speed: 0.9,
            horizontal_speed: 3.0,
            up_speed: 3.0,
            acceleration: 10.0,
            down_distance: 1.0,
            shot_speed_x: 4.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyLakitoConstants {
    pub throw_time: f32,
    pub respawn_time: f32,
    pub space: f32,
    pub distance_time: f32,
    pub hide_time: f32,
    pub passive_speed: f32,
    pub max_spikey_count: usize,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
}

impl Default for LegacyLakitoConstants {
    fn default() -> Self {
        Self {
            throw_time: 4.0,
            respawn_time: 16.0,
            space: 4.0,
            distance_time: 1.5,
            hide_time: 0.5,
            passive_speed: 3.0,
            max_spikey_count: 3,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHammerConstants {
    pub speed: f32,
    pub start_y_speed: f32,
    pub gravity: f32,
    pub animation_speed: f32,
}

impl Default for LegacyHammerConstants {
    fn default() -> Self {
        Self {
            speed: 4.0,
            start_y_speed: 8.0,
            gravity: 25.0,
            animation_speed: 0.05,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHammerBroConstants {
    pub prepare_time: f32,
    pub throw_time_short: f32,
    pub throw_time_long: f32,
    pub speed: f32,
    pub animation_speed: f32,
    pub jump_time: f32,
    pub jump_force: f32,
    pub jump_force_down: f32,
    pub gravity: f32,
    pub friction: f32,
    pub side_collision_speed: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub shot_removal_y: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacyHammerBroConstants {
    fn default() -> Self {
        Self {
            prepare_time: 0.5,
            throw_time_short: 0.6,
            throw_time_long: 1.6,
            speed: 1.5,
            animation_speed: 0.15,
            jump_time: 3.0,
            jump_force: 19.0,
            jump_force_down: 6.0,
            gravity: 40.0,
            friction: 14.0,
            side_collision_speed: 2.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            shot_removal_y: 18.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFlyingFishConstants {
    pub gravity: f32,
    pub jump_force: f32,
    pub animation_speed: f32,
    pub shot_speed_x: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacyFlyingFishConstants {
    fn default() -> Self {
        Self {
            gravity: 20.0,
            jump_force: 23.0,
            animation_speed: 0.35,
            shot_speed_x: 4.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireballConstants {
    pub speed: f32,
    pub jump_force: f32,
    pub max_count: usize,
    pub animation_delay: f32,
    pub offscreen_y: f32,
}

impl Default for LegacyFireballConstants {
    fn default() -> Self {
        Self {
            speed: 15.0,
            jump_force: 10.0,
            max_count: 2,
            animation_delay: 0.04,
            offscreen_y: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireConstants {
    pub speed: f32,
    pub vertical_speed: f32,
    pub animation_delay: f32,
}

impl Default for LegacyFireConstants {
    fn default() -> Self {
        Self {
            speed: 4.69,
            vertical_speed: 2.0,
            animation_delay: 0.05,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyUpFireConstants {
    pub start_y_offset: f32,
    pub force: f32,
    pub gravity: f32,
    pub hide_y: f32,
}

impl Default for LegacyUpFireConstants {
    fn default() -> Self {
        Self {
            start_y_offset: 8.0,
            force: 19.0,
            gravity: 20.0,
            hide_y: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCastleFireConstants {
    pub angle_step_degrees: f32,
    pub angle_delay: f32,
    pub animation_delay: f32,
}

impl Default for LegacyCastleFireConstants {
    fn default() -> Self {
        let angle_step_degrees = 11.25;

        Self {
            angle_step_degrees,
            angle_delay: 3.4 / (360.0 / angle_step_degrees),
            animation_delay: 0.07,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireworkConstants {
    pub duration: f32,
    pub sound_time: f32,
    pub score_delta: u32,
}

impl Default for LegacyFireworkConstants {
    fn default() -> Self {
        Self {
            duration: 0.55,
            sound_time: 0.2,
            score_delta: 200,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockAnimationConstants {
    pub animation_delay: f32,
    pub remove_frame: u32,
    pub floating_score: u32,
}

impl Default for LegacyCoinBlockAnimationConstants {
    fn default() -> Self {
        Self {
            animation_delay: 0.5 / 30.0,
            remove_frame: 31,
            floating_score: 200,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockDebrisConstants {
    pub animation_time: f32,
    pub gravity: f32,
    pub removal_y: f32,
}

impl Default for LegacyBlockDebrisConstants {
    fn default() -> Self {
        Self {
            animation_time: 0.1,
            gravity: 60.0,
            removal_y: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyScrollingScoreConstants {
    pub lifetime: f32,
    pub rise_height: f32,
    pub y_base_offset: f32,
    pub numeric_x_offset: f32,
}

impl Default for LegacyScrollingScoreConstants {
    fn default() -> Self {
        Self {
            lifetime: 0.8,
            rise_height: 2.5,
            y_base_offset: 1.5,
            numeric_x_offset: 0.4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockRewardConstants {
    pub score_delta: u32,
    pub coin_life_threshold: u32,
    pub many_coins_timer_duration: f32,
}

impl Default for LegacyCoinBlockRewardConstants {
    fn default() -> Self {
        Self {
            score_delta: 200,
            coin_life_threshold: 100,
            many_coins_timer_duration: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyKoopaConstants {
    pub speed: f32,
    pub shell_speed: f32,
    pub animation_speed: f32,
    pub jump_force: f32,
    pub flying_gravity: f32,
    pub normal_gravity: f32,
    pub friction: f32,
    pub shot_speed_x: f32,
    pub shot_jump_force: f32,
    pub shot_gravity: f32,
    pub rotation_alignment_speed: f32,
}

impl Default for LegacyKoopaConstants {
    fn default() -> Self {
        Self {
            speed: 2.0,
            shell_speed: 12.0,
            animation_speed: 0.2,
            jump_force: 10.0,
            flying_gravity: 30.0,
            normal_gravity: 80.0,
            friction: 14.0,
            shot_speed_x: 4.0,
            shot_jump_force: 8.0,
            shot_gravity: 60.0,
            rotation_alignment_speed: 15.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPlantConstants {
    pub in_time: f32,
    pub out_time: f32,
    pub animation_delay: f32,
    pub move_distance: f32,
    pub move_speed: f32,
}

impl Default for LegacyPlantConstants {
    fn default() -> Self {
        Self {
            in_time: 1.8,
            out_time: 2.0,
            animation_delay: 0.15,
            move_distance: 23.0 / 16.0,
            move_speed: 2.3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBulletBillConstants {
    pub lifetime: f32,
    pub speed: f32,
    pub launcher_time_min: f32,
    pub launcher_time_max: f32,
    pub range: f32,
    pub max_count: usize,
    pub shot_speed_x: f32,
    pub shot_gravity: f32,
    pub removal_y: f32,
}

impl Default for LegacyBulletBillConstants {
    fn default() -> Self {
        Self {
            lifetime: 20.0,
            speed: 8.0,
            launcher_time_min: 1.0,
            launcher_time_max: 4.5,
            range: 3.0,
            max_count: 5,
            shot_speed_x: 4.0,
            shot_gravity: 60.0,
            removal_y: 18.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlueGelBounceConstants, LegacyBlockDebrisConstants, LegacyBulletBillConstants,
        LegacyCastleFireConstants, LegacyCheepCheepConstants, LegacyCoinBlockAnimationConstants,
        LegacyCoinBlockRewardConstants, LegacyFireConstants, LegacyFireballConstants,
        LegacyFireworkConstants, LegacyFlyingFishConstants, LegacyGoombaConstants,
        LegacyHammerBroConstants, LegacyHammerConstants, LegacyKoopaConstants,
        LegacyLakitoConstants, LegacyPlantConstants, LegacyPowerUpConstants,
        LegacyScrollingScoreConstants, LegacySquidConstants, LegacyUpFireConstants,
        LegacyVineConstants, OrangeGelMovementConstants, PhysicsConstants,
        PlayerAnimationConstants, PlayerMovementConstants, SpringConstants,
        UnderwaterMovementConstants,
    };

    #[test]
    fn physics_constants_match_variables_lua() {
        let constants = PhysicsConstants::default();

        assert_eq!(constants.gravity, 80.0);
        assert_eq!(constants.jumping_gravity, 30.0);
        assert_eq!(constants.max_y_speed, 100.0);
        assert_eq!(constants.bounce_height, 0.875);
        assert_eq!(constants.space_run_room, 1.2 / 16.0);
    }

    #[test]
    fn player_movement_constants_match_variables_lua() {
        let constants = PlayerMovementConstants::default();

        assert_eq!(constants.walk_acceleration, 8.0);
        assert_eq!(constants.run_acceleration, 16.0);
        assert_eq!(constants.max_walk_speed, 6.4);
        assert_eq!(constants.max_run_speed, 9.0);
    }

    #[test]
    fn orange_gel_movement_constants_match_variables_lua() {
        let constants = OrangeGelMovementConstants::default();

        assert_eq!(constants.max_run_speed, 50.0);
        assert_eq!(constants.max_walk_speed, 25.0);
        assert_eq!(constants.run_acceleration, 25.0);
        assert_eq!(constants.walk_acceleration, 12.5);
    }

    #[test]
    fn blue_gel_bounce_constants_match_variables_lua() {
        let constants = BlueGelBounceConstants::default();

        assert_eq!(constants.floor_threshold_gravity_multiplier, 10.0);
        assert_eq!(constants.horizontal_multiplier, 1.5);
        assert_eq!(constants.horizontal_vertical_speed, 20.0);
        assert_eq!(constants.horizontal_max_speed_x, 15.0);
        assert_eq!(constants.horizontal_min_speed_x, 2.0);
    }

    #[test]
    fn player_animation_constants_match_variables_lua() {
        let constants = PlayerAnimationConstants::default();

        assert_eq!(constants.run_animation_speed, 10.0);
        assert_eq!(constants.swim_animation_speed, 10.0);
    }

    #[test]
    fn spring_constants_match_variables_lua() {
        let constants = SpringConstants::default();

        assert_eq!(constants.duration, 0.2);
        assert_eq!(constants.high_force, 41.0);
        assert_eq!(constants.force, 24.0);
        assert_eq!(constants.player_y_offset, 31.0 / 16.0);
    }

    #[test]
    fn legacy_vine_constants_match_variables_lua_and_vine_lua() {
        let constants = LegacyVineConstants::default();

        assert_eq!(constants.speed, 2.13);
        assert_eq!(constants.move_speed, 3.21);
        assert_eq!(constants.move_down_speed, 6.42);
        assert_eq!(constants.frame_delay, 0.15);
        assert_eq!(constants.frame_delay_down, 0.075);
        assert_eq!(constants.animation_start_y, 4.0);
        assert_eq!(constants.drop_x_offset, 7.0 / 16.0);
        assert_eq!(constants.start_limit, 9.0 + 1.0 / 16.0);
        assert_eq!(constants.regular_limit, -1.0);
        assert_eq!(constants.width, 10.0 / 16.0);
        assert_eq!(constants.spawn_y_offset, 1.0);
        assert_eq!(constants.growth_height_offset, 1.7);
    }

    #[test]
    fn legacy_power_up_constants_match_variables_lua_and_item_lua() {
        let constants = LegacyPowerUpConstants::default();

        assert_eq!(constants.mushroom_speed, 3.6);
        assert_eq!(constants.emergence_time, 0.7);
        assert_eq!(constants.mushroom_jump_force, 13.0);
        assert_eq!(constants.star_jump_force, 13.0);
        assert_eq!(constants.star_gravity, 40.0);
        assert_eq!(constants.star_animation_delay, 0.04);
        assert_eq!(constants.flower_gravity, 0.0);
        assert_eq!(constants.flower_emerged_y_offset, 27.0 / 16.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
        assert_eq!(constants.one_up_offscreen_y, 18.0);
        assert_eq!(constants.one_up_offscreen_left_margin, 3.0);
    }

    #[test]
    fn underwater_movement_constants_match_variables_lua() {
        let constants = UnderwaterMovementConstants::default();

        assert_eq!(constants.walk_acceleration, 8.0);
        assert_eq!(constants.run_acceleration, 16.0);
        assert_eq!(constants.max_air_walk_speed, 5.0);
        assert_eq!(constants.max_walk_speed, 3.6);
        assert_eq!(constants.max_run_speed, 5.0);
        assert_eq!(constants.gravity, 9.0);
        assert_eq!(constants.jumping_gravity, 12.0);
        assert_eq!(constants.max_height, 2.5);
        assert_eq!(constants.push_down_speed, 3.0);
    }

    #[test]
    fn legacy_goomba_constants_match_variables_lua() {
        let constants = LegacyGoombaConstants::default();

        assert_eq!(constants.speed, 2.0);
        assert_eq!(constants.animation_speed, 0.2);
        assert_eq!(constants.death_time, 0.5);
        assert_eq!(constants.friction, 14.0);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_cheep_cheep_constants_match_variables_lua() {
        let constants = LegacyCheepCheepConstants::default();

        assert_eq!(constants.white_speed, 1.0);
        assert_eq!(constants.red_speed, 1.8);
        assert_eq!(constants.vertical_speed, 0.3);
        assert_eq!(constants.vertical_range, 1.0);
        assert_eq!(constants.animation_speed, 0.35);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_squid_constants_match_variables_lua() {
        let constants = LegacySquidConstants::default();

        assert_eq!(constants.fall_speed, 0.9);
        assert_eq!(constants.horizontal_speed, 3.0);
        assert_eq!(constants.up_speed, 3.0);
        assert_eq!(constants.acceleration, 10.0);
        assert_eq!(constants.down_distance, 1.0);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_lakito_constants_match_variables_lua() {
        let constants = LegacyLakitoConstants::default();

        assert_eq!(constants.throw_time, 4.0);
        assert_eq!(constants.respawn_time, 16.0);
        assert_eq!(constants.space, 4.0);
        assert_eq!(constants.distance_time, 1.5);
        assert_eq!(constants.hide_time, 0.5);
        assert_eq!(constants.passive_speed, 3.0);
        assert_eq!(constants.max_spikey_count, 3);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
    }

    #[test]
    fn legacy_hammer_constants_match_variables_lua() {
        let constants = LegacyHammerConstants::default();

        assert_eq!(constants.speed, 4.0);
        assert_eq!(constants.start_y_speed, 8.0);
        assert_eq!(constants.gravity, 25.0);
        assert_eq!(constants.animation_speed, 0.05);
    }

    #[test]
    fn legacy_hammer_bro_constants_match_variables_lua() {
        let constants = LegacyHammerBroConstants::default();

        assert_eq!(constants.prepare_time, 0.5);
        assert_eq!(constants.throw_time_short, 0.6);
        assert_eq!(constants.throw_time_long, 1.6);
        assert_eq!(constants.speed, 1.5);
        assert_eq!(constants.animation_speed, 0.15);
        assert_eq!(constants.jump_time, 3.0);
        assert_eq!(constants.jump_force, 19.0);
        assert_eq!(constants.jump_force_down, 6.0);
        assert_eq!(constants.gravity, 40.0);
        assert_eq!(constants.friction, 14.0);
        assert_eq!(constants.side_collision_speed, 2.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.shot_removal_y, 18.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_flying_fish_constants_match_variables_lua() {
        let constants = LegacyFlyingFishConstants::default();

        assert_eq!(constants.gravity, 20.0);
        assert_eq!(constants.jump_force, 23.0);
        assert_eq!(constants.animation_speed, 0.35);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_fireball_constants_match_variables_lua() {
        let constants = LegacyFireballConstants::default();

        assert_eq!(constants.speed, 15.0);
        assert_eq!(constants.jump_force, 10.0);
        assert_eq!(constants.max_count, 2);
        assert_eq!(constants.animation_delay, 0.04);
        assert_eq!(constants.offscreen_y, 15.0);
    }

    #[test]
    fn legacy_fire_constants_match_variables_lua() {
        let constants = LegacyFireConstants::default();

        assert_eq!(constants.speed, 4.69);
        assert_eq!(constants.vertical_speed, 2.0);
        assert_eq!(constants.animation_delay, 0.05);
    }

    #[test]
    fn legacy_up_fire_constants_match_variables_lua() {
        let constants = LegacyUpFireConstants::default();

        assert_eq!(constants.start_y_offset, 8.0);
        assert_eq!(constants.force, 19.0);
        assert_eq!(constants.gravity, 20.0);
        assert_eq!(constants.hide_y, 15.0);
    }

    #[test]
    fn legacy_castle_fire_constants_match_variables_lua() {
        let constants = LegacyCastleFireConstants::default();

        assert_eq!(constants.angle_step_degrees, 11.25);
        assert_eq!(constants.angle_delay, 3.4 / (360.0 / 11.25));
        assert_eq!(constants.animation_delay, 0.07);
    }

    #[test]
    fn legacy_firework_constants_match_variables_lua_and_firework_lua() {
        let constants = LegacyFireworkConstants::default();

        assert_eq!(constants.duration, 0.55);
        assert_eq!(constants.sound_time, 0.2);
        assert_eq!(constants.score_delta, 200);
    }

    #[test]
    fn legacy_coin_block_animation_constants_match_variables_lua_and_coinblockanimation_lua() {
        let constants = LegacyCoinBlockAnimationConstants::default();

        assert_eq!(constants.animation_delay, 0.5 / 30.0);
        assert_eq!(constants.remove_frame, 31);
        assert_eq!(constants.floating_score, 200);
    }

    #[test]
    fn legacy_block_debris_constants_match_variables_lua_and_blockdebris_lua() {
        let constants = LegacyBlockDebrisConstants::default();

        assert_eq!(constants.animation_time, 0.1);
        assert_eq!(constants.gravity, 60.0);
        assert_eq!(constants.removal_y, 15.0);
    }

    #[test]
    fn legacy_scrolling_score_constants_match_variables_lua_and_game_lua_draw_offsets() {
        let constants = LegacyScrollingScoreConstants::default();

        assert_eq!(constants.lifetime, 0.8);
        assert_eq!(constants.rise_height, 2.5);
        assert_eq!(constants.y_base_offset, 1.5);
        assert_eq!(constants.numeric_x_offset, 0.4);
    }

    #[test]
    fn legacy_coin_block_reward_constants_match_variables_lua_and_mario_lua() {
        let constants = LegacyCoinBlockRewardConstants::default();

        assert_eq!(constants.score_delta, 200);
        assert_eq!(constants.coin_life_threshold, 100);
        assert_eq!(constants.many_coins_timer_duration, 4.0);
    }

    #[test]
    fn legacy_koopa_constants_match_variables_lua() {
        let constants = LegacyKoopaConstants::default();

        assert_eq!(constants.speed, 2.0);
        assert_eq!(constants.shell_speed, 12.0);
        assert_eq!(constants.animation_speed, 0.2);
        assert_eq!(constants.jump_force, 10.0);
        assert_eq!(constants.flying_gravity, 30.0);
        assert_eq!(constants.normal_gravity, 80.0);
        assert_eq!(constants.friction, 14.0);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_jump_force, 8.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.rotation_alignment_speed, 15.0);
    }

    #[test]
    fn legacy_plant_constants_match_variables_lua() {
        let constants = LegacyPlantConstants::default();

        assert_eq!(constants.in_time, 1.8);
        assert_eq!(constants.out_time, 2.0);
        assert_eq!(constants.animation_delay, 0.15);
        assert_eq!(constants.move_distance, 23.0 / 16.0);
        assert_eq!(constants.move_speed, 2.3);
    }

    #[test]
    fn legacy_bullet_bill_constants_match_variables_and_bulletbill_lua() {
        let constants = LegacyBulletBillConstants::default();

        assert_eq!(constants.lifetime, 20.0);
        assert_eq!(constants.speed, 8.0);
        assert_eq!(constants.launcher_time_min, 1.0);
        assert_eq!(constants.launcher_time_max, 4.5);
        assert_eq!(constants.range, 3.0);
        assert_eq!(constants.max_count, 5);
        assert_eq!(constants.shot_speed_x, 4.0);
        assert_eq!(constants.shot_gravity, 60.0);
        assert_eq!(constants.removal_y, 18.0);
    }
}

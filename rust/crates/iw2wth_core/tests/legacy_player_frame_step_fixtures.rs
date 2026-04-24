use iw2wth_core::{
    LegacyOnVineContext, LegacyOnVineDirection, LegacyOnVineUpdate, LegacyVineConstants,
    PlayerAnimationState, PlayerMovementState, PlayerVerticalBounds, update_legacy_on_vine_motion,
};

const FIXTURES: &str = include_str!("fixtures/legacy_player_frame_steps.generated.tsv");
const EPSILON: f32 = 0.0001;

#[test]
fn legacy_on_vine_frame_steps_match_generated_lua_fixtures() {
    let mut fixture_count = 0;

    for (line_number, line) in FIXTURES.lines().enumerate() {
        let line_number = line_number + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fixture = parse_fixture(line_number, line);
        let mut state = PlayerMovementState::default();
        let actual = update_legacy_on_vine_motion(
            &mut state,
            fixture.context,
            LegacyVineConstants::default(),
            fixture.dt,
        );

        assert_eq!(
            state.animation_state,
            PlayerAnimationState::Climbing,
            "{} animation state",
            fixture.name
        );
        assert_update_close(&fixture.name, actual, fixture.expected);
        fixture_count += 1;
    }

    assert_eq!(
        fixture_count, 4,
        "fixture table should cover up, down, idle, and blocked down-vine frame steps"
    );
}

#[derive(Clone, Debug)]
struct Fixture {
    name: String,
    context: LegacyOnVineContext,
    dt: f32,
    expected: LegacyOnVineUpdate,
}

fn parse_fixture(line_number: usize, line: &str) -> Fixture {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    assert_eq!(
        columns.len(),
        14,
        "fixture line {line_number} should have 14 columns"
    );

    let name = columns[0].to_owned();
    let y = parse_f32(line_number, columns[1]);
    let height = parse_f32(line_number, columns[2]);
    let move_timer = parse_f32(line_number, columns[3]);
    let direction = parse_direction(line_number, columns[4]);
    let block_y = parse_optional_f32(line_number, columns[5]);
    let block_height = parse_optional_f32(line_number, columns[6]);
    let dt = parse_f32(line_number, columns[7]);
    let expected_y = parse_f32(line_number, columns[8]);
    let expected_move_timer = parse_f32(line_number, columns[9]);
    let expected_climb_frame = parse_u8(line_number, columns[10]);
    let expected_trigger_animation = parse_bool(line_number, columns[11]);
    let expected_portal_probe_y = parse_optional_f32(line_number, columns[12]);
    let expected_blocked_by_solid = parse_bool(line_number, columns[13]);

    let blocking_collision = match (block_y, block_height) {
        (Some(block_y), Some(block_height)) => {
            Some(PlayerVerticalBounds::new(block_y, block_height))
        }
        (None, None) => None,
        _ => panic!("fixture line {line_number} block_y and block_height must both be set or none"),
    };

    Fixture {
        name,
        context: LegacyOnVineContext {
            y,
            height,
            move_timer,
            direction,
            blocking_collision,
        },
        dt,
        expected: LegacyOnVineUpdate {
            y: expected_y,
            move_timer: expected_move_timer,
            climb_frame: expected_climb_frame,
            trigger_animation: expected_trigger_animation,
            portal_probe_y: expected_portal_probe_y,
            blocked_by_solid: expected_blocked_by_solid,
        },
    }
}

fn parse_direction(line_number: usize, value: &str) -> LegacyOnVineDirection {
    match value {
        "up" => LegacyOnVineDirection::Up,
        "down" => LegacyOnVineDirection::Down,
        "idle" => LegacyOnVineDirection::Idle,
        _ => panic!("invalid direction `{value}` on fixture line {line_number}"),
    }
}

fn parse_bool(line_number: usize, value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("invalid bool `{value}` on fixture line {line_number}"),
    }
}

fn parse_optional_f32(line_number: usize, value: &str) -> Option<f32> {
    if value == "none" {
        None
    } else {
        Some(parse_f32(line_number, value))
    }
}

fn parse_f32(line_number: usize, value: &str) -> f32 {
    match value.parse::<f32>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("invalid f32 `{value}` on fixture line {line_number}: {error}"),
    }
}

fn parse_u8(line_number: usize, value: &str) -> u8 {
    match value.parse::<u8>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("invalid u8 `{value}` on fixture line {line_number}: {error}"),
    }
}

fn assert_update_close(name: &str, actual: LegacyOnVineUpdate, expected: LegacyOnVineUpdate) {
    assert_close(name, "y", actual.y, expected.y);
    assert_close(name, "move_timer", actual.move_timer, expected.move_timer);
    assert_eq!(
        actual.climb_frame, expected.climb_frame,
        "{name} climb_frame"
    );
    assert_eq!(
        actual.trigger_animation, expected.trigger_animation,
        "{name} trigger_animation"
    );
    assert_optional_close(
        name,
        "portal_probe_y",
        actual.portal_probe_y,
        expected.portal_probe_y,
    );
    assert_eq!(
        actual.blocked_by_solid, expected.blocked_by_solid,
        "{name} blocked_by_solid"
    );
}

fn assert_optional_close(name: &str, field: &str, actual: Option<f32>, expected: Option<f32>) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => assert_close(name, field, actual, expected),
        (None, None) => {}
        _ => panic!("{name} {field}: expected {actual:?} to match {expected:?}"),
    }
}

fn assert_close(name: &str, field: &str, actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "{name} {field}: expected {actual} to be close to {expected}"
    );
}

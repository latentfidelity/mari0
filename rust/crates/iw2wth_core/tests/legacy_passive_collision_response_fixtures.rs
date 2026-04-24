use iw2wth_core::{
    LegacyCollisionActor, LegacyCollisionHandlerResult, LegacyCollisionTarget,
    LegacyPassiveCollisionResponse, Vec2, legacy_passive_collision_response,
};

const FIXTURES: &str = include_str!("fixtures/legacy_passive_collision_responses.generated.tsv");
const EPSILON: f32 = 0.0001;

#[test]
fn legacy_passive_collision_responses_match_generated_lua_fixtures() {
    let mut fixture_count = 0;
    let mut passive_handler_count = 0;
    let mut default_floor_count = 0;
    let mut suppressed_floor_count = 0;

    for (line_number, line) in FIXTURES.lines().enumerate() {
        let line_number = line_number + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fixture = parse_fixture(line_number, line);
        let actual = legacy_passive_collision_response(
            fixture.moving,
            fixture.target,
            fixture.moving_passive_handler,
            fixture.target_passive_handler,
            fixture.floor_handler,
        );

        if fixture.moving_passive_handler {
            passive_handler_count += 1;
        } else if fixture.floor_handler == Some(LegacyCollisionHandlerResult::SuppressDefault) {
            suppressed_floor_count += 1;
        } else {
            default_floor_count += 1;
        }

        assert_response_close(&fixture.name, actual, fixture.expected);
        fixture_count += 1;
    }

    assert_eq!(
        fixture_count, 5,
        "fixture table should cover passive dispatch plus default and suppressed floor handling"
    );
    assert_eq!(
        passive_handler_count, 2,
        "fixture table should cover passive dispatch with and without a target handler"
    );
    assert_eq!(
        default_floor_count, 2,
        "fixture table should cover default floor snapping with downward and upward speeds"
    );
    assert_eq!(
        suppressed_floor_count, 1,
        "fixture table should cover Lua false floor handler suppression"
    );
}

#[derive(Clone, Debug)]
struct Fixture {
    name: String,
    moving: LegacyCollisionActor,
    target: LegacyCollisionTarget,
    moving_passive_handler: bool,
    target_passive_handler: bool,
    floor_handler: Option<LegacyCollisionHandlerResult>,
    expected: LegacyPassiveCollisionResponse,
}

fn parse_fixture(line_number: usize, line: &str) -> Fixture {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    assert_eq!(
        columns.len(),
        25,
        "fixture line {line_number} should have 25 columns"
    );

    let name = columns[0].to_owned();
    let moving_x = parse_f32(line_number, columns[1]);
    let moving_y = parse_f32(line_number, columns[2]);
    let moving_width = parse_f32(line_number, columns[3]);
    let moving_height = parse_f32(line_number, columns[4]);
    let moving_speed_x = parse_f32(line_number, columns[5]);
    let moving_speed_y = parse_f32(line_number, columns[6]);
    let target_x = parse_f32(line_number, columns[7]);
    let target_y = parse_f32(line_number, columns[8]);
    let target_width = parse_f32(line_number, columns[9]);
    let target_height = parse_f32(line_number, columns[10]);
    let target_speed_x = parse_optional_f32(line_number, columns[11]);
    let target_speed_y = parse_optional_f32(line_number, columns[12]);
    let moving_passive_handler = parse_bool(line_number, columns[13]);
    let target_passive_handler = parse_bool(line_number, columns[14]);
    let floor_handler = parse_floor_handler(line_number, columns[15]);
    let expected_moving_x = parse_f32(line_number, columns[16]);
    let expected_moving_y = parse_f32(line_number, columns[17]);
    let expected_moving_speed_x = parse_f32(line_number, columns[18]);
    let expected_moving_speed_y = parse_f32(line_number, columns[19]);
    let expected_target_speed_x = parse_optional_f32(line_number, columns[20]);
    let expected_target_speed_y = parse_optional_f32(line_number, columns[21]);
    let expected_resolved = parse_bool(line_number, columns[22]);
    let expected_moving_passive_called = parse_bool(line_number, columns[23]);
    let expected_target_passive_called = parse_bool(line_number, columns[24]);

    Fixture {
        name,
        moving: LegacyCollisionActor::new(
            moving_x,
            moving_y,
            moving_width,
            moving_height,
            moving_speed_x,
            moving_speed_y,
        ),
        target: LegacyCollisionTarget::new(
            target_x,
            target_y,
            target_width,
            target_height,
            make_velocity(line_number, target_speed_x, target_speed_y),
        ),
        moving_passive_handler,
        target_passive_handler,
        floor_handler,
        expected: LegacyPassiveCollisionResponse {
            moving: LegacyCollisionActor::new(
                expected_moving_x,
                expected_moving_y,
                moving_width,
                moving_height,
                expected_moving_speed_x,
                expected_moving_speed_y,
            ),
            target_velocity: make_velocity(
                line_number,
                expected_target_speed_x,
                expected_target_speed_y,
            ),
            resolved: expected_resolved,
            moving_passive_called: expected_moving_passive_called,
            target_passive_called: expected_target_passive_called,
        },
    }
}

fn parse_floor_handler(line_number: usize, value: &str) -> Option<LegacyCollisionHandlerResult> {
    match value {
        "none" => None,
        "apply" => Some(LegacyCollisionHandlerResult::ApplyDefault),
        "suppress" => Some(LegacyCollisionHandlerResult::SuppressDefault),
        _ => panic!("invalid floor handler `{value}` on fixture line {line_number}"),
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

fn make_velocity(line_number: usize, speed_x: Option<f32>, speed_y: Option<f32>) -> Option<Vec2> {
    match (speed_x, speed_y) {
        (Some(speed_x), Some(speed_y)) => Some(Vec2::new(speed_x, speed_y)),
        (None, None) => None,
        _ => panic!("fixture line {line_number} velocity components must both be set or none"),
    }
}

fn assert_response_close(
    name: &str,
    actual: LegacyPassiveCollisionResponse,
    expected: LegacyPassiveCollisionResponse,
) {
    assert_vec_close(
        name,
        "moving.bounds.min",
        actual.moving.bounds.min,
        expected.moving.bounds.min,
    );
    assert_vec_close(
        name,
        "moving.bounds.max",
        actual.moving.bounds.max,
        expected.moving.bounds.max,
    );
    assert_vec_close(
        name,
        "moving.velocity",
        actual.moving.velocity,
        expected.moving.velocity,
    );
    assert_optional_vec_close(
        name,
        "target_velocity",
        actual.target_velocity,
        expected.target_velocity,
    );
    assert_eq!(actual.resolved, expected.resolved, "{name} resolved");
    assert_eq!(
        actual.moving_passive_called, expected.moving_passive_called,
        "{name} moving_passive_called"
    );
    assert_eq!(
        actual.target_passive_called, expected.target_passive_called,
        "{name} target_passive_called"
    );
}

fn assert_optional_vec_close(
    name: &str,
    field: &str,
    actual: Option<Vec2>,
    expected: Option<Vec2>,
) {
    match (actual, expected) {
        (Some(actual), Some(expected)) => assert_vec_close(name, field, actual, expected),
        (None, None) => {}
        _ => panic!("{name} {field}: expected {actual:?} to match {expected:?}"),
    }
}

fn assert_vec_close(name: &str, field: &str, actual: Vec2, expected: Vec2) {
    assert_close(name, &format!("{field}.x"), actual.x, expected.x);
    assert_close(name, &format!("{field}.y"), actual.y, expected.y);
}

fn assert_close(name: &str, field: &str, actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "{name} {field}: expected {actual} to be close to {expected}"
    );
}

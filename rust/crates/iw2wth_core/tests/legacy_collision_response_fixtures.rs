use iw2wth_core::{
    LegacyCollisionActor, LegacyCollisionHandlerResult, LegacyCollisionResponse,
    LegacyCollisionTarget, Vec2, legacy_horizontal_collision_response,
    legacy_vertical_collision_response,
};

const FIXTURES: &str = include_str!("fixtures/legacy_collision_responses.generated.tsv");
const EPSILON: f32 = 0.0001;

#[test]
fn legacy_collision_responses_match_generated_lua_fixtures() {
    let mut fixture_count = 0;
    let mut horizontal_count = 0;
    let mut vertical_count = 0;

    for (line_number, line) in FIXTURES.lines().enumerate() {
        let line_number = line_number + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fixture = parse_fixture(line_number, line);
        let actual = if fixture.name.starts_with("horizontal_") {
            horizontal_count += 1;
            legacy_horizontal_collision_response(
                fixture.moving,
                fixture.target,
                fixture.moving_handler,
                fixture.target_handler,
            )
        } else if fixture.name.starts_with("vertical_") {
            vertical_count += 1;
            legacy_vertical_collision_response(
                fixture.moving,
                fixture.target,
                fixture.moving_handler,
                fixture.target_handler,
            )
        } else {
            panic!(
                "fixture line {line_number} has unsupported collision axis in `{}`",
                fixture.name
            );
        };

        assert_response_close(&fixture.name, actual, fixture.expected);
        fixture_count += 1;
    }

    assert_eq!(
        fixture_count, 10,
        "fixture table should cover horizontal left/right movement plus vertical up/down movement"
    );
    assert_eq!(
        horizontal_count, 5,
        "fixture table should cover horizontal movement, Lua false handlers, and static targets"
    );
    assert_eq!(
        vertical_count, 5,
        "fixture table should cover vertical movement, Lua false handlers, and static targets"
    );
}

#[derive(Clone, Debug)]
struct Fixture {
    name: String,
    moving: LegacyCollisionActor,
    target: LegacyCollisionTarget,
    moving_handler: LegacyCollisionHandlerResult,
    target_handler: LegacyCollisionHandlerResult,
    expected: LegacyCollisionResponse,
}

fn parse_fixture(line_number: usize, line: &str) -> Fixture {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    assert_eq!(
        columns.len(),
        22,
        "fixture line {line_number} should have 22 columns"
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
    let moving_handler = parse_handler(line_number, columns[13]);
    let target_handler = parse_handler(line_number, columns[14]);
    let expected_moving_x = parse_f32(line_number, columns[15]);
    let expected_moving_y = parse_f32(line_number, columns[16]);
    let expected_moving_speed_x = parse_f32(line_number, columns[17]);
    let expected_moving_speed_y = parse_f32(line_number, columns[18]);
    let expected_target_speed_x = parse_optional_f32(line_number, columns[19]);
    let expected_target_speed_y = parse_optional_f32(line_number, columns[20]);
    let expected_resolved = parse_bool(line_number, columns[21]);

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
        moving_handler,
        target_handler,
        expected: LegacyCollisionResponse {
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
        },
    }
}

fn parse_handler(line_number: usize, value: &str) -> LegacyCollisionHandlerResult {
    match value {
        "apply" => LegacyCollisionHandlerResult::ApplyDefault,
        "suppress" => LegacyCollisionHandlerResult::SuppressDefault,
        _ => panic!("invalid handler `{value}` on fixture line {line_number}"),
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
    actual: LegacyCollisionResponse,
    expected: LegacyCollisionResponse,
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

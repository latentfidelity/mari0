use iw2wth_core::{
    AnimationDirection, Facing, LegacyPortalEndpoint, LegacyPortalTransit,
    LegacyPortalTransitInput, Vec2, legacy_portal_coords,
};

const FIXTURES: &str = include_str!("fixtures/legacy_portalcoords.generated.tsv");
const EPSILON: f32 = 0.0001;

#[test]
fn legacy_portalcoords_matches_generated_lua_fixtures() {
    let mut fixture_count = 0;

    for (line_number, line) in FIXTURES.lines().enumerate() {
        let line_number = line_number + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fixture = parse_fixture(line_number, line);
        let actual = legacy_portal_coords(fixture.input);
        assert_transit_close(&fixture.name, actual, fixture.expected);
        fixture_count += 1;
    }

    assert_eq!(
        fixture_count, 16,
        "fixture table should cover all facing pairs"
    );
}

#[derive(Clone, Debug)]
struct Fixture {
    name: String,
    input: LegacyPortalTransitInput,
    expected: LegacyPortalTransit,
}

fn parse_fixture(line_number: usize, line: &str) -> Fixture {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    assert_eq!(
        columns.len(),
        24,
        "fixture line {line_number} should have 24 columns"
    );

    let name = columns[0].to_owned();
    let entry_facing = parse_facing(line_number, columns[1]);
    let exit_facing = parse_facing(line_number, columns[2]);
    let x = parse_f32(line_number, columns[3]);
    let y = parse_f32(line_number, columns[4]);
    let speed_x = parse_f32(line_number, columns[5]);
    let speed_y = parse_f32(line_number, columns[6]);
    let width = parse_f32(line_number, columns[7]);
    let height = parse_f32(line_number, columns[8]);
    let rotation = parse_f32(line_number, columns[9]);
    let animation_direction = parse_animation_direction(line_number, columns[10]);
    let entry_x = parse_f32(line_number, columns[11]);
    let entry_y = parse_f32(line_number, columns[12]);
    let exit_x = parse_f32(line_number, columns[13]);
    let exit_y = parse_f32(line_number, columns[14]);
    let live = parse_bool(line_number, columns[15]);
    let gravity = parse_f32(line_number, columns[16]);
    let frame_dt = parse_f32(line_number, columns[17]);
    let expected_x = parse_f32(line_number, columns[18]);
    let expected_y = parse_f32(line_number, columns[19]);
    let expected_speed_x = parse_f32(line_number, columns[20]);
    let expected_speed_y = parse_f32(line_number, columns[21]);
    let expected_rotation = parse_f32(line_number, columns[22]);
    let expected_animation_direction = parse_animation_direction(line_number, columns[23]);

    Fixture {
        name,
        input: LegacyPortalTransitInput {
            position: Vec2::new(x, y),
            velocity: Vec2::new(speed_x, speed_y),
            size: Vec2::new(width, height),
            rotation,
            animation_direction,
            entry: LegacyPortalEndpoint::new(entry_x, entry_y, entry_facing),
            exit: LegacyPortalEndpoint::new(exit_x, exit_y, exit_facing),
            live,
            gravity,
            frame_dt,
        },
        expected: LegacyPortalTransit {
            position: Vec2::new(expected_x, expected_y),
            velocity: Vec2::new(expected_speed_x, expected_speed_y),
            rotation: expected_rotation,
            animation_direction: expected_animation_direction,
        },
    }
}

fn parse_facing(line_number: usize, value: &str) -> Facing {
    match value {
        "up" => Facing::Up,
        "right" => Facing::Right,
        "down" => Facing::Down,
        "left" => Facing::Left,
        _ => panic!("invalid facing `{value}` on fixture line {line_number}"),
    }
}

fn parse_animation_direction(line_number: usize, value: &str) -> Option<AnimationDirection> {
    match value {
        "left" => Some(AnimationDirection::Left),
        "right" => Some(AnimationDirection::Right),
        "none" => None,
        _ => panic!("invalid animation direction `{value}` on fixture line {line_number}"),
    }
}

fn parse_bool(line_number: usize, value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("invalid bool `{value}` on fixture line {line_number}"),
    }
}

fn parse_f32(line_number: usize, value: &str) -> f32 {
    match value.parse::<f32>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("invalid f32 `{value}` on fixture line {line_number}: {error}"),
    }
}

fn assert_transit_close(name: &str, actual: LegacyPortalTransit, expected: LegacyPortalTransit) {
    assert_vec_close(name, "position", actual.position, expected.position);
    assert_vec_close(name, "velocity", actual.velocity, expected.velocity);
    assert_close(name, "rotation", actual.rotation, expected.rotation);
    assert_eq!(
        actual.animation_direction, expected.animation_direction,
        "{name} animation direction"
    );
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

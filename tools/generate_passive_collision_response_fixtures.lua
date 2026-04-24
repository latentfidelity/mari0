-- Generates Rust-consumed fixtures for physics.lua passivecollision default
-- response without loading LOVE. The formulas mirror the deterministic branch
-- that dispatches passive handlers or falls back to floor collision snapping.

local function option(value)
	if value == nil then
		return "none"
	end
	return string.format("%.12g", value)
end

local function bool(value)
	if value then
		return "true"
	end
	return "false"
end

local function floor_applies(handler)
	return handler == nil or handler == "apply"
end

local cases = {
	{
		name = "passive_moving_handler_calls_target_handler",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.5,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_passive_handler = true,
		target_passive_handler = true,
		floor_handler = nil,
	},
	{
		name = "passive_moving_handler_without_target_handler",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.5,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_passive_handler = true,
		target_passive_handler = false,
		floor_handler = nil,
	},
	{
		name = "passive_default_floor_snap_downward",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.5,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_passive_handler = false,
		target_passive_handler = true,
		floor_handler = nil,
	},
	{
		name = "passive_floor_handler_apply_upward_preserves_speed",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = -3.0,
		target_x = 2.0,
		target_y = 4.5,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = nil,
		target_speed_y = nil,
		moving_passive_handler = false,
		target_passive_handler = false,
		floor_handler = "apply",
	},
	{
		name = "passive_floor_handler_false_suppresses_default",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.5,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_passive_handler = false,
		target_passive_handler = true,
		floor_handler = "suppress",
	},
}

local function step(case)
	local moving_x = case.moving_x
	local moving_y = case.moving_y
	local moving_speed_x = case.moving_speed_x
	local moving_speed_y = case.moving_speed_y
	local resolved = false
	local moving_passive_called = false
	local target_passive_called = false

	if case.moving_passive_handler then
		moving_passive_called = true
		target_passive_called = case.target_passive_handler
	elseif floor_applies(case.floor_handler) then
		if moving_speed_y > 0 then
			moving_speed_y = 0
		end
		moving_y = case.target_y - case.moving_height
		resolved = true
	end

	return {
		moving_x = moving_x,
		moving_y = moving_y,
		moving_speed_x = moving_speed_x,
		moving_speed_y = moving_speed_y,
		target_speed_x = case.target_speed_x,
		target_speed_y = case.target_speed_y,
		resolved = resolved,
		moving_passive_called = moving_passive_called,
		target_passive_called = target_passive_called,
	}
end

print("# Generated from physics.lua passivecollision by tools/generate_passive_collision_response_fixtures.lua.")
print("# case moving_x moving_y moving_width moving_height moving_speed_x moving_speed_y target_x target_y target_width target_height target_speed_x target_speed_y moving_passive_handler target_passive_handler floor_handler expected_moving_x expected_moving_y expected_moving_speed_x expected_moving_speed_y expected_target_speed_x expected_target_speed_y expected_resolved expected_moving_passive_called expected_target_passive_called")

for _, case in ipairs(cases) do
	local expected = step(case)
	print(string.format(
		"%s %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %s %s %s %s %s %.12g %.12g %.12g %.12g %s %s %s %s %s",
		case.name,
		case.moving_x,
		case.moving_y,
		case.moving_width,
		case.moving_height,
		case.moving_speed_x,
		case.moving_speed_y,
		case.target_x,
		case.target_y,
		case.target_width,
		case.target_height,
		option(case.target_speed_x),
		option(case.target_speed_y),
		bool(case.moving_passive_handler),
		bool(case.target_passive_handler),
		case.floor_handler or "none",
		expected.moving_x,
		expected.moving_y,
		expected.moving_speed_x,
		expected.moving_speed_y,
		option(expected.target_speed_x),
		option(expected.target_speed_y),
		bool(expected.resolved),
		bool(expected.moving_passive_called),
		bool(expected.target_passive_called)
	))
end

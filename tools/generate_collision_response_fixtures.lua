-- Generates Rust-consumed fixtures for physics.lua horizontal and vertical
-- default collision responses without loading LOVE. The formulas mirror the
-- deterministic horcollision/vercollision branches that snap the moving object
-- and zero inward speeds.

local function handler_applies(handler)
	return handler ~= "suppress"
end

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

local cases = {
	{
		name = "horizontal_left_default",
		axis = "horizontal",
		moving_x = 4.0,
		moving_y = 2.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = -6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_handler = "apply",
		target_handler = "apply",
	},
	{
		name = "horizontal_right_default",
		axis = "horizontal",
		moving_x = 2.0,
		moving_y = 2.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 4.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = -4.0,
		target_speed_y = 9.0,
		moving_handler = "apply",
		target_handler = "apply",
	},
	{
		name = "horizontal_moving_handler_false",
		axis = "horizontal",
		moving_x = 2.0,
		moving_y = 2.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 4.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = -4.0,
		target_speed_y = 9.0,
		moving_handler = "suppress",
		target_handler = "apply",
	},
	{
		name = "horizontal_target_handler_false",
		axis = "horizontal",
		moving_x = 2.0,
		moving_y = 2.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 4.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = -4.0,
		target_speed_y = 9.0,
		moving_handler = "apply",
		target_handler = "suppress",
	},
	{
		name = "horizontal_static_target",
		axis = "horizontal",
		moving_x = 2.0,
		moving_y = 2.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 0.0,
		moving_speed_y = 3.0,
		target_x = 4.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = nil,
		target_speed_y = nil,
		moving_handler = "apply",
		target_handler = "apply",
	},
	{
		name = "vertical_up_default",
		axis = "vertical",
		moving_x = 2.0,
		moving_y = 4.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = -3.0,
		target_x = 2.0,
		target_y = 2.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = 9.0,
		moving_handler = "apply",
		target_handler = "apply",
	},
	{
		name = "vertical_down_default",
		axis = "vertical",
		moving_x = 2.0,
		moving_y = 1.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = -9.0,
		moving_handler = "apply",
		target_handler = "apply",
	},
	{
		name = "vertical_moving_handler_false",
		axis = "vertical",
		moving_x = 2.0,
		moving_y = 1.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = -9.0,
		moving_handler = "suppress",
		target_handler = "apply",
	},
	{
		name = "vertical_target_handler_false",
		axis = "vertical",
		moving_x = 2.0,
		moving_y = 1.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 3.0,
		target_x = 2.0,
		target_y = 4.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = 4.0,
		target_speed_y = -9.0,
		moving_handler = "apply",
		target_handler = "suppress",
	},
	{
		name = "vertical_static_target",
		axis = "vertical",
		moving_x = 2.0,
		moving_y = 1.0,
		moving_width = 1.0,
		moving_height = 1.5,
		moving_speed_x = 6.0,
		moving_speed_y = 0.0,
		target_x = 2.0,
		target_y = 4.0,
		target_width = 1.25,
		target_height = 2.0,
		target_speed_x = nil,
		target_speed_y = nil,
		moving_handler = "apply",
		target_handler = "apply",
	},
}

local function step(case)
	local moving_x = case.moving_x
	local moving_y = case.moving_y
	local moving_speed_x = case.moving_speed_x
	local moving_speed_y = case.moving_speed_y
	local target_speed_x = case.target_speed_x
	local target_speed_y = case.target_speed_y
	local resolved = false

	if case.axis == "horizontal" and case.moving_speed_x < 0 then
		if handler_applies(case.target_handler) and target_speed_x ~= nil and target_speed_x > 0 then
			target_speed_x = 0
		end

		if handler_applies(case.moving_handler) then
			if moving_speed_x < 0 then
				moving_speed_x = 0
			end
			moving_x = case.target_x + case.target_width
			resolved = true
		end
	elseif case.axis == "horizontal" then
		if handler_applies(case.target_handler) and target_speed_x ~= nil and target_speed_x < 0 then
			target_speed_x = 0
		end

		if handler_applies(case.moving_handler) then
			if moving_speed_x > 0 then
				moving_speed_x = 0
			end
			moving_x = case.target_x - case.moving_width
			resolved = true
		end
	elseif case.moving_speed_y < 0 then
		if handler_applies(case.target_handler) and target_speed_y ~= nil and target_speed_y > 0 then
			target_speed_y = 0
		end

		if handler_applies(case.moving_handler) then
			if moving_speed_y < 0 then
				moving_speed_y = 0
			end
			moving_y = case.target_y + case.target_height
			resolved = true
		end
	else
		if handler_applies(case.target_handler) and target_speed_y ~= nil and target_speed_y < 0 then
			target_speed_y = 0
		end

		if handler_applies(case.moving_handler) then
			if moving_speed_y > 0 then
				moving_speed_y = 0
			end
			moving_y = case.target_y - case.moving_height
			resolved = true
		end
	end

	return {
		moving_x = moving_x,
		moving_y = moving_y,
		moving_speed_x = moving_speed_x,
		moving_speed_y = moving_speed_y,
		target_speed_x = target_speed_x,
		target_speed_y = target_speed_y,
		resolved = resolved,
	}
end

print("# Generated from physics.lua horcollision/vercollision by tools/generate_collision_response_fixtures.lua.")
print("# case moving_x moving_y moving_width moving_height moving_speed_x moving_speed_y target_x target_y target_width target_height target_speed_x target_speed_y moving_handler target_handler expected_moving_x expected_moving_y expected_moving_speed_x expected_moving_speed_y expected_target_speed_x expected_target_speed_y expected_resolved")

for _, case in ipairs(cases) do
	local expected = step(case)
	print(string.format(
		"%s %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %.12g %s %s %s %s %.12g %.12g %.12g %.12g %s %s %s",
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
		case.moving_handler,
		case.target_handler,
		expected.moving_x,
		expected.moving_y,
		expected.moving_speed_x,
		expected.moving_speed_y,
		option(expected.target_speed_x),
		option(expected.target_speed_y),
		bool(expected.resolved)
	))
end

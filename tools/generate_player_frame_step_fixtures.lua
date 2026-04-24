-- Generates Rust-consumed fixtures for the mario.lua on-vine frame-step branch
-- without loading LOVE. The formulas mirror the deterministic subsection in
-- mario.lua that updates y, vinemovetimer, climbframe, and the optional portal
-- probe before any adapter-bound checkrect/checkportalHOR calls.

local vinemovespeed = 3.21
local vinemovedownspeed = vinemovespeed * 2
local vineframedelay = 0.15
local vineframedelaydown = vineframedelay / 2
local vineanimationstart = 4

local function climbframe(timer, delay)
	local frame = math.ceil(math.fmod(timer, delay * 2) / delay)
	return math.max(frame, 1)
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
		name = "vine_up_animation_threshold",
		y = 3.1,
		height = 0.75,
		move_timer = 0.10,
		direction = "up",
		block_y = nil,
		block_height = nil,
		dt = 0.05,
	},
	{
		name = "vine_down_open_probe",
		y = 5.0,
		height = 0.75,
		move_timer = 0.04,
		direction = "down",
		block_y = nil,
		block_height = nil,
		dt = 0.04,
	},
	{
		name = "vine_idle_resets_timer",
		y = 6.0,
		height = 0.75,
		move_timer = 1.2,
		direction = "idle",
		block_y = nil,
		block_height = nil,
		dt = 0.2,
	},
	{
		name = "vine_down_blocked_after_probe",
		y = 5.0,
		height = 0.75,
		move_timer = 0.02,
		direction = "down",
		block_y = 5.5,
		block_height = 1.0,
		dt = 0.04,
	},
}

local function step(case)
	local y
	local move_timer
	local frame
	local portal_probe_y = nil

	if case.direction == "up" then
		move_timer = case.move_timer + case.dt
		y = case.y - vinemovespeed * case.dt
		frame = climbframe(move_timer, vineframedelay)
	elseif case.direction == "down" then
		move_timer = case.move_timer + case.dt
		y = case.y + vinemovedownspeed * case.dt
		portal_probe_y = y
		frame = climbframe(move_timer, vineframedelaydown)
	else
		move_timer = 0
		y = case.y
		frame = 2
	end

	local blocked = false
	if case.block_y ~= nil then
		if case.direction == "up" then
			y = case.block_y + case.block_height
			frame = 2
			blocked = true
		elseif case.direction == "down" then
			y = case.block_y - case.height
			frame = 2
			blocked = true
		end
	end

	return {
		y = y,
		move_timer = move_timer,
		climb_frame = frame,
		trigger_animation = y + case.height <= vineanimationstart,
		portal_probe_y = portal_probe_y,
		blocked = blocked,
	}
end

print("# Generated from mario.lua on-vine movement by tools/generate_player_frame_step_fixtures.lua.")
print("# case y height move_timer direction block_y block_height dt expected_y expected_move_timer expected_climb_frame expected_trigger_animation expected_portal_probe_y expected_blocked_by_solid")

for _, case in ipairs(cases) do
	local expected = step(case)
	print(string.format(
		"%s %.12g %.12g %.12g %s %s %s %.12g %.12g %.12g %d %s %s %s",
		case.name,
		case.y,
		case.height,
		case.move_timer,
		case.direction,
		option(case.block_y),
		option(case.block_height),
		case.dt,
		expected.y,
		expected.move_timer,
		expected.climb_frame,
		bool(expected.trigger_animation),
		option(expected.portal_probe_y),
		bool(expected.blocked)
	))
end

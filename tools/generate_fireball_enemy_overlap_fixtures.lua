-- Generates Rust-consumed fixtures for the narrow fireball/enemy overlap
-- ordering branch from physics.lua passive/swept collision dispatch and
-- fireball.lua hitstuff outcomes. This avoids loading LOVE and keeps the
-- fixture focused on report-only runtime adapter parity.

local function option(value)
	if value == nil then
		return "none"
	end
	return tostring(value)
end

local function bool(value)
	if value then
		return "true"
	end
	return "false"
end

local function number_option(value)
	if value == nil then
		return "none"
	end
	return string.format("%.12g", value)
end

local function aabb(ax, ay, awidth, aheight, bx, by, bwidth, bheight)
	return ax+awidth > bx and ax < bx+bwidth and ay+aheight > by and ay < by+bheight
end

local firepoints = {
	goomba = 100,
	koopa = 200,
	plant = 200,
	bowser = nil,
	squid = 200,
	cheep = 200,
	flyingfish = 200,
	hammerbro = 1000,
	lakito = 200,
}

local function target_type(target)
	if target == "koopa_beetle" then
		return "koopa"
	end
	return target
end

local function collision_axis(projectile, enemy, dt)
	if aabb(projectile.x, projectile.y, projectile.width, projectile.height, enemy.x, enemy.y, enemy.width, enemy.height) then
		return "passive"
	end

	if not aabb(
		projectile.x + projectile.speed_x * dt,
		projectile.y + projectile.speed_y * dt,
		projectile.width,
		projectile.height,
		enemy.x,
		enemy.y,
		enemy.width,
		enemy.height
	) then
		return nil
	end

	if aabb(projectile.x + projectile.speed_x * dt, projectile.y, projectile.width, projectile.height, enemy.x, enemy.y, enemy.width, enemy.height) then
		if projectile.speed_x < 0 then
			return "left"
		end
		return "right"
	end

	if aabb(projectile.x, projectile.y + projectile.speed_y * dt, projectile.width, projectile.height, enemy.x, enemy.y, enemy.width, enemy.height) then
		if projectile.speed_y < 0 then
			return "ceiling"
		end
		return "floor"
	end

	if math.abs(projectile.speed_y - 80 * dt) < math.abs(projectile.speed_x) then
		if projectile.speed_y < 0 then
			return "ceiling"
		end
		return "floor"
	end

	if projectile.speed_x < 0 then
		return "left"
	end
	return "right"
end

local function hitstuff(projectile, target, axis)
	local after = {
		active = projectile.active,
		destroy_soon = false,
		speed_x = projectile.speed_x,
		speed_y = projectile.speed_y,
		released_thrower = false,
		shoot_target = nil,
		points = nil,
	}

	if axis == "left" then
		after.speed_x = 15
	elseif axis == "right" then
		after.speed_x = -15
	elseif axis == "floor" then
		after.speed_y = -10
	end

	local lua_target = target_type(target)
	if firepoints[lua_target] ~= nil or lua_target == "bowser" or target == "koopa_beetle" then
		if target ~= "koopa_beetle" then
			after.shoot_target = "right"
			after.points = firepoints[lua_target]
		end

		if projectile.active then
			after.released_thrower = true
			after.active = false
			after.destroy_soon = true
		end
	end

	return after
end

local cases = {
	{
		name = "passive_goomba_before_later_nonoverlap",
		projectile = { x = 3.375, y = 4.25, width = 0.5, height = 0.5, speed_x = 15, speed_y = 0, active = true },
		dt = 0.2,
		enemies = {
			{ target = "goomba", index = 7, x = 3.5, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = true },
			{ target = "koopa", index = 8, x = 6.0, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = true },
		},
	},
	{
		name = "skip_missing_handler_then_hit_plant",
		projectile = { x = 3.375, y = 4.25, width = 0.5, height = 0.5, speed_x = 15, speed_y = 0, active = true },
		dt = 0.2,
		enemies = {
			{ target = "goomba", index = 3, x = 3.5, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = false },
			{ target = "plant", index = 4, x = 3.5, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = true },
		},
	},
	{
		name = "horizontal_sweep_hits_koopa",
		projectile = { x = 3.0, y = 4.25, width = 0.5, height = 0.5, speed_x = 8, speed_y = 0, active = true },
		dt = 0.1,
		enemies = {
			{ target = "koopa", index = 5, x = 3.75, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = true },
		},
	},
	{
		name = "vertical_sweep_hits_bowser_without_points",
		projectile = { x = 3.0, y = 3.0, width = 0.5, height = 0.5, speed_x = 0, speed_y = 8, active = true },
		dt = 0.1,
		enemies = {
			{ target = "bowser", index = 1, x = 3.0, y = 3.75, width = 1.0, height = 1.0, has_shotted_handler = true },
		},
	},
	{
		name = "passive_beetle_explodes_without_shot",
		projectile = { x = 3.375, y = 4.25, width = 0.5, height = 0.5, speed_x = 15, speed_y = 0, active = true },
		dt = 0.2,
		enemies = {
			{ target = "koopa_beetle", index = 6, x = 3.5, y = 4.0, width = 1.0, height = 1.0, has_shotted_handler = true },
		},
	},
}

print("# Generated from physics.lua fireball/enemy overlap ordering and fireball.lua hitstuff by tools/generate_fireball_enemy_overlap_fixtures.lua.")
print("# case projectile_x projectile_y projectile_width projectile_height projectile_speed_x projectile_speed_y projectile_active dt enemy_order enemy_target enemy_index enemy_x enemy_y enemy_width enemy_height enemy_has_shotted_handler expected_probe_count expected_enemy_index expected_axis expected_target expected_shoot_target expected_points expected_released_thrower expected_after_active expected_after_destroy_soon expected_after_speed_x expected_after_speed_y")

for _, case in ipairs(cases) do
	local expected_enemy = nil
	local expected_axis = nil
	for _, enemy in ipairs(case.enemies) do
		if enemy.has_shotted_handler then
			local axis = collision_axis(case.projectile, enemy, case.dt)
			if axis ~= nil then
				expected_enemy = enemy
				expected_axis = axis
				break
			end
		end
	end

	local expected = nil
	if expected_enemy ~= nil then
		expected = hitstuff(case.projectile, expected_enemy.target, expected_axis)
	end

	for enemy_order, enemy in ipairs(case.enemies) do
		print(string.format(
			"%s %.12g %.12g %.12g %.12g %.12g %.12g %s %.12g %d %s %d %.12g %.12g %.12g %.12g %s %d %s %s %s %s %s %s %s %s %s %s",
			case.name,
			case.projectile.x,
			case.projectile.y,
			case.projectile.width,
			case.projectile.height,
			case.projectile.speed_x,
			case.projectile.speed_y,
			bool(case.projectile.active),
			case.dt,
			enemy_order,
			enemy.target,
			enemy.index,
			enemy.x,
			enemy.y,
			enemy.width,
			enemy.height,
			bool(enemy.has_shotted_handler),
			expected_enemy and 1 or 0,
			expected_enemy and expected_enemy.index or "none",
			expected_axis or "none",
			expected_enemy and expected_enemy.target or "none",
			expected and option(expected.shoot_target) or "none",
			expected and option(expected.points) or "none",
			expected and bool(expected.released_thrower) or "false",
			expected and bool(expected.active) or bool(case.projectile.active),
			expected and bool(expected.destroy_soon) or "false",
			number_option(expected and expected.speed_x or case.projectile.speed_x),
			number_option(expected and expected.speed_y or case.projectile.speed_y)
		))
	end
end

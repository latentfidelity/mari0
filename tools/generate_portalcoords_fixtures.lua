-- Generates Rust-consumed fixtures for physics.lua portalcoords without loading LOVE.

local facings = {"up", "right", "down", "left"}

local function portalcoords(x, y, speedx, speedy, width, height, rotation, animationdirection, entryportalX, entryportalY, entryportalfacing, exitportalX, exitportalY, exitportalfacing, live, gravity, gdt)
	x = x + width/2
	y = y + height/2

	local directrange
	local relativerange

	if entryportalfacing == "up" then
		directrange = entryportalY - y - 1
		if width == 2 then
			relativerange = 0
		else
			relativerange = ((x-width/2) - entryportalX + 1) / (2-width)
		end
	elseif entryportalfacing == "right" then
		directrange = x - entryportalX
		if height == 2 then
			relativerange = 0
		else
			relativerange = ((y-height/2) - entryportalY + 1) / (2-height)
		end
	elseif entryportalfacing == "down" then
		directrange = y - entryportalY
		if width == 2 then
			relativerange = 0
		else
			relativerange = ((x-width/2) - entryportalX + 2) / (2-width)
		end
	elseif entryportalfacing == "left" then
		directrange = entryportalX - x - 1
		if height == 2 then
			relativerange = 0
		else
			relativerange = ((y-height/2) - entryportalY + 2) / (2-height)
		end
	end

	local newx
	local newy

	if entryportalfacing == "up" and exitportalfacing == "up" then
		newx = x + (exitportalX - entryportalX)
		newy = exitportalY + directrange - 1
		speedy = -speedy
		rotation = rotation - math.pi

		if live then
			local minspeed = math.sqrt(2*gravity*height)
			if speedy > -minspeed then
				speedy = -minspeed
			end
		end
	elseif entryportalfacing == "down" and exitportalfacing == "down" then
		newx = x + (exitportalX - entryportalX)
		newy = exitportalY - directrange
		speedy = -speedy
		rotation = rotation - math.pi
	elseif entryportalfacing == "up" and exitportalfacing == "right" then
		newy = exitportalY - relativerange*(2-height) - height/2 + 1
		newx = exitportalX - directrange
		speedx, speedy = speedy, -speedx
		rotation = rotation - math.pi/2
	elseif entryportalfacing == "up" and exitportalfacing == "left" then
		newy = exitportalY + relativerange*(2-height) + height/2 - 2
		newx = exitportalX + directrange - 1
		speedx, speedy = -speedy, speedx
		rotation = rotation + math.pi/2
	elseif entryportalfacing == "up" and exitportalfacing == "down" then
		newx = x + (exitportalX - entryportalX) - 1
		newy = exitportalY - directrange

		if entryportalY > exitportalY then
			while newy+.5 + speedy*gdt > entryportalY do
				newy = newy - 0.01
			end

			while newy+.5 < exitportalY do
				newy = newy + 0.01
			end
		end

		if newx <= exitportalX - 2 + width/2 then
			newx = exitportalX - 2 + width/2
		elseif newx > exitportalX - width/2 then
			newx = exitportalX - width/2
		end
	elseif entryportalfacing == "down" and exitportalfacing == "up" then
		newx = x + (exitportalX - entryportalX) + 1
		newy = exitportalY + directrange - 1
	elseif entryportalfacing == "down" and exitportalfacing == "left" then
		newy = exitportalY - relativerange*(2-height) - height/2
		newx = exitportalX + directrange - 1
		speedx, speedy = speedy, -speedx
		rotation = rotation - math.pi/2
	elseif entryportalfacing == "down" and exitportalfacing == "right" then
		newy = exitportalY + relativerange*(2-height) + height/2 - 1
		newx = exitportalX - directrange
		speedx, speedy = -speedy, speedx
		rotation = rotation + math.pi/2
	elseif entryportalfacing == "left" and exitportalfacing == "right" then
		newx = exitportalX - directrange
		newy = y + (exitportalY - entryportalY)+1
	elseif entryportalfacing == "right" and exitportalfacing == "left" then
		newx = exitportalX + directrange - 1
		newy = y + (exitportalY - entryportalY)-1
	elseif entryportalfacing == "right" and exitportalfacing == "right" then
		newx = exitportalX - directrange
		newy = y + (exitportalY - entryportalY)
		speedx = -speedx
		if animationdirection == "left" then
			animationdirection = "right"
		elseif animationdirection == "right" then
			animationdirection = "left"
		end
	elseif entryportalfacing == "left" and exitportalfacing == "left" then
		newx = exitportalX + directrange - 1
		newy = y + (exitportalY - entryportalY)
		speedx = -speedx
		if animationdirection == "left" then
			animationdirection = "right"
		elseif animationdirection == "right" then
			animationdirection = "left"
		end
	elseif entryportalfacing == "left" and exitportalfacing == "up" then
		newx = exitportalX + relativerange*(2-width) + width/2 - 1
		newy = exitportalY + directrange - 1
		speedx, speedy = speedy, -speedx
		rotation = rotation - math.pi/2

		if live then
			local minspeed = math.sqrt(2*gravity*height)
			if speedy > -minspeed then
				speedy = -minspeed
			end
		end
	elseif entryportalfacing == "right" and exitportalfacing == "up" then
		newx = exitportalX - relativerange*(2-width) - width/2 + 1
		newy = exitportalY + directrange - 1
		speedx, speedy = -speedy, speedx
		rotation = rotation + math.pi/2

		if live then
			local minspeed = math.sqrt(2*gravity*height)
			if speedy > -minspeed then
				speedy = -minspeed
			end
		end
	elseif entryportalfacing == "left" and exitportalfacing == "down" then
		newx = exitportalX - relativerange*(2-width) - width/2
		newy = exitportalY - directrange
		speedx, speedy = -speedy, speedx
		rotation = rotation + math.pi/2
	elseif entryportalfacing == "right" and exitportalfacing == "down" then
		newx = exitportalX + relativerange*(2-width) + width/2 - 2
		newy = exitportalY - directrange
		speedx, speedy = speedy, -speedx
		rotation = rotation - math.pi/2
	end

	newx = newx - width/2
	newy = newy - height/2

	return newx, newy, speedx, speedy, rotation, animationdirection
end

local x = 9.25
local y = 7.5
local speedx = 3.0
local speedy = -4.0
local width = 1.0
local height = 1.0
local rotation = 0.25
local animationdirection = "left"
local entryX = 10.0
local entryY = 8.0
local exitX = 20.0
local exitY = 12.0
local live = false
local gravity = 80.0
local gdt = 1.0/60.0

print("# Generated from physics.lua portalcoords by tools/generate_portalcoords_fixtures.lua.")
print("# case entry exit x y speed_x speed_y width height rotation animation entry_x entry_y exit_x exit_y live gravity frame_dt expected_x expected_y expected_speed_x expected_speed_y expected_rotation expected_animation")

for _, entry in ipairs(facings) do
	for _, exit in ipairs(facings) do
		local newx, newy, newspeedx, newspeedy, newrotation, newanimationdirection =
			portalcoords(x, y, speedx, speedy, width, height, rotation, animationdirection, entryX, entryY, entry, exitX, exitY, exit, live, gravity, gdt)

		print(string.format(
			"%s_to_%s %s %s %.12g %.12g %.12g %.12g %.12g %.12g %.12g %s %.12g %.12g %.12g %.12g %s %.12g %.12g %.12g %.12g %.12g %.12g %.12g %s",
			entry,
			exit,
			entry,
			exit,
			x,
			y,
			speedx,
			speedy,
			width,
			height,
			rotation,
			animationdirection,
			entryX,
			entryY,
			exitX,
			exitY,
			tostring(live),
			gravity,
			gdt,
			newx,
			newy,
			newspeedx,
			newspeedy,
			newrotation,
			newanimationdirection
		))
	end
end

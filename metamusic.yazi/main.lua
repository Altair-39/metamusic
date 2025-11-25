local METAMUSIC = {}

local function setup(_, opts)
	opts = opts or {}

	return {
		on = {
			-- Register command via the Command event
			Command = function(args)
				if args.args == "metamusic" or args.args == "mp3edit" then
					-- Get current directory and run the editor
					local cwd = ya.manager().cwd
					ya.shell(Command({
						cmd = "metamusic",
						args = { tostring(cwd) },
					}))

					ya.manager_emit("update", {})
					return true
				end
			end,

			-- Add to context menu for MP3 files
			Menu = function(args)
				if args.type == "file" and args.file.name:match("%.mp3$") then
					return {
						{ name = "Edit MP3 Tags", command = "metamusic" },
					}
				end
			end,

			Key = function(args)
				if args.key == "e" and args.ctrl then
					local cwd = ya.manager().cwd
					ya.shell(Command({
						cmd = "metamusic",
						args = { tostring(cwd) },
					}))
					ya.manager_emit("update", {})
					return true
				end
			end,
		},
	}
end

METAMUSIC.setup = setup

return METAMUSIC

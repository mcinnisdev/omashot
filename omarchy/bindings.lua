-- Omashot keys, for ~/.config/hypr/bindings.lua.
--
-- Wayland has no global hotkey API, so the compositor holds these the same
-- way it holds every other key on the system. Each one runs `omashot <verb>`,
-- which reaches the running app over its socket -- and starts it first if it
-- is not up yet, so the first press after a login works like the rest.
--
-- `omarchy/install.sh` appends these for you; this file is what it appends,
-- and what to copy if you would rather do it by hand.
--
-- Every chord here is one Omarchy leaves free, checked against
-- `omarchy menu keybindings --print` on a stock install. Nothing is unbound
-- and nothing of yours is taken: SUPER+SHIFT+1..9 stay on "move window to
-- workspace", SUPER+SHIFT+RETURN stays on the browser, SUPER+ALT+1..5 stay
-- on the window groups, and SUPER+ALT+S stays on the scratchpad.
--
-- Every letter is the first letter of its verb, so the keys and the command
-- line are the same vocabulary: shot, add, trail, record, section, brief,
-- done, zoom.

o.bind("SUPER + ALT + Q", "Omashot: take a shot", "omashot shot")
o.bind("SUPER + ALT + A", "Omashot: add to the brief", "omashot add")
o.bind("SUPER + ALT + T", "Omashot: trail what I do", "omashot trail")
o.bind("SUPER + ALT + R", "Omashot: record the screen", "omashot record")
o.bind("SUPER + ALT + N", "Omashot: next section", "omashot section")
o.bind("SUPER + ALT + B", "Omashot: open the brief", "omashot brief")
o.bind("SUPER + ALT + D", "Omashot: brief is done", "omashot done")

-- Only does anything while a recording is running.
o.bind("SUPER + ALT + Z", "Omashot: mark a zoom", "omashot zoom")

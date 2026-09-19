-- Omacut keys, for ~/.config/hypr/bindings.lua.
--
-- Wayland has no global hotkey API, so the compositor holds these the same
-- way it holds every other key on the system. Each one runs `omacut <verb>`,
-- which reaches the running app over its socket -- and starts it first if it
-- is not up yet, so the first press after a login works like the rest.
--
-- `omarchy/install.sh` appends these for you; this file is what it appends,
-- and what to copy if you would rather do it by hand.
--
-- Every chord here is one Omarchy leaves free, checked against
-- `omarchy menu keybindings --print` on a stock install. Nothing is unbound
-- and nothing of yours is taken: SUPER+SHIFT+1..9 stay on "move window to
-- workspace", SUPER+SHIFT+RETURN stays on the browser, and SUPER+ALT+1..5
-- stay on the window groups. SUPER+ALT is where Omarchy keeps its second
-- rank of bindings, which is what these are.
--
-- The letters are the action, not the position: Quick, Capture, Auto,
-- New group, Bundle, End, Video, Zoom.

o.bind("SUPER + ALT + Q", "Omacut quick shot", "omacut quick")
o.bind("SUPER + ALT + C", "Omacut capture to bundle", "omacut capture")
o.bind("SUPER + ALT + A", "Omacut auto-capture", "omacut record")
o.bind("SUPER + ALT + N", "Omacut new group", "omacut group")
o.bind("SUPER + ALT + B", "Omacut bundle window", "omacut peek")
o.bind("SUPER + ALT + E", "Omacut finish bundle", "omacut finish")
o.bind("SUPER + ALT + V", "Omacut Studio recording", "omacut studio")

-- Only does anything while a Studio recording is running.
o.bind("SUPER + ALT + Z", "Omacut mark a zoom", "omacut zoom")

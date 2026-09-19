-- Omashot window rules, for ~/.config/hypr/looknfeel.lua.
--
-- Omarchy tags every window with `default-opacity` and renders it at
-- "0.985 0.96". That is the right default for a desktop and the wrong one
-- for Omashot: its windows show screenshots you are about to mark up and
-- hand to someone, and a review surface that lets the wallpaper through is
-- showing you something other than the pixels you captured.
--
-- Dropping the tag is the opt-out Omarchy documents, and the same one its
-- own bundled rules use for qemu, RetroArch and DaVinci Resolve.

o.window("^omashot$", { tag = "-default-opacity", opacity = "1 1" })

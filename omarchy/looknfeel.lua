-- Omashot window rules, for ~/.config/hypr/looknfeel.lua.
--
-- Two things Omarchy's defaults get wrong for this particular app.

-- Opacity. Omarchy tags every window with `default-opacity` and renders it
-- at "0.985 0.96". That is the right default for a desktop and the wrong one
-- for Omashot: its windows show screenshots you are about to mark up and
-- hand to someone, and a review surface that lets the wallpaper through is
-- showing you something other than the pixels you captured.
--
-- Dropping the tag is the opt-out Omarchy documents, and the same one its
-- own bundled rules use for qemu, RetroArch and DaVinci Resolve.
o.window("^omashot$", { tag = "-default-opacity", opacity = "1 1" })

-- Tiling. Omashot's markup and note windows ask for a size taken from the
-- image they are about to show -- that is the whole point of them -- and a
-- tiling layout overrules it, which is what makes a marked-up screenshot
-- look squashed. Floating them hands the size back to the app.
--
-- No `size` here on purpose: the app already knows how big each one should
-- be, and pinning a number would break the next image that is a different
-- shape.
--
-- The brief window is left tiled. It is the one you keep open and work
-- alongside, so it belongs in the layout like any other window.
local floating = "^Omashot (edit|review|shot|note|prompt library)$"
o.window({ class = "^omashot$", title = floating }, { float = true })
o.window({ class = "^omashot$", title = floating }, { center = true })

-- The recording badge is an overlay, not a window: it sits over the region
-- being recorded, ignores the mouse, and should follow you between
-- workspaces rather than belong to one.
o.window({ class = "^omashot$", title = "^Omashot recording$" }, {
  float = true,
  pin = true,
  no_initial_focus = true,
  border_size = 0,
  no_shadow = true,
})

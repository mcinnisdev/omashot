/// What to call the Super key in a hint.
///
/// Windows calls it Win and prints a Windows logo on it; everywhere else,
/// and especially in Hyprland's own config and `omarchy menu keybindings`,
/// it is SUPER. Printing "Win" on Omarchy would name a key that is not on
/// the keyboard and does not match anything the user can look up.
export const SUPER_LABEL = navigator.userAgent.includes("Windows")
  ? "Win"
  : "Super";

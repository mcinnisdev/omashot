// Omashot in the Omarchy bar.
//
// The app publishes what it is holding to a small JSON file in
// $XDG_RUNTIME_DIR and this watches it. That keeps the bar out of the app's
// process and means the widget is correct the moment the shell starts,
// whether or not Omashot happens to be running: no file, nothing to show.
//
// The widget only appears when there is something to say. A bar is the
// user's, and an app that sits in it permanently to announce that it is
// idle has not earned the space.

import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui

BarWidget {
  id: root
  moduleName: "omashot"

  property var state: null

  readonly property string brief: state && state.brief !== null ? state.brief : ""
  readonly property int shots: state && state.shots ? state.shots : 0
  readonly property int sections: state && state.sections ? state.sections : 0
  readonly property int quick: state && state.quick ? state.quick : 0
  readonly property bool trailing: state ? state.trailing === true : false
  readonly property bool recording: state ? state.recording === true : false

  readonly property bool holding: state !== null
    && (state.brief !== null || quick > 0 || trailing || recording)

  // A recording or a trail is something you need to see at a glance and be
  // able to stop; a brief sitting open is just a count.
  readonly property string glyph: recording ? "󰑋" : trailing ? "󰀘" : "󰹑"

  readonly property string countLabel: {
    if (recording) return "REC"
    if (trailing) return "TRAIL"
    if (quick > 0) return quick + ""
    if (shots > 0) return shots + ""
    return ""
  }

  readonly property string tip: {
    if (recording) return "Omashot is recording — click to stop"
    if (trailing) return "Omashot is trailing what you do — click to stop"
    if (quick > 0) return quick + (quick === 1 ? " loose shot" : " loose shots") + " waiting to be handed off"
    if (state && state.brief !== null) {
      var name = brief === "" ? "Unnamed brief" : brief
      var g = sections > 1 ? " across " + sections + " sections" : ""
      return name + " — " + shots + (shots === 1 ? " shot" : " shots") + g
    }
    return "Omashot"
  }

  visible: holding
  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  FileView {
    // Same name the app derives from $WAYLAND_DISPLAY, so two sessions on
    // one login do not read each other's state.
    path: (Quickshell.env("XDG_RUNTIME_DIR") || "/tmp") + "/omashot-"
      + (Quickshell.env("WAYLAND_DISPLAY") || "0") + ".state"
    watchChanges: true
    printErrors: false
    onFileChanged: reload()
    onLoaded: {
      try {
        root.state = JSON.parse(text())
      } catch (e) {
        root.state = null
      }
    }
    // No file means Omashot is not running, or has nothing open.
    onLoadFailed: root.state = null
  }

  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.glyph + (root.countLabel === "" ? "" : "  " + root.countLabel)
    active: root.recording || root.trailing
    tooltipText: root.tip
    onPressed: function (b) {
      if (root.recording) return root.bar.run("omashot record")
      if (root.trailing) return root.bar.run("omashot trail")
      // Middle click finishes and copies, which is the other thing you ever
      // want from a brief you can see the count of.
      if (b === Qt.MiddleButton) return root.bar.run("omashot done")
      root.bar.run("omashot brief")
    }
  }
}

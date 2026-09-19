import DefaultTheme from "vitepress/theme";
import { h } from "vue";
import MobileSidebar from "./MobileSidebar.vue";
import "./custom.css";

export default {
  extends: DefaultTheme,
  Layout() {
    return h(DefaultTheme.Layout, null, {
      "nav-screen-content-after": () => h(MobileSidebar),
    });
  },
};

import DefaultTheme from "vitepress/theme";
import { h } from "vue";
import MobileSidebar from "./MobileSidebar.vue";
import Footer from "./Footer.vue";
import "./custom.css";
import "./landing.css";
import "./usecases.css";
import "./omashot.css";

export default {
  extends: DefaultTheme,
  Layout() {
    return h(DefaultTheme.Layout, null, {
      "nav-screen-content-after": () => h(MobileSidebar),
      "layout-bottom": () => h(Footer),
    });
  },
};

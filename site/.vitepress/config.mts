import { defineConfig } from "vitepress";

// The docs are written with QACut itself: process docs exported from the
// studio land under docs/ as markdown with their images and clips beside
// them. VitePress turns the folder into the site; nothing else to do.
export default defineConfig({
  title: "QACut",
  description:
    "Capture, record and hand off. Bundles for agents, polished walkthroughs for people. Free and open source.",
  cleanUrls: true,
  lastUpdated: true,
  head: [["link", { rel: "icon", href: "/favicon.png" }]],
  themeConfig: {
    logo: "/logo.png",
    nav: [
      { text: "Docs", link: "/docs/getting-started" },
      { text: "Download", link: "https://github.com/mcinnisdev/qacut/releases/latest" },
      { text: "GitHub", link: "https://github.com/mcinnisdev/qacut" },
    ],
    sidebar: [
      {
        text: "Start here",
        items: [
          { text: "Getting started", link: "/docs/getting-started" },
          { text: "Keyboard shortcuts", link: "/docs/shortcuts" },
        ],
      },
      {
        text: "QACut",
        items: [
          { text: "Bundles for agents", link: "/docs/qacut" },
          { text: "Brand kit", link: "/docs/brand-kit" },
        ],
      },
      {
        text: "QACut Studio",
        items: [{ text: "Recordings for people", link: "/docs/studio" }],
      },
    ],
    socialLinks: [{ icon: "github", link: "https://github.com/mcinnisdev/qacut" }],
    footer: {
      message: "Free and open source under the MIT License.",
      copyright: "© 2026 Nick McInnis",
    },
    search: { provider: "local" },
  },
});

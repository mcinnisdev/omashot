import { defineConfig } from "vitepress";

// The docs are written with QACut itself: process docs exported from the
// studio land under docs/ as markdown with their images and clips beside
// them. VitePress turns the folder into the site; nothing else to do.
export default defineConfig({
  title: "QACut",
  description:
    "Screenshots an AI agent can act on. Screen recordings people will actually watch. Free and open source for Windows.",
  cleanUrls: true,
  lastUpdated: true,
  head: [
    ["link", { rel: "icon", href: "/favicon.png" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "QACut" }],
    ["meta", { property: "og:title", content: "QACut: screen capture that hands off" }],
    ["meta", { property: "og:description", content: "Screenshots an AI agent can act on. Screen recordings people will actually watch. Free and open source for Windows." }],
    ["meta", { property: "og:url", content: "https://qacut.com/" }],
    ["meta", { property: "og:image", content: "https://qacut.com/og.png" }],
    ["meta", { property: "og:image:width", content: "1200" }],
    ["meta", { property: "og:image:height", content: "630" }],
    ["meta", { property: "og:image:alt", content: "QACut: screen capture that hands off" }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:title", content: "QACut: screen capture that hands off" }],
    ["meta", { name: "twitter:description", content: "Screenshots an AI agent can act on. Screen recordings people will actually watch. Free and open source for Windows." }],
    ["meta", { name: "twitter:image", content: "https://qacut.com/og.png" }],
  ],
  themeConfig: {
    siteTitle: '<span class="qa">QA</span>Cut',
    logo: "/logo.png",
    nav: [
      {
        text: "Use cases",
        items: [
          { text: "QACut: quick shots for your agent", link: "/use-cases/quick-shots" },
          { text: "QACut Bundles: bigger jobs for your agent", link: "/use-cases/bundles" },
          { text: "QACut Studio: recordings for people", link: "/use-cases/studio" },
        ],
      },
      { text: "Docs", link: "/docs/getting-started" },
      { text: "Download", link: "https://github.com/mcinnisdev/qacut/releases/latest" },
      { text: "GitHub", link: "https://github.com/mcinnisdev/qacut" },
    ],
    sidebar: {
      "/use-cases/": [
        {
          text: "Use cases",
          items: [
            { text: "QACut", link: "/use-cases/quick-shots" },
            { text: "QACut Bundles", link: "/use-cases/bundles" },
            { text: "QACut Studio", link: "/use-cases/studio" },
          ],
        },
      ],
      "/docs/": [
        {
          text: "Start here",
          items: [
            { text: "Getting started", link: "/docs/getting-started" },
            { text: "Keyboard shortcuts", link: "/docs/shortcuts" },
          ],
        },
        {
          text: "QACut",
          items: [{ text: "Quick shots", link: "/docs/quick" }],
        },
        {
          text: "QACut Bundles",
          items: [
            { text: "Bundles for agents", link: "/docs/qacut" },
            { text: "Brand kit", link: "/docs/brand-kit" },
          ],
        },
        {
          text: "QACut Studio",
          items: [{ text: "Polished screen recordings", link: "/docs/studio" }],
        },
      ],
    },
    socialLinks: [{ icon: "github", link: "https://github.com/mcinnisdev/qacut" }],
    footer: {
      message: "Free and open source under the MIT License.",
      copyright: "© 2026 Nick McInnis",
    },
    search: { provider: "local" },
  },
});

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
      { text: "Docs", link: "/docs/getting-started" },
      {
        text: "Use cases",
        items: [
          {
            text: "QACut Basic",
            items: [
              { text: "Quick shot: copy and paste", link: "/use-cases/quick-shots#copy-and-paste" },
              { text: "Quick shot: mark up, copy, paste", link: "/use-cases/quick-shots#mark-up-copy-paste" },
              { text: "Quick shot: agent feedback loops", link: "/use-cases/quick-shots#agent-feedback-loops" },
            ],
          },
          {
            text: "QACut Bundles",
            items: [
              { text: "Organize, mark up, notate, export", link: "/use-cases/bundles#organize-mark-up-notate-export" },
              { text: "Automated process capture, edit, export", link: "/use-cases/bundles#automated-process-capture-edit-export" },
              { text: "Send to an agent with prompt and brand kit", link: "/use-cases/bundles#send-to-an-agent-with-prompt-and-brand-kit-for-polish" },
              { text: "A task list for agents", link: "/use-cases/bundles#organize-mark-up-notate-a-task-list-for-agents" },
            ],
          },
          {
            text: "QACut Studio",
            items: [
              { text: "Create polished screen recordings", link: "/use-cases/studio#create-polished-screen-recordings" },
              { text: "Include your microphone", link: "/use-cases/studio#include-your-microphone" },
              { text: "Include your camera", link: "/use-cases/studio#include-your-camera" },
              { text: "Include your brand", link: "/use-cases/studio#include-your-brand" },
            ],
          },
        ],
      },
      { text: "Changelog", link: "/changelog" },
      { text: "Feedback", link: "/feedback" },
      { text: "Download", link: "https://github.com/mcinnisdev/qacut/releases/latest" },
      { text: "GitHub", link: "https://github.com/mcinnisdev/qacut" },
    ],
    sidebar: {
      "/use-cases/": [
        {
          text: "Use cases",
          items: [
            { text: "QACut Basic", link: "/use-cases/quick-shots" },
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
            { text: "Prompt library", link: "/docs/prompts" },
          ],
        },
        {
          text: "QACut Basic",
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
    socialLinks: [
      { icon: "github", link: "https://github.com/mcinnisdev/qacut" },
      { icon: "x", link: "https://x.com/qa_cut" },
    ],
    // The footer is Footer.vue, mounted in theme/index.ts.
    search: { provider: "local" },
  },
});

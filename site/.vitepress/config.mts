import { defineConfig } from "vitepress";

const SITE_URL = "https://omashot.com";
const SITE_DESCRIPTION =
  "Shoot the screen, note it, hand it off. Screenshots an AI agent can act on, built for Omarchy. Free and open source.";

// The docs are written with Omashot itself: process docs exported from the
// studio land under docs/ as markdown with their images and clips beside
// them. VitePress turns the folder into the site; nothing else to do.
export default defineConfig({
  title: "Omashot",
  description: SITE_DESCRIPTION,
  cleanUrls: true,
  lastUpdated: true,
  sitemap: { hostname: SITE_URL },
  head: [
    ["link", { rel: "icon", href: "/favicon.png" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "Omashot" }],
    ["meta", { property: "og:title", content: "Omashot: screen capture that hands off" }],
    ["meta", { property: "og:description", content: SITE_DESCRIPTION }],
    ["meta", { property: "og:url", content: "https://omashot.com/" }],
    ["meta", { property: "og:image", content: "https://omashot.com/og.png" }],
    ["meta", { property: "og:image:width", content: "1200" }],
    ["meta", { property: "og:image:height", content: "630" }],
    ["meta", { property: "og:image:alt", content: "Omashot: screen capture that hands off" }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:title", content: "Omashot: screen capture that hands off" }],
    ["meta", { name: "twitter:description", content: SITE_DESCRIPTION }],
    ["meta", { name: "twitter:image", content: "https://omashot.com/og.png" }],
  ],
  // Per-page Open Graph and Twitter tags. The home page keeps the
  // site-wide values above; every other page describes itself. VitePress
  // drops a site head tag when the page sets the same one, so these win.
  transformPageData(pageData) {
    if (pageData.relativePath === "index.md") return;
    const path = pageData.relativePath.replace(/\.md$/, "").replace(/(^|\/)index$/, "$1");
    const url = `${SITE_URL}/${path}`;
    const title = pageData.frontmatter.title || pageData.title || "Omashot";
    const description = pageData.frontmatter.description || pageData.description || SITE_DESCRIPTION;
    pageData.frontmatter.head ??= [];
    pageData.frontmatter.head.push(
      ["meta", { property: "og:title", content: title }],
      ["meta", { property: "og:description", content: description }],
      ["meta", { property: "og:url", content: url }],
      ["meta", { name: "twitter:title", content: title }],
      ["meta", { name: "twitter:description", content: description }],
    );
  },
  themeConfig: {
    siteTitle: "Omashot",
    logo: "/logo.png",
    nav: [
      { text: "Docs", link: "/docs/getting-started" },
      {
        text: "Use cases",
        items: [
          {
            text: "For an agent",
            items: [
              { text: "A fix list", link: "/use-cases/fix-list" },
              { text: "A process, trailed", link: "/use-cases/trail-a-process" },
              { text: "One thing, right now", link: "/use-cases/one-shot" },
            ],
          },
          {
            text: "For a person",
            items: [
              { text: "Which button do you mean", link: "/use-cases/answer-a-question" },
              { text: "A document that writes itself", link: "/use-cases/write-the-doc" },
              { text: "A walkthrough", link: "/use-cases/a-walkthrough" },
            ],
          },
          { text: "All use cases", link: "/use-cases/" },
        ],
      },
      { text: "Changelog", link: "/changelog" },
      { text: "Feedback", link: "/feedback" },
      { text: "GitHub", link: "https://github.com/mcinnisdev/omashot" },
    ],
    sidebar: {
      "/use-cases/": [
        { text: "All use cases", link: "/use-cases/" },
        {
          text: "For an agent",
          items: [
            { text: "A fix list", link: "/use-cases/fix-list" },
            { text: "A process, trailed", link: "/use-cases/trail-a-process" },
            { text: "One thing, right now", link: "/use-cases/one-shot" },
          ],
        },
        {
          text: "For a person",
          items: [
            { text: "Which button do you mean", link: "/use-cases/answer-a-question" },
            { text: "A document that writes itself", link: "/use-cases/write-the-doc" },
            { text: "A walkthrough", link: "/use-cases/a-walkthrough" },
          ],
        },
      ],
      "/docs/": [
        {
          text: "Start here",
          items: [
            { text: "Getting started", link: "/docs/getting-started" },
            { text: "The keys", link: "/docs/keys" },
          ],
        },
        {
          text: "Capturing",
          items: [
            { text: "Shots", link: "/docs/shots" },
            { text: "Briefs", link: "/docs/briefs" },
            { text: "Recordings", link: "/docs/recordings" },
          ],
        },
        {
          text: "Handing off",
          items: [
            { text: "Prompts", link: "/docs/prompts" },
            { text: "Brand kit", link: "/docs/brand-kit" },
          ],
        },
      ],
    },
    socialLinks: [
      { icon: "github", link: "https://github.com/mcinnisdev/omashot" },
      { icon: "x", link: "https://x.com/qa_cut" },
    ],
    // The footer is Footer.vue, mounted in theme/index.ts.
    search: { provider: "local" },
  },
});

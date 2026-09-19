---
description: "Put a logo, colours and voice notes in ~/QACut/brand and QACut carries them into every bundle an AI agent writes from and every video the studio exports."
---

# Brand kit

Anything an agent writes from a bundle, and anything the studio renders, can carry your brand.

## The folder

Put whatever describes the business in `~/QACut/brand/`: a logo, colour swatches, fonts, a style guide, a document whose voice to imitate. Subfolders are fine. The bundle window's **Brand kit…** panel opens the folder and takes voice notes: tone, audience, terminology, things never to say. Notes are saved as `brand/brand.md`.

## In bundles

When a bundle is written, the folder is copied into it as `brand/`, so the bundle stays self-contained, and the markdown file gets a **Brand kit** section near the top with your notes inlined and the files listed. The built-in prompts tell the agent to match it. Untick **Include in this bundle** for a bundle where it does not apply.

<figure class="qc-shot">
<img src="/media/docs/brand-panel.png" alt="The Brand kit panel in the bundle window: the file count, the include toggle and the voice notes." loading="lazy" />
<figcaption>The Brand kit panel in the bundle window: the file count, the include toggle and the voice notes.</figcaption>
</figure>

## In the studio

The **Branding** section of the inspector places a title and subtitle in the padding above or below the frame, and a logo from the brand folder in a corner of it. Both render in the preview and the export.

## A good brand.md

Short and concrete beats long and vague. What worked for us:

- who you are and who reads what you write, in three sentences,
- voice rules as bullets: person, tone, sentence length,
- words to use and words to avoid,
- how a process document should be shaped,
- colours as a table with their roles, the typeface, and logo rules.

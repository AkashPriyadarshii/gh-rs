---
version: alpha
name: gh-rs Warm Premium — Gallery × Flagship, Light
description: A light, warm, premium launch page in the register of Stripe/Apple/Notion at their calmest. Limestone canvas, ink text, one clay voltage. B's gallery photography meets E's flagship density. Dark appears only inside terminal cards, like product shots on a gallery wall.
colors:
  canvas: "#FAF8F3"
  pearl: "#F1EDE3"
  card: "#FFFFFF"
  ink: "#1A1C1E"
  body: "#4E4A42"
  muted: "#8A8474"
  hairline: "#E4DED1"
  clay: "#B8422E"
  clay-deep: "#93331F"
  clay-tint: "#F7E4DC"
  term: "#141412"
  term-hair: "#2A2A26"
  term-text: "#ECE7D9"
  gold: "#C99700"
  success: "#0C6049"
typography:
  display:
    fontFamily: Space Grotesk
    fontSize: 72px
    fontWeight: 700
    lineHeight: 1.0
    letterSpacing: -0.035em
  h2:
    fontFamily: Space Grotesk
    fontSize: 38px
    fontWeight: 600
    lineHeight: 1.12
    letterSpacing: -0.02em
  body:
    fontFamily: system-ui
    fontSize: 17px
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: 0em
  label:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: 0.12em
  code:
    fontFamily: JetBrains Mono
    fontSize: 13.5px
    fontWeight: 400
    lineHeight: 1.8
rounded:
  sm: 8px
  md: 14px
  lg: 22px
spacing:
  section: 112px
---

## Overview

The merge of B (gallery) and E (flagship), re-grounded light. The page reads
as a warm gallery wall: limestone canvas {colors.canvas}, white cards
{colors.card} with hairline frames {colors.hairline}, dark terminal cards
{colors.term} hung like photographs. Clay {colors.clay} is the single voltage:
CTAs, active tabs, benchmark wins, focus rings. Nothing else carries color.

## Colors

- **Canvas** {colors.canvas} is warm paper, never pure white. **Pearl**
  {colors.pearl} bands alternate sections. Cards are white with 1px hairlines —
  elevation without shadows, gallery-style.
- **Clay** {colors.clay} labels actions only: primary buttons (white text),
  active tab pills, win cells, the brand tick. Deep {colors.clay-deep} is the
  press/hover state. Tint {colors.clay-tint} backs the announcement rail text.
- **Terminals** are the only dark on the page {colors.term}: warm-tinted black
  so they sit inside the palette, not as holes punched through it. Terminal
  prompt lines run clay, output runs {colors.term-text}, wins run gold
  {colors.gold}.
- **Success** {colors.success} marks benchmark wins on light (emerald holds
  7:1 on limestone; bright greens do not).

## Typography

Display Space Grotesk 700 at −0.035em, left-aligned, one headline per block.
Body system-ui 400 — the platform face, zero webfont risk for the long read.
Mono carries every label (caps, 0.12em), number, command, and table cell.
Tracking is size-specific: display tight, body zero, labels wide.

## Layout

Ten blocks: clay announcement rail, sticky translucent nav, split hero (copy
left, photographed terminal right), evidence strip (five hairline cells),
asymmetric feature grid (wide + narrow + two uneven, never three equal),
gh-vs-gh-rs comparison (white vs ink card), tabbed command terminal on pearl,
benchmark table, architecture + refusals, ink install band, footer. Max width
1160px. Section rhythm 112px. No ticker — premium pages do not scroll text at
you. No staggered entrances — the page is already calm.

## Motion

Almost nothing moves (gallery restraint). Terminal tabs crossfade 200ms
ease-out with a 2px blur mask. Buttons press to 0.97 in 120ms. Copy buttons
morph label only. Under `prefers-reduced-motion` even the crossfade goes
static. Hover states exist only behind `@media (hover: hover)` — touch sees
final states.

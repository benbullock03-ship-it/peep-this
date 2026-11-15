# AI hallmarks

The checklist Scrubadubdub (`/peep:scrubadubdub`, AV-87) reads. An AI hallmark is a pattern that marks a UI as made by default: nobody chose it. Exceptions live in `.peep/scrub.md` (`formats.md` §6).

Each entry: `ID · severity · how · signal · where it comes from · alternatives · [credit]`.

- **Severity** is who notices. `P0`: anyone. `P1`: a developer or designer. `P2`: craft only.
- **How.** `grep`: a regex in `ai-tells/rules.mjs` finds it in source, tested against a bad and a good snippet in `rules.test.mjs`; the signal here is a summary, the regex is the truth. `eye`: needs a render or a read of intent; report it as a judgement, not a match.
- **Credit** tags are listed under Sources.

## The earned test

A device is earned when it carries meaning a reader needs and would lose if it went. Looking like a hallmark elsewhere is not the test, and looking nice is not either: if it could be deleted and the page would say the same thing, it is not earned. Earned devices go in `.peep/scrub.md` with a written reason. [tropes]

A pass that only removes things is not finished: the page goes flat. Each removal names what replaces it. [tropes]

## Never flagged

- Anything the repo's `DESIGN.md` declares: its fonts, colours, radii, shadows, spacing scale and named components. A declared choice is the contract, even when it matches an entry here.
- Anything earned in `.peep/scrub.md`, and any line carrying `scrub-ignore: <ID> <reason>` (same line or the line above).
- Vendored and generated files (`vendor/`, `node_modules/`, build output) and the reference folders a repo names as source of truth (here `design/`).
- Loading indicators, skeleton shimmer included, under the Motion rules. [tropes]

## Colour

- **purple-gradient** · P0 · grep · a gradient through indigo, violet or purple (`#667eea → #764ba2`, `from-indigo-*`) · Tailwind UI's `indigo-500` default, copied into every starter · a flat brand colour with one accent that has a job; a tonal gradient inside one hue · [aad C1, slopcop, paralect 20, imp ai-color-palette]
- **violet-accent** · P1 · grep · indigo or violet `bg/text/border/ring-*-400..700` on actions · same root: the accent defaulted · the brand's own accent on the primary action; ink for links with an underline · [aad C2, slopcop]
- **gradient-text** · P0 · grep · `background-clip: text` or `bg-clip-text` · a 2024 hero flourish that doubles the purple gradient and hurts contrast · solid ink headings; emphasis by size or weight · [aad C6, imp gradient-text, slopcop]
- **colour-glow** · P2 · grep · `box-shadow: 0 0 ≥10px` in a non-black colour, `shadow-violet-500` · effect libraries' neon cards · shadow only for elevation, in neutral; atmosphere in the ground, not under each card · [aad C5, imp dark-glow, slopcop, paralect 21]
- **cream-terracotta** · P1 · grep · `#F4F1EA` ground or `#D97757` / `#C96442` accent · Claude's own interface palette, the first "tasteful" escape · a ground and accent taken from the subject's materials or signage; an accent from another hue family · [aad SD1, imp cream palette]
- **repeating-stripes** · P2 · grep · `repeating-linear-gradient(` or `repeating-radial-gradient(` · decorative stripes and grid lines filling empty space · a flat ground; a single rule where a division is real · [imp repeating-gradient stripes, paralect 21, tropes 19]
- **acid-on-black** · P1 · eye · near-black ground with one lime, green or vermilion accent, often mono · the dark twin of cream-terracotta: "technical" by default · a dark palette themed as a palette, not one neon; light, if nothing earns dark · [aad SD2]
- **emerald-fallback** · P2 · eye · emerald or teal as the only accent when nothing asked for green · the next reflex once purple was ruled out · tie the accent to the brand; give it one job (action or status) · [aad SD8]
- **halo** · P1 · eye · a radial glow or soft spotlight behind a hero or section · "the Linear effect" lifted onto unrelated products · atmosphere built into the ground; nothing behind content that does not explain it · [imp radial halo, imp soft spotlight, aad M3]
- **perma-dark** · P1 · eye · dark by default with grey body text and a purple accent · the stock "modern dark" starter · light or dark chosen for the use; AA body text on whichever ground · [aad C3, aad C7, slopcop]
- **low-contrast** · P1 · eye · muted or secondary text under 4.5:1 · barely-passing greys from starter tokens · raise muted text to AA; drop text nobody needs instead of greying it · [aad C9, paralect 16, imp low-contrast]
- **many-tints** · P2 · eye · several greys and per-category tinted fills at similar weight · colour spread evenly to avoid committing · ink, one accent and data colour; one dominant, one accent used sparingly · [tropes 7, aad C4, crit]

## Type

- **default-face** · P0 · grep · Inter or Geist as the first `font-family`, or Inter from Google Fonts · the default in nearly every AI tool and Next.js starter; a nit unless it is the only face · a face chosen for the brief, named by role on `:root`; a display face paired with the body face · [aad T1, aad T4, imp overused font, slopcop]
- **tasteful-face** · P1 · grep · Space Grotesk, Instrument Serif, Fraunces, Syne, Sora or Cal Sans · the "designed" free fonts that became generic · type from the domain's own artefacts; a foundry face the last three projects did not ship · [aad T2, slopcop, tropes 3]
- **tracked-caps** · P1 · grep · `text-transform: uppercase` with positive `letter-spacing` in one rule, `uppercase tracking-wide` · template chrome: micro-labels that read as "designed" · sentence case in the body face; drop the label when the heading says it · [aad SD4, aad T5, imp tracked-caps, tropes 18, crit]
- **neg-tracking** · P2 · grep · negative `letter-spacing`, `tracking-tight` · the "premium" display treatment copied to every heading · zero tracking; tighten only a display size the face was not drawn for · [tropes 4, imp crushed letter spacing]
- **accent-word** · P1 · grep · `<em>` or `<i>` inside an `h1`/`h2` · the one-word italic-serif or coloured garnish, a Claude signature · let the whole line carry the idea through size and line break; emphasise the thing that matters (a number, a name) · [aad SD5, aad T3, slopcop hero font mix, paralect 3, imp italic-serif-display]
- **eyebrow** · P1 · grep · a class named `eyebrow`, `kicker` or `overline` · a kicker above every heading · the heading alone; a real crumb in sentence case · [tropes 13, imp kicker-above-heading, paralect 4]
- **excessive-mono** · P2 · eye · mono for sentences, captions, labels, inputs or buttons · mono worn to look technical · mono for code, figures, ids and paths; the UI face for everything else · [tropes 17, paralect 6, aad SD4]
- **flat-scale** · P2 · eye · one face at one weight with headings barely above body · no hierarchy decision was made · a real type scale with weight steps; one figure at display size as the anchor · [aad T6, imp flat type hierarchy]
- **excess-italics** · P2 · eye · italic straps, outros and quotes · italics as decoration · upright text; italic for a rare single word · [tropes 2]
- **unshipped-face** · P1 · eye · a `font-family` no `@font-face` or font link loads · the design exists only in the stylesheet · ship the files locally; pick a face the platform already has · [aad T7]

## Layout

- **stat-strip** · P1 · grep · `10K+`, `99.9%`, `4.9★` · placeholder social proof on products with no users · one true figure at figure scale; cut the strip · [aad L4, slopcop stat banner, tropes 22, paralect 19]
- **numbered-markers** · P1 · grep · three or more bare `01`..`09` markers · numbering that imitates rigour on unordered things · a real label or none; numbers only for steps, rankings and citations · [aad SD6, tropes 12, imp tiny numbered labels, slopcop numbered steps]
- **default-shell** · P1 · grep · three or more `container mx-auto` / `max-w-7xl mx-auto` wrappers · the reflex Tailwind shell: one width forever · widths by section role; some full bleed, some a narrow measure · [aad L6]
- **most-popular** · P1 · grep · a "Most popular" badge · the three-tier pricing template with a ringed middle card · structure from the real offer (two plans, a table); a highlight earned by the plan, not a ring · [aad L7]
- **centred-hero** · P0 · eye · pill badge, centred H1, centred subhead, centred CTAs over three cards · the default landing skeleton · open on the subject's most characteristic thing (a live demo, a real screen); an asymmetric split · [aad L1, slopcop, tropes 23]
- **card-trio** · P0 · eye · three or six identical icon-topped cards · the most worn SaaS pattern · one showcase with smaller points; prose with inline emphasis · [aad L2, imp identical card grids]
- **hero-metric** · P1 · eye · a big number, a small label, supporting stats and a gradient accent · the stock dashboard hero · the one figure the reader came for; a figure in context (period, source) · [imp hero metric layout, aad L4]
- **bento-default** · P2 · eye · a mixed-size tile grid the content does not need · a saturated 2025 trend · tile sizes that encode importance; the layout the content's hierarchy asks for · [aad L3]
- **section-waterfall** · P1 · eye · hero, logos, features, how it works, testimonials, stats, pricing, FAQ, CTA · the template's order, whatever the product · order by what the reader needs to decide; one structural idea · [aad L9]
- **nested-boxes** · P2 · eye · a bordered box inside a bordered box, or a border around every block · containers standing in for hierarchy · one level of containment; space, rules and type scale for the rest · [aad K11, imp nested cards, tropes 8, paralect 2]
- **rule-overuse** · P2 · eye · a rule above and below every block, rules inside cards · dividers as decoration · a rule only between unlike things; spacing for grouping · [tropes 6]
- **monotonous-spacing** · P2 · eye · the same gap and padding everywhere · even spacing reads as a flat hierarchy · more space above a heading than below; crowd related things, isolate others · [aad S1, imp monotonous spacing]
- **off-scale-spacing** · P2 · eye · spacing values off the repo's scale (13px, 17px, 0.85rem beside a 4px scale) · values nudged by eye, one at a time · the nearest scale step; a new token if the scale truly lacks one · [crit]
- **overstuffed** · P1 · eye · banners, tips, cards and buttons competing with the main task · everything the template had, kept · remove what nobody acts on; one primary action per view · [paralect 11, tropes 5]

## Components

- **shadcn-radius** · P1 · grep · `--radius: 0.5rem` or `0.625rem` · shadcn's default theme, never touched · a radius chosen for the direction; radius by element role · [aad K8, aad K1]
- **card-kit** · P1 · grep · `rounded-xl/2xl/3xl` with `shadow-md/lg/xl` on one element · the SaaS card kit: one radius, one soft shadow, everywhere · flat surfaces with rules; strong shadow only for what floats · [aad K2, paralect 15]
- **glass** · P1 · grep · `backdrop-filter: blur`, `backdrop-blur` · frosted panels by reflex · solid surfaces; blur only where layering is real (a menu over content) · [aad K3, slopcop, paralect 14, imp glassmorphism]
- **side-stripe** · P1 · grep · `border-left` 2 to 9px solid, `border-l-4` · "almost as reliable a sign as em-dashes" · weight, position or a heading for emphasis; a stripe only for real status · [aad K4, slopcop accent stripe, imp side-tab, tropes 25, paralect 1]
- **headline-pill** · P2 · grep · a `badge`/`pill`/`chip` element directly before the `h1` · "✨ New: v2 is here" · drop it; real dated news as a line of text near the date · [aad K5, slopcop headline badge, imp hero-eyebrow-chip]
- **pill-radius** · P2 · grep · `border-radius: 999px`, `rounded-full` · pills as labels and filters · status as a word, category as coloured text, filter as a native select · [tropes 1, paralect 5, aad K12]
- **effect-library** · P1 · grep · Aceternity or Magic UI components (Spotlight, BorderBeam, NumberTicker, Marquee) · copy-paste effects on every launch · an effect built from the product's own data or interface; cut it · [aad K10, imp auto-scrolling marquee]
- **emoji-icons** · P1 · grep · two or more emoji in markup · emoji standing in for an icon set · one consistent icon set; a plain list with no icons · [aad I2, slopcop emoji nav, paralect 9]
- **worn-icons** · P1 · grep · `Sparkles`, `Zap` or `Rocket` from `lucide-react` · the same glyphs in the same roles; Sparkles means "AI" · icons picked for meaning; no icon where the word is enough · [aad I3]
- **fake-window** · P2 · grep · macOS traffic-light colours (`#ff5f57`, `#febc2e`, `#28c840`) · the stock fake-terminal hero when there is no product shot · a real screenshot of the product; a mock of the product's own interface · [aad SD7]
- **default-dark-card** · P1 · grep · `bg-zinc-950` (or gray/slate/neutral 900–950) with `border-white/10` · the unmodified "modern dark" card · a near-black chosen for the product; surfaces separated by value, not a 10% border · [aad K9]
- **hard-shadow** · P2 · grep · an unblurred `box-shadow` offset 4px or more · neo-brutalist reflex · no shadow on resting surfaces; a soft neutral shadow only for what floats · [imp hard offset shadow]
- **icon-tile** · P1 · eye · a small icon on a tinted rounded square above each heading · Lucide plus shadcn's stock feature-icon treatment · remove the tile and usually the icon; icons only where they are the control · [aad K6, imp icon-tile-stack, tropes 16, tropes 21]
- **missing-states** · P1 · eye · no hover, `focus-visible`, active, disabled, empty or error states · the happy path only · every control's states designed; a visible focus ring distinct from the accent · [aad K7]
- **chip-soup** · P2 · eye · "New", "Beta", "Live" chips and dots with no state behind them · chips to look app-like · a chip only for real changing state, one visual language · [aad K12, paralect 5]
- **ghost-card** · P2 · eye · a hairline border and a wide soft shadow defining the same card · border and shadow stacked out of indecision · one of the two; neither, with a ground shift · [imp hairline border with wide shadow, aad K2]
- **fake-proof** · P1 · eye · logo walls of non-customers, anonymous testimonials, generated faces · trust chrome with nothing behind it · real names, roles and links; leave the section out · [paralect 19, aad L4]

## Motion

- **fade-up** · P2 · grep · `initial={{ opacity: 0, y }}`, `whileInView`, `data-aos="fade-up"`, a `fadeUp` keyframe · a reflexive reveal on every section · content shown at once; one entrance for one key element · [aad M1, tropes motion ban 3, paralect 18]
- **count-up** · P2 · grep · `CountUp`, `NumberTicker`, `tweened(` · theatre around hollow numbers · show the number; make it true · [aad M5, tropes motion ban 4]
- **bounce-easing** · P2 · grep · an overshooting `cubic-bezier`, `animate-bounce`, `easeOutBack` · springy UI motion from effect demos · ease-out with no overshoot; stepped motion where the repo uses it · [aad M4, imp bounce-easing]
- **ambient-loop** · P2 · grep · `animation: … infinite`, `animate-pulse`, `animate-ping` · pulsing dots and drifting blobs with nothing to say · animate only what changes; a finite pulse on entering an attention state · [tropes motion ban 2, imp pulsing status dot, paralect 17]
- **hover-lift** · P2 · grep · `:hover` with `translateY(-…)`, `hover:-translate-y-*` · cards that jump on hover · a colour or underline change; a border or ground shift · [tropes motion ban 1]
- **no-reduced-motion** · P1 · eye · animation with no `prefers-reduced-motion` handling · the quality floor generated pages skip · the finished state under `reduce`; motion wrapped in `no-preference` · [aad M6]
- **scattered-motion** · P2 · eye · a different hover or bounce on every element, no shared timing · motion sprinkled on · one duration scale and easing; motion that answers an action · [aad M2]
- **blinking-cursor** · P2 · eye · a blinking cursor on static text · the fake-terminal hero · static text; a real input if it is one · [imp decorative blinking cursor, aad SD7]

## Copy

- **aspirational** · P1 · grep · "Elevate your", "Unleash", "all-in-one platform", "the future of" · brand-agnostic filler that fronts any product · what it does, for whom, in plain words; a verb-led line naming the outcome · [aad CP1, imp generic marketing claims, paralect 8]
- **buzzwords** · P2 · grep · seamless, cutting-edge, best-in-class, effortless, next-gen, world-class · the microcopy of beige · a concrete claim or a number; a verb · [aad CP2]
- **tricolon** · P2 · grep · three one-word comparatives: "Faster, smarter, simpler." · the rule of three by reflex · one concrete claim; a single true comparison · [aad CP4]
- **arrow-links** · P1 · grep · two or more `→` glued to link or button text · template chrome · one arrow per view on the onward link; `↗` for external · [aad CP3, tropes 24]
- **generic-cta** · P2 · grep · "Get started", "Learn more", "Click here", "Submit" · labels that do not say what happens · name the action and its object ("Install the hooks"); keep the verb through the flow · [aad CP6]
- **em-dash** · P2 · grep · three or more em dashes in one file's UI copy · the generated-prose tic · a full stop; a comma or colon · [imp em-dash overuse]
- **full-stop-heading** · P2 · grep · an `h1`..`h3` ending in a full stop · the slogan heading · no full stop; a heading that names what is shown · [tropes 20]
- **placeholder-copy** · P0 · grep · lorem ipsum, John Doe, Acme Inc · generated stand-ins left in · the real string; an empty state that says what to do · [paralect 10, aad IM4]
- **forced-contrast** · P2 · eye · "Not X, but Y" slogans · contrast as a sentence template · the direct statement; the Y alone · [imp forced contrast]
- **said-twice** · P2 · eye · a subtitle restating the title; a fact shown twice · padding to fill a layout slot · say it once; delete the weaker one · [tropes 10, imp same text repeated]
- **title-case** · P2 · eye · every heading, button and label in Title Case · uniform casing habit · sentence case for interface text; Title Case only where a house style asks · [aad CP5]
- **code-crumbs** · P2 · eye · `AREA · KIND · ID` breadcrumbs and `A · B · C` meta strings · template chrome · a real crumb in sentence case; the one fact that matters · [tropes 11, aad SD4]
- **numbers-as-words** · P2 · eye · "three steps", "five sessions" in headings and labels · copy padded to sound written · digits · [paralect 7]

## Code

- **colour-literal** · P2 · grep · a hex or `rgb(` literal outside a `--token:` definition line · values pasted per component; the root of drift · a role token from `:root`; a new token if none fits · [tropes, aad X1, crit]
- **z-index-max** · P2 · grep · `z-index` of four digits or more · escalation instead of a stacking order · a small named z scale; fix the stacking context · [own]
- **important-pile** · P2 · grep · three or more `!important` in one file · specificity fights won by force · lower the competing selector; a layer (`@layer`) order · [own]
- **generator-tag** · P1 · grep · `lovable-tagger`, `gpt-engineer`, v0 or Bolt generator meta · the build tool's signature left in · remove the tagger and meta; keep attribution only where a licence requires it · [aad F1]
- **placeholder-media** · P1 · grep · pravatar, DiceBear, placehold.co, picsum and similar · stand-in media services · a real screenshot or avatar; an intentional empty frame · [aad IM4, paralect 10]
- **arbitrary-px** · P2 · grep · three or more Tailwind arbitrary px spacing values (`p-[13px]`) · spacing nudged by eye · the scale's steps; a theme token for a value used twice · [own]
- **drift** · P0 · eye · one product, two systems: fonts, palette, radius or chrome change between pages · each page regenerated from the median instead of the repo's tokens · one token contract reused verbatim; shared chrome as one component · [aad X1, crit]
- **hand-built-twice** · P2 · eye · the same panel or control hand-built in several places · each prompt rebuilds it · one component; delete the copies · [crit]

## What not to over-flag

- **Never claim a model made the page.** A hallmark is a default nobody chose. Designers shipped centred heroes and rounded cards long before models did; the fix is the same whoever defaulted. [aad]
- **One hit is a nit.** Judge combinations: one accented word on an authored page is a nit; cream-terracotta plus tracked-caps plus accent-word plus numbered-markers is a template. Report single hits under P2 at most unless the entry is P0. [aad]
- **The brief and the repo win.** DESIGN.md, earned entries and the owner's rulings outrank this list. A repo whose reference design (here `design/shell/`, AV-44) uses a device has chosen it.
- **Earned beats familiar.** Run the earned test before flagging; looking like a hallmark elsewhere is not enough. [tropes]
- **Over-blamed:** glass, bento grids and mesh backgrounds rank low as tells people actually notice; flag them only as reflexive defaults. [aad]
- **Spacing is soft.** Do not lead a report with monotonous-spacing. [aad]
- **Restraint is a decision.** A minimal design done with intent is not "timid".
- **A false positive is a bug** in `rules.mjs`: fix the regex and add the snippet that tripped it to `rules.test.mjs` as a good case.

## Sources

Borrowed entries carry these tags. Entries are paraphrased; no text is copied beyond rule names.

- **aad**: avoid-ai-design, `references/ai-tells-catalog.md` and `scripts/detect.mjs`, github.com/funboy322/avoid-ai-design. MIT, Copyright (c) 2026 ungspirit. Severity scale and the over-flag rules come from it; several regexes in `rules.mjs` are adapted from `detect.mjs`.
- **imp**: Impeccable, impeccable.style/slop and github.com/pbakaus/impeccable (`crates/core/src/checks/rules.rs`), by Paul Bakaus. Apache-2.0. Rule names and ideas only; no code taken.
- **slopcop**: design-slop-cop, github.com/AdrianKrebs/design-slop-cop, by Adrian Krebs. MIT, Copyright (c) 2026 Adrian Krebs. Its 14 patterns, from a study of about 1,400 Show HN sites.
- **paralect**: "Signs of AI slop in design", paralect.com/blog/signs-of-ai-slop-in-design. Numbers are the article's sign order. Ideas paraphrased with credit; no licence stated.
- **tropes**: the owner's design-tropes skill (28 tropes, the earned test, the motion bans). Numbers are its trope order.
- **crit**: the owner's critique reports (off-system palettes, tracked caps left behind, panels hand-built many times, drift between routes).
- **own**: added for this pack; the "messy spacing and text" half of Scrubadubdub.

Licences for aad, slopcop and imp were read from each repo's `LICENSE` via the GitHub API on 2026-11-15.

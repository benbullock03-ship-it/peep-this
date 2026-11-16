# Pass 2: eye

Entries marked `eye` in the catalogue need a render or a read of intent. Half of the scrub pass is messy spacing and text, so every `--scrub` run covers these entries even when no grep rule fired:

- Spacing: monotonous-spacing, off-scale-spacing (measured against the house spacing scale), nested-boxes, rule-overuse, overstuffed.
- Text: title-case, said-twice, numbers-as-words, forced-contrast, code-crumbs, low-contrast, flat-scale.
- Then any other eye entry the page could plausibly show. Skip entries that cannot apply, such as pricing entries on a settings panel.

## With a `url`

Judge the shots this run already took (same pages, themes and widths); take more only for a page in scope that has none. The pass/fail questions are the eye entries above, phrased against the house style:

- "muted text on `--panel` reaches 4.5:1"
- "every gap is a step of the 4px scale"
- "no subtitle repeats its heading"

A failed question becomes a finding at the file and line that styles the element (find it by selector).

## Without a `url`

(Shoot stops at step 1 without a `url`, so `--scrub` never reaches here from Shoot; kept for a scan run by hand.) Read the markup and styles instead. Mark every eye finding `inferred`. Contrast is computed from the token values. Spacing is read from the declared values.

## Rules

- Run the earned test (catalogue) before each eye finding. A device that carries meaning the reader needs is not a finding: offer an earned line instead.
- Eye findings are scored like grep findings: the catalogue severity weight (P0 3, P1 2, P2 1), plus one for each other ID in the same file. A single non-P0 hit is reported as P2.
- Never lead the report with monotonous-spacing (catalogue, "Spacing is soft").

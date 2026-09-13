# Agents

A Rust-based project for generating SVG visuals from text-based markup, compiled to WebAssembly.

## Project Structure

- `src/` — Rust source (core markup-to-SVG logic)
<!-- placeholder: confirm/add other top-level dirs, e.g. `wasm/`, `examples/` -->
- `visuals/` — HTML files that accompany each Rust visual type (see "Visuals" below)
- `docs/` — GitHub Pages documentation site (`docs/index.html`)

## Build & Test

<!-- placeholder: add actual commands, e.g. -->
- Build for WASM: `wasm-pack build --target web`
- Run tests: `cargo test`
- Format/lint: `cargo fmt && cargo clippy`

## SVG Rendering Requirements

- All generated SVGs must support both light and dark mode.
- Use CSS variables and `prefers-color-scheme` media queries so the SVG automatically switches based on system settings — do not hardcode colors that would break dark mode.
- When designing or updating the visual style of an SVG, apply the **`premium` design skill**:
    - Skill definition: `<!-- placeholder: path, e.g. .claude/skills/premium/SKILL.md -->`
    - Design tokens (colors, typography, spacing): `<!-- placeholder: path, e.g. DESIGN.md -->`
    - Follow the token values and rules defined in these files (e.g. primary `#3B82F6`, Inter/JetBrains Mono typography, 4/8/12/16/24/32 spacing scale) rather than inventing new values.
    - Prioritize accessibility (WCAG 2.2 AA, visible focus states, sufficient contrast) over novel styling choices, per the skill's rules.

## WebAssembly Target

- The library compiles to WebAssembly for use in the browser/docs site.
- Any new dependency must be WASM-compatible before it's added.
<!-- placeholder: note any known-incompatible crates to avoid -->

## Visuals Directory

- Each new type of visual (or update to an existing one) must have a corresponding HTML file in `visuals/` that demonstrates/exercises the Rust type.
<!-- placeholder: naming convention, e.g. `visuals/<visual_type>.html` -->
- When adding or changing a visual type, update or create its HTML file in the same change.

## Documentation

- The project's documentation site is published via GitHub Pages, maintained locally at `docs/index.html`.
- Whenever a new visual type is added, or an existing one changes (API, output, or behavior), update `docs/index.html` to reflect the change in the same PR/commit — don't let docs drift from the implementation.
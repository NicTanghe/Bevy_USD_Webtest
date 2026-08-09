# OpenUSD Web Viewer

A Leptos shell around the Bevy/OpenUSD application in the sibling
`bevy_openusd` repository. The browser viewport is hosted by
[`leptos-bevy-canvas`](https://github.com/Synphonyte/leptos-bevy-canvas), and
the scene is projected from a live in-memory OpenUSD stage through `usd_bevy`.

## Run locally

```powershell
rustup target add wasm32-unknown-unknown
cargo install --locked cargo-leptos
cargo leptos watch --release
```

Open <http://127.0.0.1:3000>. Axum serves the server-rendered Leptos page and
the hydrated browser bundle starts the Bevy/OpenUSD canvas.

## Styling

`styles/main.scss` is the stylesheet entry point. Component styles are split
into partials in `styles/`, while shared colors, layout dimensions, breakpoints,
transitions, and mixins live in `styles/_tokens.scss`. `cargo leptos` compiles
the SCSS as part of the normal build and watch workflows.

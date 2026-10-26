# NowPlaying implementation update

## Changes included
- Added the Cassette skin to the control page skin selector.
- Added a recommended Cassette Browser Source size of 916 × 505 px.
- Removed the unused live skin preview iframe, preview-only JavaScript/CSS, preview image handlers/routes, and preview images.
- Added HTTP routes for the bundled WOFF2 fonts referenced by the control page.
- Kept the existing shared playback polling/interpolation architecture and Cassette skin's 12-segment progress behavior.
- Preserved the existing Default, Ambient, and Glass skins.

## Validation
- `node --check` passed for all JavaScript files.
- CSS brace-balance checks passed for the control page and all skin stylesheets.
- Verified all `include_str!` / `include_bytes!` file references in `src/server.rs` and `build.rs` resolve.
- Rust compilation was not run in this environment because Cargo is unavailable. Build and test on Windows with `cargo build`.

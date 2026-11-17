# NSH GPUI backend, application patch

Source: `longbridge/nsh`, revision `224d9a795c7ac9d10d75750de9351adac5549675`, directory `crates/gpui/nsh-platform-gpui`. Apache-2.0; see LICENSE-APACHE.

Local change in `src/text_input.rs`, `paint_glyphs`: reserve storage for visible text and skip offscreen line bands before constructing/submitting glyphs. Axis-aligned text keeps a conservative font-metric margin; rotated/perspective text follows the original path. Layout, input, scrolling, IME, caret and selection behavior are unchanged.

Cargo.toml expands the upstream workspace dependencies so this directory works as a standalone patched dependency; the other NSH crates still use the original fixed git revision. The Pacifico test fixture and its OFL license are local to this directory; only its test include path was adjusted in src/text.rs.

Keep this patch small. Compare foreground frame statistics and long Chinese text screenshots before removing or updating it. The application package is the root workspace member; this dependency's upstream test suite is separate.

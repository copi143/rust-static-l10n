# rust-static-l10n

Static localization for Rust using procedural macros.

`static-l10n` loads translations at compile time and generates match-based
lookups, so missing translations fail the build rather than at runtime.

## Workspace Layout

- `derive/`: proc-macro crate that provides all macros.
- `test/`: example crate used for manual/testing.

## Quick Start

Add metadata in your crate's `Cargo.toml`:

```toml
[package.metadata.static-l10n]
path = "i18n"
base = "en"
langs = ["en", { name = "zh", fallback = "en" }]
```

Put translations under the `path` directory (TOML):

```toml
["Hello, world!"]
en = "Hello, world!"
zh = "你好，世界！"

["Hello, {}!"]
en = "Hello, {}!"
zh = "你好，{}！"
```

Then use the macros in your crate:

```rust
static_l10n::main!();

static_l10n::lang!("zh");
let msg = static_l10n::l10n!("Hello, world!");
let formatted = static_l10n::f16n!("Hello, {}!", "Rust");
```

## Rebuild on Translation Changes

`static_l10n::main!()` expands `include_str!` for every translation file, so
editing an existing file triggers a rebuild automatically. New files are not
picked up unless you rebuild or add a `build.rs` that watches the directory.

## Docs

- Macro reference and detailed usage: `derive/README.md`

## Development

Run tests in the example crate:

```bash
cargo test -p static-l10n-test
```

## License

See `LICENSE`.

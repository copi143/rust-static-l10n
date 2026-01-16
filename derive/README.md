# static-l10n

Procedural macros for `static-l10n`.

## Macros

- `main!()` initializes the language state with the default language.
- `lang!("xx")` switches the current language.
- `l10n!("key")` returns a localized string for `key`.
- `f16n!("fmt: {}", arg)` returns a localized formatted string.
- `l10n_print!("key")` prints a localized string.
- `l10n_println!("key")` prints a localized string with newline.
- `l10n_eprint!("key")` prints a localized string to stderr.
- `l10n_eprintln!("key")` prints a localized string to stderr with newline.
- `l10n_write!(writer, "key")` writes a localized string to a writer.
- `l10n_writeln!(writer, "key")` writes a localized string to a writer with newline.
- `l10n_panic!("key")` panics with a localized string.
- `l10n_args!("key")` returns `std::fmt::Arguments` for formatting APIs.
- `l10n_assert!(cond, "key")` asserts a condition with a localized message.
- `l10n_assert_eq!(left, right, "key")` asserts equality with a localized message.
- `l10n_assert_ne!(left, right, "key")` asserts inequality with a localized message.
- `l10n_debug_assert!(cond, "key")` debug-asserts a condition with a localized message.
- `l10n_debug_assert_eq!(left, right, "key")` debug-asserts equality with a localized message.
- `l10n_debug_assert_ne!(left, right, "key")` debug-asserts inequality with a localized message.
- `f16n_print!("fmt: {}", arg)` prints a localized formatted string.
- `f16n_println!("fmt: {}", arg)` prints a localized formatted string with newline.
- `f16n_eprint!("fmt: {}", arg)` prints a localized formatted string to stderr.
- `f16n_eprintln!("fmt: {}", arg)` prints a localized formatted string to stderr with newline.
- `f16n_write!(writer, "fmt: {}", arg)` writes a localized formatted string to a writer.
- `f16n_writeln!(writer, "fmt: {}", arg)` writes a localized formatted string to a writer with newline.
- `f16n_panic!("fmt: {}", arg)` panics with a localized formatted string.
- `f16n_args!("fmt: {}", arg)` returns `std::fmt::Arguments` for formatting APIs.
- `f16n_assert!(cond, "fmt: {}", arg)` asserts a condition with a localized formatted message.
- `f16n_assert_eq!(left, right, "fmt: {}", arg)` asserts equality with a localized formatted message.
- `f16n_assert_ne!(left, right, "fmt: {}", arg)` asserts inequality with a localized formatted message.
- `f16n_debug_assert!(cond, "fmt: {}", arg)` debug-asserts a condition with a localized formatted message.
- `f16n_debug_assert_eq!(left, right, "fmt: {}", arg)` debug-asserts equality with a localized formatted message.
- `f16n_debug_assert_ne!(left, right, "fmt: {}", arg)` debug-asserts inequality with a localized formatted message.
- `debug_print_metadata!()` prints resolved metadata and loaded translations at compile time.

## Usage

Add metadata to your crate `Cargo.toml` (this is read at compile time):

```toml
[package.metadata.static-l10n]
path = "i18n"
base = "en"
langs = ["en", { name = "zh", fallback = "en" }]
```

Create translation files under the `path` directory. Each file is TOML and can contain multiple keys:

```toml
["Hello, world!"]
en = "Hello, world!"
zh = "你好，世界！"

["Hello, {}!"]
en = "Hello, {}!"
zh = "你好，{}！"
```

You can split keys across multiple files and nested folders under `path`. Duplicate keys for the same language will cause a compile-time panic.

Then use the macros in your crate:

```rust
static_l10n::main!();

static_l10n::lang!("zh");
let msg = static_l10n::l10n!("hello");
let formatted = static_l10n::f16n!("count: {}", 3);
static_l10n::l10n_println!("hello");
static_l10n::f16n_println!("count: {}", 3);
```

## How It Works

- `main!()` defines a global mutex holding the current language. Call it once at crate root.
- `lang!("xx")` switches the current language for subsequent lookups.
- `l10n!` looks up a string key and returns a `&'static str` literal from the translations table.
- `f16n!` looks up a format string and returns a `String` built with `format!`.
- `*_args!` returns `std::fmt::Arguments` for zero-allocation formatting (use with `write!`, `format!`, etc.).

## Common Patterns

Print to stdout or stderr:

```rust
static_l10n::l10n_print!("Hello, world!");
static_l10n::l10n_eprintln!("Hello, world!");
static_l10n::f16n_println!("Hello, {}!", "Rust");
```

Write into a buffer or writer:

```rust
let mut buf = String::new();
static_l10n::l10n_write!(&mut buf, "Hello, world!");
static_l10n::f16n_writeln!(&mut buf, "Hello, {}!", "Rust");
```

Use `*_args!` with formatting APIs:

```rust
use std::fmt::Write;

let mut buf = String::new();
let args = static_l10n::f16n_args!("Hello, {}!", "Rust");
let _ = write!(&mut buf, "{}", args);
```

Localized assertions:

```rust
static_l10n::l10n_assert!(1 + 1 == 2, "Hello, world!");
static_l10n::f16n_assert_eq!(2 + 2, 4, "Hello, {}!", "Rust");
static_l10n::l10n_debug_assert_ne!(3, 4, "Hello, world!");
```

Localized panic:

```rust
static_l10n::l10n_panic!("Hello, world!");
static_l10n::f16n_panic!("Hello, {}!", "Rust");
```

## Notes

- All translations are loaded at compile time from `path`.
- Missing keys per language produce a compile-time error.
- `debug_print_metadata!()` can help inspect loaded metadata and translations during compilation.

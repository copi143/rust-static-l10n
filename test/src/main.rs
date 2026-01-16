use static_l10n::{f16n, l10n};

static_l10n::main!();

fn main() {
    static_l10n::debug_print_metadata!();
    static_l10n::lang!("zh-cn");
    println!("{}", l10n!("Hello, world!"));
    println!("{}", f16n!("Hello, {}!", "Rust"));
    println!("{}", f16n!("Hello, {name}!", name = "World"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn l10n_basic_lookup() {
        static_l10n::lang!("es");
        assert_eq!(l10n!("Hello, world!"), "¡Hola, mundo!");
        static_l10n::lang!("fr");
        assert_eq!(l10n!("Hello, world!"), "Bonjour le monde!");
    }

    #[test]
    fn f16n_formatting() {
        static_l10n::lang!("zh-cn");
        assert_eq!(f16n!("Hello, {}!", "Rust"), "你好，Rust！");
        assert_eq!(f16n!("Hello, {name}!", name = "World"), "你好，World！");
    }

    #[test]
    fn args_macros() {
        static_l10n::lang!("en");
        let msg = format!("{}", static_l10n::l10n_args!("Hello, world!"));
        assert_eq!(msg, "Hello, world!");
        let msg = format!("{}", static_l10n::f16n_args!("Hello, {}!", "Rust"));
        assert_eq!(msg, "Hello, Rust!");
    }

    #[test]
    fn write_macros() {
        static_l10n::lang!("es");
        let mut buf = String::new();
        static_l10n::l10n_write!(&mut buf, "Hello, world!");
        assert_eq!(buf, "¡Hola, mundo!");

        let mut buf = String::new();
        static_l10n::l10n_writeln!(&mut buf, "Hello, world!");
        assert_eq!(buf, "¡Hola, mundo!\n");

        let mut buf = String::new();
        static_l10n::f16n_write!(&mut buf, "Hello, {}!", "Rust");
        assert_eq!(buf, "¡Hola, Rust!");

        let mut buf = String::new();
        static_l10n::f16n_writeln!(&mut buf, "Hello, {}!", "Rust");
        assert_eq!(buf, "¡Hola, Rust!\n");
    }

    #[test]
    fn assert_macros() {
        static_l10n::lang!("en");
        static_l10n::l10n_assert!(1 + 1 == 2, "Hello, world!");
        static_l10n::l10n_assert_eq!(2 + 2, 4, "Hello, world!");
        static_l10n::l10n_assert_ne!(2 + 2, 5, "Hello, world!");
        static_l10n::l10n_debug_assert!(true, "Hello, world!");
        static_l10n::l10n_debug_assert_eq!(3, 3, "Hello, world!");
        static_l10n::l10n_debug_assert_ne!(3, 4, "Hello, world!");

        static_l10n::f16n_assert!(1 + 1 == 2, "Hello, {}!", "Rust");
        static_l10n::f16n_assert_eq!(2 + 2, 4, "Hello, {}!", "Rust");
        static_l10n::f16n_assert_ne!(2 + 2, 5, "Hello, {}!", "Rust");
        static_l10n::f16n_debug_assert!(true, "Hello, {}!", "Rust");
        static_l10n::f16n_debug_assert_eq!(3, 3, "Hello, {}!", "Rust");
        static_l10n::f16n_debug_assert_ne!(3, 4, "Hello, {}!", "Rust");
    }
}

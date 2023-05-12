/// Import assembly files relative to the current crate directory.
#[macro_export]
macro_rules! include_asm {
    ($($path:tt),+ $(,)?) => {
        $(
            // 'options(att_syntax)' really means 'stop using .intel_syntax
            // in GAS' since it's a broken mess.
            core::arch::global_asm!(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)),
                options(att_syntax)
            );
        )*
    };
}

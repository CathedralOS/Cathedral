//! Application entry contract. x86_64.S calls cathedral_user_main with boot arguments.
//! The environment variant loads installed capabilities before invoking application code.
//! Normal return and panic both terminate through the kernel task boundary.

#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn cathedral_user_main(first: u64, second: u64) -> ! {
            let main: fn(u64, u64) -> u64 = $main;
            $crate::exit(main(first, second))
        }
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
            $crate::report_panic(info);
            $crate::exit(255)
        }
    };
}

/// Opt-in entrypoint taking (profile argument, launch argument, initial endpoints).
/// Applications interpret the ordered roles; the kernel only installs authority.
#[macro_export]
macro_rules! entry_with_environment {
    ($main:path) => {
        fn cathedral_environment_entry(first: u64, second: u64) -> u64 {
            let main: fn(u64, u64, $crate::task::Environment) -> u64 = $main;
            let environment = $crate::task::Environment::load().expect("initial environment");
            main(first, second, environment)
        }
        $crate::entry!(cathedral_environment_entry);
    };
}

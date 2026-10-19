//! Application entry contract. The CPU entry stub calls cathedral_user_main with boot arguments.
//! The environment variant loads installed capabilities before invoking application code.
//! Normal return and panic both terminate through the kernel task boundary.

/// Declare matrix-hosted imports in the program image. Boot admission and the
/// legacy catalogue fixtures have their own explicit wiring; store admission
/// resolves this record against the owner's offered services before spawning.
#[macro_export]
macro_rules! service_imports {
    ($imports:expr) => {
        // Export ensures emission; user.ld KEEP retains it. #[used] would add
        // GNU retain flags and change the ELF OSABI from our System V subset.
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".cathedral.imports")]
        pub static CATHEDRAL_IMPORTS: [u8; $crate::matrix::imports::RECORD_BYTES] =
            $crate::matrix::imports::encode($imports);
    };
}

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

/// Opt into broker bootstrap and declare its required/optional service imports.
#[macro_export]
macro_rules! service_bootstrap {
    ($imports:expr) => {
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".cathedral.imports")]
        pub static CATHEDRAL_IMPORTS: [u8; $crate::matrix::imports::RECORD_BYTES] =
            $crate::matrix::imports::encode_brokered($imports);
    };
}
/// Application entry receives scoped acquisition, not an ordered endpoint bundle.
#[macro_export]
macro_rules! entry_with_services {
    ($main:path) => {
        fn cathedral_services_entry(
            first: u64,
            second: u64,
            environment: $crate::task::Environment,
        ) -> u64 {
            let services =
                $crate::services::Services::load(&environment).expect("service bootstrap");
            let main: fn(u64, u64, $crate::services::Services) -> u64 = $main;
            main(first, second, services)
        }
        $crate::entry_with_environment!(cathedral_services_entry);
    };
}

#![macro_use]

#[macro_export]
macro_rules! diag_assert {
    ($desired_level:expr, $cond:expr) => {
        if !$cond {
            $crate::diag::assert::assert($desired_level, $crate::diag::assert::rc::ResultAssertionFailed::make());
        }
    };
}

#[macro_export]
macro_rules! diag_log {
    ($logger:ty { $severity:expr, $verbosity:expr } => $msg:literal) => {
        {
            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, alloc::string::String::from($msg), file!(), cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
    ($logger:ty { $severity:expr, $verbosity:expr } => $msg:expr) => {
        {
            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, $msg, file!(), cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
    ($logger:ty { $severity:expr, $verbosity:expr } => $fmt:literal, $( $params:expr ),*) => {
        {
            let msg = format!($fmt, $( $params, )*);

            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, msg, file!(), cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
}

#[macro_export]
macro_rules! diag_log_assert {
    ($logger:ty, $desired_level:expr => $cond:expr) => {
        {
            if $cond {
                let msg = format!("Assertion suceeded -> {}", stringify!($cond));

                let mut logger = <$logger>::new();
                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Info, false, msg, file!(), cur_fn_name!(), line!());
                logger.log(&metadata);
            }
            else {
                let msg = format!("Assertion failed ({}) -> {}", stringify!($desired_level), stringify!($cond));

                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Fatal, false, msg, file!(), cur_fn_name!(), line!());
                $crate::diag::log::log_with::<$logger>(&metadata);

                $crate::diag::assert::assert($desired_level, $crate::diag::assert::rc::ResultAssertionFailed::make());
            }
        }
    };
}

#[macro_export]
macro_rules! diag_result_log_assert {
    ($logger:ty, $desired_level:expr => $rc:expr) => {
        {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                core::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let fn_name = &name[..name.len() - 3];

            if $rc.is_success() {
                let msg = format!("Result assertion suceeded -> {0} - {0:?}", $rc);

                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Info, false, msg, file!(), fn_name, line!());
                $crate::diag::log::log_with::<$logger>(&metadata);
            }
            else {
                let msg = format!("Result assertion failed ({0}) -> {1} - {1:?}", stringify!($desired_level), $rc);

                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Fatal, false, msg, file!(), fn_name, line!());
                $crate::diag::log::log_with::<$logger>(&metadata);

                $crate::diag::assert::assert($desired_level, $rc);
            }
        }
    };
}
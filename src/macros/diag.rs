#![macro_use]

#[macro_export]
macro_rules! diag_assert {
    ($desired_level:expr, $cond:expr) => {
        if !$cond {
            $crate::diag::abort::abort($desired_level, $crate::diag::rc::ResultAssertionFailed::make());
        }
    };
}

#[macro_export]
macro_rules! diag_log {
    ($logger:ty { $severity:expr, $verbosity:expr } => $msg:literal) => {
        {
            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, alloc::string::String::from($msg), file!(), $crate::cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
    ($logger:ty { $severity:expr, $verbosity:expr } => $msg:expr) => {
        {
            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, $msg, file!(), $crate::cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
    ($logger:ty { $severity:expr, $verbosity:expr } => $fmt:literal, $( $params:expr ),*) => {
        {
            let msg = format!($fmt, $( $params, )*);

            let metadata = $crate::diag::log::LogMetadata::new($severity, $verbosity, msg, file!(), $crate::cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
    };
}

#[macro_export]
macro_rules! diag_log_assert {
    ($logger:ty, $desired_abort_level:expr => $cond:expr) => {
        {
            if $cond {
                let msg = format!("Assertion succeeded: '{}'", stringify!($cond));

                let mut logger = <$logger>::new();
                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Info, false, msg, file!(), $crate::cur_fn_name!(), line!());
                logger.log(&metadata);
            }
            else {
                let msg = format!("Assertion failed: '{}' - proceeding to abort through {:?}...", stringify!($cond), $desired_abort_level);

                let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Fatal, false, msg, file!(), $crate::cur_fn_name!(), line!());
                $crate::diag::log::log_with::<$logger>(&metadata);

                $crate::diag::abort::abort($desired_abort_level, $crate::diag::rc::ResultAssertionFailed::make());
            }
        }
    };
}

#[macro_export]
macro_rules! diag_result_code_log_assert {
    ($logger:ty, $desired_abort_level:expr => $rc:expr) => {{
        let assert_rc = $rc;
        if assert_rc.is_success() {
            let msg = format!("Result assertion succeeded: '{}'", stringify!($rc_expr));

            let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Info, false, msg, file!(), $crate::cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);
        }
        else {
            let msg = format!("Result assertion failed: '{0}' was {1} ({1:?}) - proceeding to abort through {2:?}...", stringify!($rc_expr), assert_rc, $desired_abort_level);

            let metadata = $crate::diag::log::LogMetadata::new($crate::diag::log::LogSeverity::Fatal, false, msg, file!(), $crate::cur_fn_name!(), line!());
            $crate::diag::log::log_with::<$logger>(&metadata);

            $crate::diag::abort::abort($desired_abort_level, assert_rc);
        }
    }};
}

#[macro_export]
macro_rules! diag_result_log_assert {
    ($logger:ty, $desired_abort_level:expr => $rc_expr:expr) => {{
        let ret_rc = $rc_expr;
        let assert_rc = $crate::result::unpack(&ret_rc);
        $crate::diag_result_code_log_assert!($logger, $desired_abort_level => assert_rc);
        ret_rc.unwrap()
    }};
}
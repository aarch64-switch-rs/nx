#![macro_use]

#[macro_export]
macro_rules! result_define {
    ($name:ident: $module:expr, $description:expr) => {
        paste::paste! {
            pub struct [<Result $name>];

            impl $crate::result::ResultBase for [<Result $name>] {
                fn get_module() -> u32 {
                    $module
                }
                
                fn get_description() -> u32 {
                    $description
                }
            }
        }
    };
}

#[macro_export]
macro_rules! result_define_group {
    ($module:expr => { $( $name:ident: $description:expr ),* }) => {
        $( result_define!($name: $module, $description); )*
    };
}

#[macro_export]
macro_rules! result_define_subgroup {
    ($module:expr, $submodule:expr => { $( $name:ident: $description:expr ),* }) => {
        result_define_group!($module => { $( $name: $submodule + $description ),* });
    };
}

#[macro_export]
macro_rules! result_return_if {
    ($cond:expr, $res:ty) => {
        if $cond {
            return Err(<$res>::make());
        }
    };

    ($cond:expr, $res:literal) => {
        if $cond {
            return Err($crate::result::ResultCode::new($res));
        }
    };
}

#[macro_export]
macro_rules! result_return_unless {
    ($cond:expr, $res:ty) => {
        result_return_if!(!$cond, $res);
    };

    ($cond:expr, $res:literal) => {
        result_return_if!(!$cond, $res);
    };
}

#[macro_export]
macro_rules! result_try {
    ($rc:expr) => {
        if $rc.is_failure() {
            return Err($rc);
        }
    };
}


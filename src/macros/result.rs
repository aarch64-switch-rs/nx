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
        $( $crate::result_define!($name: $module, $description); )*
    };
}

#[macro_export]
macro_rules! result_define_subgroup {
    (
        $module:expr, $submodule:expr => {
            $( $name:ident: $description:expr ),*
        }
    ) => {
        $crate::result_define_group!($module => { $( $name: $submodule + $description ),* });
    };
}

#[macro_export]
macro_rules! result_return_if {
    ($cond:expr, $res:ty) => {
        let cond_val = $cond;
        if cond_val {
            return <$res>::make_err();
        }
    };

    ($cond:expr, $res:literal) => {
        let cond_val = $cond;
        if cond_val {
            return $crate::result::ResultCode::new_err($res);
        }
    };
}

#[macro_export]
macro_rules! result_return_unless {
    ($cond:expr, $res:ty) => {
        $crate::result_return_if!(!$cond, $res);
    };

    ($cond:expr, $res:literal) => {
        $crate::result_return_if!(!$cond, $res);
    };
}

#[macro_export]
macro_rules! result_try {
    ($rc_expr:expr) => {
        let rc = $rc_expr;
        if rc.is_failure() {
            return Err(rc);
        }
    };
}
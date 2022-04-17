#![macro_use]

#[macro_export]
macro_rules! rrt0_define_module_name {
    ($name:expr) => {
        #[no_mangle]
        #[used]
        #[link_section = ".module_name"]
        #[export_name = "__nx_rrt0_module_name"]
        static G_MODULE_NAME: $crate::rrt0::ModulePath = $crate::rrt0::ModulePath::new($name);
    };
}

#[macro_export]
macro_rules! rrt0_define_default_module_name {
    () => {
        rrt0_define_module_name!(env!("CARGO_PKG_NAME"));
    };
}
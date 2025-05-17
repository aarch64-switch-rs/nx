#![macro_use]

/// Defines the (runtime) module name of the current project
///
/// Must only be called once in a project
///
/// # Arguments
///
/// * `name`: The module name
///
/// # Examples
///
/// ```
/// rrt0_define_module_name!("custom-mod-name");
/// ```
#[macro_export]
macro_rules! rrt0_define_module_name {
    ($name:expr) => {
        #[unsafe(no_mangle)]
        #[used]
        #[unsafe(link_section = ".module_name")]
        #[unsafe(export_name = "__nx_rrt0_module_name")]
        static G_MODULE_NAME: $crate::rrt0::ModulePath = $crate::rrt0::ModulePath::new($name);
    };
}

/// Defines the (runtime) module name of the current project as the package name
///
/// Must only be called once in a project
///
/// # Examples
///
/// ```
/// rrt0_define_default_module_name!();
/// ```
#[macro_export]
macro_rules! rrt0_define_default_module_name {
    () => {
        rrt0_define_module_name!(env!("CARGO_PKG_NAME"));
    };
}

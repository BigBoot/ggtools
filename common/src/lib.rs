pub mod config;
pub mod default_option;
pub mod macros;
pub mod models;
pub mod utils;

use std::option::Option;

static mut APP_INFO: Option<AppInfo> = Option::None;

pub struct AppInfo {
    #[doc = "The Continuous Integration platform detected during compilation."]
    pub ci_platform: Option<String>,
    #[doc = "The full version."]
    pub pkg_version: String,
    #[doc = "The major version."]
    pub pkg_version_major: String,
    #[doc = "The minor version."]
    pub pkg_version_minor: String,
    #[doc = "The patch version."]
    pub pkg_version_patch: String,
    #[doc = "The pre-release version."]
    pub pkg_version_pre: String,
    #[doc = "A colon-separated list of authors."]
    pub pkg_authors: String,
    #[doc = "The name of the package."]
    pub pkg_name: String,
    #[doc = "The description."]
    pub pkg_description: String,
    #[doc = "The homepage."]
    pub pkg_homepage: String,
    #[doc = "The target triple that was being compiled for."]
    pub target: String,
    #[doc = "The host triple of the rust compiler."]
    pub host: String,
    #[doc = "`release` for release builds, `debug` for other builds."]
    pub profile: String,
    #[doc = "The compiler that cargo resolved to use."]
    pub rustc: String,
    #[doc = "The documentation generator that cargo resolved to use."]
    pub rustdoc: String,
    #[doc = "Value of OPT_LEVEL for the profile used during compilation."]
    pub opt_level: String,
    #[doc = "The parallelism that was specified during compilation."]
    pub num_jobs: u32,
    #[doc = "Value of DEBUG for the profile used during compilation."]
    pub debug: bool,
    #[doc = "The features that were enabled during compilation."]
    pub features: Vec<String>,
    #[doc = "The features as a comma-separated string."]
    pub features_str: String,
    #[doc = "The output of `rustc -V`"]
    pub rustc_version: String,
    #[doc = "The output of `rustdoc -V`"]
    pub rustdoc_version: String,
    #[doc = "If the crate was compiled from within a git-repository, `GIT_VERSION` contains HEAD's tag. The short commit id is used if HEAD is not tagged."]
    pub git_version: Option<String>,
    #[doc = "The built-time in RFC2822, UTC"]
    pub built_time_utc: String,
    #[doc = "The target architecture, given by `cfg!(target_arch)`."]
    pub cfg_target_arch: String,
    #[doc = "The endianness, given by `cfg!(target_endian)`."]
    pub cfg_endian: String,
    #[doc = "The toolchain-environment, given by `cfg!(target_env)`."]
    pub cfg_env: String,
    #[doc = "The OS-family, given by `cfg!(target_family)`."]
    pub cfg_family: String,
    #[doc = "The operating system, given by `cfg!(target_os)`."]
    pub cfg_os: String,
    #[doc = "The pointer width, given by `cfg!(target_pointer_width)`."]
    pub cfg_pointer_width: String,
    pub version_string: String,
}

impl AppInfo {
    pub fn get() -> &'static Self {
        unsafe {
            return APP_INFO.as_ref().unwrap_or_else(|| panic!("AppInfo has not been initialized"));
        };
    }

    pub fn set(info: Self) {
        unsafe {
            if APP_INFO.is_some() {
                panic!("AppInfo has already been initialized");
            }
            APP_INFO.replace(info);
        }
    }
}

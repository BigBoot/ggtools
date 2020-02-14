use rgcp_common::AppInfo;

include!(concat!(env!("OUT_DIR"), "/built.rs"));

pub fn init() {
    AppInfo::set(AppInfo {
        ci_platform: CI_PLATFORM.map_or(None, |x| Some(x.to_owned())),
        pkg_version: PKG_VERSION.to_owned(),
        pkg_version_major: PKG_VERSION_MAJOR.to_owned(),
        pkg_version_minor: PKG_VERSION_MINOR.to_owned(),
        pkg_version_patch: PKG_VERSION_PATCH.to_owned(),
        pkg_version_pre: PKG_VERSION_PRE.to_owned(),
        pkg_authors: PKG_AUTHORS.to_owned(),
        pkg_name: PKG_NAME.to_owned(),
        pkg_description: PKG_DESCRIPTION.to_owned(),
        pkg_homepage: PKG_HOMEPAGE.to_owned(),
        target: TARGET.to_owned(),
        host: HOST.to_owned(),
        profile: PROFILE.to_owned(),
        rustc: RUSTC.to_owned(),
        rustdoc: RUSTDOC.to_owned(),
        opt_level: OPT_LEVEL.to_owned(),
        num_jobs: NUM_JOBS.to_owned(),
        debug: DEBUG.to_owned(),
        features: FEATURES.iter().map(|x| x.to_string()).collect(),
        features_str: FEATURES_STR.to_owned(),
        rustc_version: RUSTC_VERSION.to_owned(),
        rustdoc_version: RUSTDOC_VERSION.to_owned(),
        git_version: GIT_VERSION.map_or(None, |x| Some(x.to_owned())),
        built_time_utc: BUILT_TIME_UTC.to_owned(),
        cfg_target_arch: CFG_TARGET_ARCH.to_owned(),
        cfg_endian: CFG_ENDIAN.to_owned(),
        cfg_env: CFG_ENV.to_owned(),
        cfg_family: CFG_FAMILY.to_owned(),
        cfg_os: CFG_OS.to_owned(),
        cfg_pointer_width: CFG_POINTER_WIDTH.to_owned(),
        version_string: format!("{}{}", PKG_VERSION, GIT_VERSION.map_or("".to_owned(), |x| format!(" ({})", x))),
    })
}

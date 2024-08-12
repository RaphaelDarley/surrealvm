pub mod commands;
pub mod error;

#[cfg(target_os = "linux")]
pub static OSS: &str = "linux";

#[cfg(target_os = "macos")]
pub static OSS: &str = "darwin";

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Unsupported operating system");

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub static CPU: &str = "amd64";

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub static CPU: &str = "arm64";

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "arm"
)))]
compile_error!("Unsupported CPU architecture");

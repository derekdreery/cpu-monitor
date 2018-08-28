
#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod linux;
#[cfg(not(any(windows, target_os="linux")))]
compile_error!("Only windows and linux supported, for now.");

#[cfg(windows)]
pub use self::windows::*;
#[cfg(unix)]
pub use self::linux::*;


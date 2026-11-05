use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ},
    RegKey,
};

use crate::errors::AppError;

pub fn use_sys_env_path_var() -> Result<(), AppError> {
    let key_path =
        "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
    // Open the registry key
    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            AppError::new(format!(
                "Failed to get reg key for read: {}",
                e.to_string()
            ))
        })?;

    // Read the current value of the PATH variable
    let current_path: String = reg_key.get_value("Path").map_err(|e| {
        AppError::new(format!("Failed to read PATH var: {}", e.to_string()))
    })?;

    // Update path variable in the app process
    std::env::set_var("PATH", current_path);

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn notify_environment_change() {
    #[link(name = "user32")]
    extern "system" {
        fn SendMessageTimeoutW(
            hWnd: *mut std::ffi::c_void,
            Msg: u32,
            wParam: usize,
            lParam: *const u16,
            fuFlags: u32,
            uTimeout: u32,
            lpdwResult: *mut usize,
        ) -> isize;
    }

    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let env: Vec<u16> = OsStr::new("Environment")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut result: usize = 0;
    const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut std::ffi::c_void;
    const WM_SETTINGCHANGE: u32 = 0x001A;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;

    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env.as_ptr(),
            SMTO_ABORTIFHUNG,
            3000,
            &mut result,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn notify_environment_change() {}


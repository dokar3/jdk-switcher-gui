use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ},
    RegKey,
};

pub fn use_sys_env_path_var() -> Result<(), String> {
    let key_path =
        "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
    // Open the registry key
    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| {
            format!("Failed to get reg key for read: {}", e.to_string())
        })?;

    // Read the current value of the PATH variable
    let current_path: String = reg_key
        .get_value("Path")
        .map_err(|e| format!("Failed to read PATH var: {}", e.to_string()))?;

    // Update path variable in the app process
    std::env::set_var("PATH", current_path);

    Ok(())
}

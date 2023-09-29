use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
};

pub fn find_command_exe_path(command: &str) -> Option<PathBuf> {
    let output = Command::new("where")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .output()
        .ok()?;
    if let Ok(stdout) = String::from_utf8(output.stdout) {
        if let Some(first_match) = stdout.lines().next() {
            return Some(PathBuf::from(first_match));
        }
    }
    None
}

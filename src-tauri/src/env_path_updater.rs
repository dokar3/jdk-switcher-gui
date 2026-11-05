use std::{fs::OpenOptions, io::Write, process::exit, vec};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ},
    types::ToRegValue,
    RegKey,
};

const CODE_BAD_ARGS: i32 = -10;
const CODE_OPT_FAILED: i32 = -20;

const HELP_MESSAGE: &str = r#"
Command line tool to edit system's PATH variable.

Example: 
  env-path-updater --remove "/path/1/" --add "/path/2/"

Args:
  -a, --add     Add a path to the variable.
  -r, --remove  Remove a path from the variable.
  -i, --id      Specify the execution id, which will be written to the result file.
  -h, --help    Print help message.
"#;

#[derive(Debug)]
enum CliCommand {
    Help,
    None,
    ExecId(String),
    AddPath(String),
    RemovePath(String),
}

/// A command line executable that requires to be run as admin
/// to update the system's PATH variable.
fn main() {
    let commands = parse_commands();

    if commands.is_err() {
        eprintln!("{}", commands.unwrap_err());
        exit(CODE_BAD_ARGS);
    }

    let result_file_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("env-path-updater.log");
    let mut result_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(result_file_path)
        .unwrap();

    let commands = commands.unwrap();

    if let Some(exec_id) = commands.iter().find_map(|item| match item {
        CliCommand::ExecId(id) => Some(id),
        _ => None,
    }) {
        // Write exec id at the first line
        result_file
            .write(format!("ID: {}\n", exec_id).as_bytes())
            .unwrap();
        result_file.flush().unwrap();
    }

    let mut updated_any = false;
    for command in commands {
        let ret = match command {
            CliCommand::Help | CliCommand::None => {
                Ok(println!("{}", HELP_MESSAGE))
            }
            CliCommand::AddPath(path) => {
                let r = add_to_env_path(&path);
                if r.is_ok() {
                    updated_any = true;
                }
                r
            }
            CliCommand::RemovePath(path) => {
                let r = remove_from_env_path(&path);
                if r.is_ok() {
                    updated_any = true;
                }
                r
            }
            CliCommand::ExecId(_) => Ok(()),
        };
        if let Err(e) = ret {
            eprintln!("{}", e);
            // Log error result
            result_file.write(b"ERR\n").unwrap();
            result_file.write(e.as_bytes()).unwrap();
            result_file.flush().unwrap();
            exit(CODE_OPT_FAILED);
        }
    }

    if updated_any {
        notify_environment_change();
    }

    // Log success result
    result_file.write(b"OK").unwrap();
    result_file.flush().unwrap();

    exit(0);
}

fn parse_commands() -> Result<Vec<CliCommand>, String> {
    let mut args = std::env::args().skip(1);

    if args.len() == 0 {
        return Ok(vec![CliCommand::None]);
    }

    let mut commands: Vec<CliCommand> = vec![];

    while let Some(cmd) = args.next() {
        match cmd.as_str() {
            "-h" | "--help" => return Ok(vec![CliCommand::Help]),
            "-a" | "--add" => {
                let path =
                    args.next().ok_or(format!("Missing path after {}", cmd))?;
                commands.push(CliCommand::AddPath(path))
            }
            "-r" | "--remove" => {
                let path =
                    args.next().ok_or(format!("Missing path after {}", cmd))?;
                commands.push(CliCommand::RemovePath(path))
            }
            "-i" | "--id" => {
                let id = args
                    .next()
                    .ok_or(format!("Missing exec id after {}", cmd))?;
                commands.push(CliCommand::ExecId(id))
            }
            _ => return Err(format!("Unknown command {}.", cmd)),
        }
    }

    Ok(commands)
}

fn paths_match(a: &str, b: &str) -> bool {
    let trim_a = a.trim().trim_end_matches(['\\', '/']);
    let trim_b = b.trim().trim_end_matches(['\\', '/']);
    trim_a.eq_ignore_ascii_case(trim_b)
}

fn add_to_env_path(value: &str) -> Result<(), String> {
    update_env_path(|path| {
        let mut entries: Vec<String> = path
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if entries.iter().any(|entry| paths_match(entry, value)) {
            return path;
        }

        let clean_value = value.trim().trim_end_matches(['\\', '/']).to_string();
        entries.push(clean_value);

        let mut res = entries.join(";");
        res.push(';');
        res
    })
}

fn remove_from_env_path(value: &str) -> Result<(), String> {
    update_env_path(|path| {
        let entries: Vec<&str> = path
            .split(';')
            .map(|s| s.trim())
            .filter(|entry| !entry.is_empty() && !paths_match(entry, value))
            .collect();

        if entries.is_empty() {
            String::new()
        } else {
            let mut res = entries.join(";");
            res.push(';');
            res
        }
    })
}

/// Update system's PATH variable.
///
/// # Examples:
/// ```rust no_run
/// update_env_path(|path| path.replace("/path/to/my_program;", ""));
/// ```
fn update_env_path<F>(closure: F) -> Result<(), String>
where
    F: FnOnce(String) -> String,
{
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

    // Update value
    let updated = closure(current_path.clone());
    if updated == current_path {
        return Ok(());
    }

    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(key_path, KEY_SET_VALUE)
        .map_err(|e| {
            format!("Failed to get reg key for update: {}", e.to_string())
        })?;

    // Update the PATH variable in the registry as REG_EXPAND_SZ so variables are expanded properly
    let mut raw_val = updated.to_reg_value();
    raw_val.vtype = REG_EXPAND_SZ;
    reg_key
        .set_raw_value("Path", &raw_val)
        .map_err(|e| format!("Failed to update PATH var: {}", e.to_string()))
}

#[cfg(target_os = "windows")]
fn notify_environment_change() {
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
fn notify_environment_change() {}

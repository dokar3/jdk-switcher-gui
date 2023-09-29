use std::{fs::OpenOptions, io::Write, process::exit, vec};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE},
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

    for command in commands {
        let ret = match command {
            CliCommand::Help | CliCommand::None => Ok(println!("{}", HELP_MESSAGE)),
            CliCommand::AddPath(path) => add_to_env_path(&path),
            CliCommand::RemovePath(path) => remove_from_env_path(&path),
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
                let path = args.next().ok_or(format!("Missing path after {}", cmd))?;
                commands.push(CliCommand::AddPath(path))
            }
            "-r" | "--remove" => {
                let path = args.next().ok_or(format!("Missing path after {}", cmd))?;
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

fn add_to_env_path(value: &str) -> Result<(), String> {
    update_env_path(|path| {
        if path.contains(value) {
            return path;
        }
        let mut new_path = path.clone();
        new_path.push_str(value);
        new_path.push(';');
        new_path
    })
}

fn remove_from_env_path(value: &str) -> Result<(), String> {
    let var = format!("{};", value);
    update_env_path(|path| path.replace(&var, ""))
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
    let key_path = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
    // Open the registry key
    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(key_path, KEY_READ)
        .map_err(|e| format!("Failed to get reg key for read: {}", e.to_string()))?;

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
        .map_err(|e| format!("Failed to get reg key for update: {}", e.to_string()))?;
    // Update the PATH variable in the registry
    reg_key
        .set_value("Path", &updated)
        .map_err(|e| format!("Failed to update PATH var: {}", e.to_string()))
}

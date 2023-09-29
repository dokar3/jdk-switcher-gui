use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    model::jdk::Jdk,
    util::{self, paths::find_command_exe_path},
};

pub fn switch_to_jdk(jdk: &Jdk) -> Result<(), String> {
    let path = PathBuf::from(&jdk.path);
    if !path.exists() {
        return Err("Target jdk path does not exist.".to_string());
    }

    let curr_java_bin_dir = find_curr_java_bin_dir();
    let to_add = path.to_str().unwrap();

    let exec_args = if let Some(to_remove) = curr_java_bin_dir.as_ref() {
        vec!["--remove", to_remove, "--add", to_add]
    } else {
        vec!["--add", to_add]
    };
    // Updater system env var
    let ret = exec_env_path_updater(exec_args);

    if ret.is_ok() {
        // Update path var of the current process
        util::env::use_sys_env_path_var()?;
    }

    ret
}

fn find_curr_java_bin_dir() -> Option<String> {
    let curr_java_path = find_command_exe_path("java");
    if curr_java_path.is_none() {
        return None;
    }
    if let Some(curr_java_bin_dir) = curr_java_path.unwrap().parent() {
        Some(curr_java_bin_dir.to_str().unwrap().to_string())
    } else {
        None
    }
}

fn exec_env_path_updater(args: Vec<&str>) -> Result<(), String> {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let program = exe_dir
        .join("env-path-updater")
        .to_str()
        .unwrap()
        .to_string();

    // Unique id used to verify execute result.
    let exec_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    // Execute command as admin
    let status = runas::Command::new(program)
        .args(&args)
        .args(&["--id", &exec_id])
        .status()
        .map_err(|e| e.to_string())?;
    let _ = status.success();

    let result_file_path = exe_dir.join("env-path-updater.log");
    let mut paused_millis = 0;
    let mut prev_err = "Internal error.".to_string();

    while paused_millis < 2000 {
        // Verify the result file
        if let Err(e) = verify_exec_result(&result_file_path, &exec_id) {
            std::thread::sleep(Duration::from_millis(50));
            prev_err = e;
            paused_millis += 50;
        } else {
            return Ok(());
        }
    }

    Err(prev_err)
}

fn verify_exec_result(result_file_path: &PathBuf, exec_id: &str) -> Result<(), String> {
    if !result_file_path.exists() {
        // Result file not found, failed
        return Err("Update result not found.".to_string());
    }
    let lines: Vec<String> = std::fs::read_to_string(result_file_path)
        .map_err(|e| format!("Cannot read update result: {}.", e.to_string()))?
        .lines()
        .map(|s| s.to_owned())
        .collect();
    if lines.is_empty() {
        // Empty result file, failed
        return Err("Empty update result.".to_string());
    }
    if lines.len() < 2 {
        // Unsupported result, failed
        return Err("Unsupported update result.".to_string());
    }
    if lines[0] != format!("ID: {}", exec_id) {
        // ID not matched, failed
        return Err("Target update result not found.".to_string());
    }
    // Verify update result
    match lines[1].as_str() {
        "ERR" => {
            let err = if lines.len() > 2 {
                lines[2].to_owned()
            } else {
                "Unknown error.".to_string()
            };
            Err(err)
        }
        "OK" => Ok(()),
        _ => Err(format!("Unknown update result '{}'", lines[1])),
    }
}

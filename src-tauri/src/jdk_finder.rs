use std::{fs::DirEntry, path::PathBuf, process::Command, os::windows::process::CommandExt};

use crate::model::jdk::Jdk;

pub fn find_jdks_from_dir(path: &PathBuf) -> Result<Vec<Jdk>, String> {
    if !path.exists() {
        return Err("Dir does not exist.".to_string());
    }
    let mut jdks: Vec<Jdk> = vec![];
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else {
            return Err("Cannot read dir.".to_string());
        };
        let files: Vec<DirEntry> = entries
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect();

        // Find the java exe
        let java = files
            .iter()
            .find(|item| item.path().is_file() && item.file_name() == get_java_executable());
        if java.is_some() {
            let jdk = find_jdk_from_exe_path(&java.unwrap().path());
            if jdk.is_ok() {
                jdks.push(jdk.unwrap());
            }
            return Ok(jdks);
        }

        // Find the bin dir
        let bin_dir = files.iter().find(|item| {
            let path = item.path();
            path.is_dir() && path.file_name().unwrap() == "bin"
        });
        if bin_dir.is_some() {
            let sub_jdks = find_jdks_from_dir(&bin_dir.unwrap().path());
            if sub_jdks.is_ok() {
                jdks.extend(sub_jdks.unwrap());
            }
            return Ok(jdks);
        }

        // Try every sub dir
        for file in files {
            let file_path = file.path();
            if file_path.is_dir() {
                let sub_jdks = find_jdks_from_dir(&file_path);
                if sub_jdks.is_ok() {
                    jdks.extend(sub_jdks.unwrap());
                }
            }
        }

        Ok(jdks)
    } else {
        Err("Target path is not a directory.".to_string())
    }
}

pub fn find_jdk_from_exe_path(path: &PathBuf) -> Result<Jdk, String> {
    if !path.exists() {
        return Err("Target exe does not exist.".to_string());
    }
    let output = Command::new(path.as_os_str())
        .arg("-version")
        .stdout(std::process::Stdio::piped())
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;
    // java --version -> stderr
    // java -version  -> stdout
    let Ok(stderr) = String::from_utf8(output.stderr) else {
        return Err("Cannot not read jdk -version output".to_string());
    };
    let lines: Vec<&str> = stderr.lines().take(3).collect();
    if lines.len() < 3 {
        return Err("Unsupported -version output".to_string());
    }
    let version = parse_version(lines[0]).unwrap_or("Unknown".to_string());
    let name = parse_name(lines[1]);
    let arch = parse_arch(lines[2]);
    Ok(Jdk {
        name,
        path: path.parent().unwrap().to_str().unwrap().to_string(),
        version,
        arch,
        is_current: false,
        is_valid: true,
    })
}

fn parse_version(first_line: &str) -> Option<String> {
    if let Some(start) = first_line.find('"') {
        if let Some(end) = first_line[start + 1..].find('"') {
            Some((&first_line[start + 1..start + 1 + end]).to_string())
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_name(second_line: &str) -> String {
    let mut splits = second_line.split("Runtime Environment");
    splits.nth(0).unwrap().to_string()
}

fn parse_arch(third_line: &str) -> String {
    if third_line.contains("64-Bit") {
        "64-Bit".to_string()
    } else {
        "32-Bit".to_string()
    }
}

const fn get_java_executable() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "java.exe"
    }

    #[cfg(not(target_os = "windows"))]
    {
        "java"
    }
}

#[cfg(test)]
mod test {
    use super::parse_version;

    #[test]
    fn test_parse_jdk_version() {
        assert_eq!(
            "21",
            parse_version("openjdk version \"21\" 2023-09-19").unwrap()
        );
    }
}

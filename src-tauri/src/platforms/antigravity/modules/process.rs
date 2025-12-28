use std::process::Command;
use std::path::PathBuf;
use sysinfo::{System, ProcessRefreshKind, ProcessesToUpdate};

/// 检查 Antigravity 是否正在运行
pub fn is_antigravity_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    sys.processes().values().any(|process| {
        let name = process.name().to_string_lossy().to_lowercase();
        name.contains("antigravity")
    })
}

/// 杀死所有 Antigravity 进程
pub fn kill_antigravity_processes() -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    let mut killed_count = 0;

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("antigravity") {
            if process.kill() {
                killed_count += 1;
                println!("Killed Antigravity process: {} (PID: {})", process.name().to_string_lossy(), pid);
            }
        }
    }

    if killed_count > 0 {
        // 等待进程完全退出
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    } else {
        Err("No Antigravity processes found".to_string())
    }
}

/// 获取 Antigravity 可执行文件路径
pub fn get_antigravity_executable_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity");
        if path.exists() {
            return Ok(path);
        }
        Err("Antigravity not found in /Applications".to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::env;
        
        let local_appdata = env::var("LOCALAPPDATA").ok();
        let program_files = env::var("ProgramFiles")
            .unwrap_or_else(|_| "C:\\Program Files".to_string());
        
        let mut possible_paths = Vec::new();
        
        if let Some(local) = local_appdata {
            possible_paths.push(
                PathBuf::from(&local)
                    .join("Programs")
                    .join("Antigravity")
                    .join("Antigravity.exe")
            );
        }
        
        possible_paths.push(
            PathBuf::from(&program_files)
                .join("Antigravity")
                .join("Antigravity.exe")
        );
        
        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err("Antigravity not found".to_string())
    }
    
    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            PathBuf::from("/usr/bin/antigravity"),
            PathBuf::from("/usr/local/bin/antigravity"),
            PathBuf::from("/opt/Antigravity/antigravity"),
        ];
        
        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err("Antigravity not found".to_string())
    }
}

/// 启动 Antigravity
pub fn launch_antigravity() -> Result<(), String> {
    let exe_path = get_antigravity_executable_path()?;
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg(exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Antigravity: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Antigravity: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Antigravity: {}", e))?;
    }
    
    Ok(())
}


use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use std::{
    ffi::OsStr,
    io,
    process::Command,
    thread::sleep,
    time::{Duration, Instant},
};
use log::{info, warn};


pub fn ensure_steam_stopped(timeout : Duration) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let steam_name = "steam.exe";

    #[cfg(target_family = "unix")]
    let steam_name = "steam";

    let os_steam_name = OsStr::new(steam_name);

    let s = System::new_all();
    let processes = s.processes_by_name(os_steam_name);
    for process in processes {
        let mut s = System::new();

        let quit_res = process.kill_with(sysinfo::Signal::Quit);
        let kill_res = process.kill_with(sysinfo::Signal::Kill);

        if quit_res == Some(false) && kill_res == Some(false) {
            // Couldn't kill the process, this could be because it was already killed or we don't have permissions
            // For instance, the process "steamos-manager" in the Steam Deck is owned by root
            continue;
        }
        
        let pid = process.pid();
        let process_name = process.name();
        let pid_arr = [pid];
        let process_to_update = ProcessesToUpdate::Some(&pid_arr);

        while s.refresh_processes_specifics(process_to_update, true,ProcessRefreshKind::everything()) > 0 
                // The process is still alive
                && s.process(pid).is_some()
                 {
            info!("Waiting for steam to stop. PID: {pid:?} Name: {process_name:?}");
            sleep(timeout);
            process.kill_with(sysinfo::Signal::Term);
            process.kill_with(sysinfo::Signal::Kill);
        }
    }
    info!("Steam is stopped");
    return Ok(());
}

/// Launch Steam (platform-specific).
pub fn start_steam() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Use URL handler so we don't care where Steam is installed.
        // `start "" "steam://open/main"` launches detached.
        Command::new("cmd")
            .args(["/C", "start", "", "steam://open/main"])
            .spawn()
            .map(|_| ())?;
        info!("Starting Steam (Windows)…");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        // Launch the app by bundle id/name.
        Command::new("open")
            .args(["-a", "Steam"])
            .spawn()
            .map(|_| ())?;
        info!("Starting Steam (macOS)…");
        return Ok(());
    }

    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        // Try a few common launch methods; succeed on the first that works.
        // 1) Native package, 2) Flatpak, 3) systemd user service (SteamOS/Deck), 4) fallback.
        let tries: &[&dyn Fn() -> io::Result<()>] = &[
            &|| Command::new("steam").arg("-silent").spawn().map(|_| ()),
            &|| Command::new("flatpak")
                .args(["run", "com.valvesoftware.Steam"])
                .spawn()
                .map(|_| ()),
            &|| Command::new("systemctl")
                .args(["--user", "start", "steam"])
                .spawn()
                .map(|_| ()),
            &|| Command::new("steam").spawn().map(|_| ()),
        ];

        let mut last_err: Option<io::Error> = None;
        for f in tries {
            match f() {
                Ok(()) => {
                    info!("Starting Steam (Unix)…");
                    return Ok(());
                }
                Err(e) => last_err = Some(e),
            }
        }
        return Err(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not launch Steam")
        }));
    }
}

/// Wait until Steam shows up in the process list (or time out).
pub fn ensure_steam_started(timeout: Duration) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let steam_name = "steam.exe";
    #[cfg(target_family = "unix")]
    let steam_name = "steam";

    let os_steam_name = OsStr::new(steam_name);

    let start = Instant::now();
    let mut s = System::new_all();

    loop {
        s.refresh_processes(ProcessesToUpdate::All, true);
        if s.processes_by_name(os_steam_name).next().is_some() {
            info!("Steam is running");
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Steam did not start in time"));
        }
        sleep(Duration::from_millis(500));
    }
}
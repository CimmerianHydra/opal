use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use std::{
    ffi::OsStr, fs::{read, write}, io, process::Command, thread::sleep, time::{Duration, Instant}
};
use log::{
    info
};
use steam_shortcuts_util::{
    parse_shortcuts,
    shortcuts_to_bytes,
    shortcut::{Shortcut, ShortcutOwned},
    app_id_generator::calculate_app_id_for_shortcut,
};
use std::path::{Path, PathBuf};
use steamlocate::*;

// Wait until Steam has stopped
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

/// Your app's "desired shortcut" input. Adapt as needed.
#[derive(Debug, Clone)]
pub struct DesiredShortcut {
    pub app_name: String,
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub launch_options: String,
    pub tags: Vec<String>,
    // Optional: populate if you use Steam's "Shortcut Path" field
    pub shortcut_path: String,
}
impl DesiredShortcut {
  pub fn make_owned(&self, order: usize) -> ShortcutOwned {
      // Build with borrowed &strs just for this call, immediately convert to owned.
      let order_string = order.to_string();
      let tmp: Shortcut = Shortcut::new(
          &order_string,         // order is a string in VDF
          &self.app_name,
          &self.exe,
          &self.start_dir,
          &self.icon,
          &self.shortcut_path,
          &self.launch_options,
      );

      let mut owned = tmp.to_owned();
      // Tags are owned strings on `ShortcutOwned`
      owned.tags = self.tags.clone();

      // Compute app_id using the borrowed view of our owned struct
      owned.app_id = calculate_app_id_for_shortcut(&owned.borrow());

      // Sensible defaults (match crate’s intent)
      owned.is_hidden = false;
      owned.allow_desktop_config = true;
      owned.allow_overlay = true;
      owned.open_vr = 0;
      owned.dev_kit = 0;
      owned.dev_kit_overrite_app_id = 0;
      owned.last_play_time = 0;

      owned
  }
}

pub fn write_steam_shortcuts(path: &Path, desired_vec: Vec<DesiredShortcut>) -> io::Result<()> {

        if !path.exists() { return Err(io::Error::last_os_error()) };
        // Make sure the content exists and can be successfully read. If not, print out error.
        // Immediately break lifetimes with `to_owned`.
        let mut existing_owned: Vec<ShortcutOwned> = {
            let bytes = read(path).unwrap_or_default();
            let parsed: Vec<Shortcut> = parse_shortcuts(bytes.as_slice())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("parse: {e}")))
                .unwrap();
            parsed.into_iter().map(|s| s.to_owned()).collect()
        };

        // Index existing by app_id (stable identifier for Steam assets).
        let mut by_id: std::collections::BTreeMap<u32, ShortcutOwned> =
            existing_owned.drain(..).map(|s| (s.app_id, s)).collect();
        

        for (i, d) in desired_vec.iter().enumerate() {
            let sc = d.make_owned(i);
            // If you prefer "app name + exe" as the identity instead of app_id, change this keying.
            by_id.insert(sc.app_id, sc);
        }
        
        // Rebuild a stable, ordered list and fix the `order` field.
        let mut final_owned: Vec<ShortcutOwned> = by_id.into_values().collect();
        final_owned.sort_by(|a, b| a.app_name.cmp(&b.app_name)); // or whatever ordering you like
        for (i, s) in final_owned.iter_mut().enumerate() {
            s.order = i.to_string();
        }

        // Borrow-on-demand to serialize.
        // NOTE: `shortcuts_to_bytes` wants `Vec<Shortcut<'_>>`, so produce a borrowed view.
        let borrowed: Vec<Shortcut> = final_owned.iter().map(|s| s.borrow()).collect();
        let out = shortcuts_to_bytes(&borrowed);

        // Write back to disk.
        write(path, out)
    }

pub fn default_steam_path() -> Result<PathBuf> {
    let directory_as_ref = SteamDir::locate()?.path().to_owned();
    return Ok(directory_as_ref)
}
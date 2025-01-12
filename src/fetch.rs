use crate::{ERROR_STR, NOT_AVAILABLE_STR};
use std::{
    env,
    fs::{read_dir, File},
    io::Read,
    path::Path,
    process::Command,
    thread,
    time::Duration,
};
use whoami::fallible::{distro, hostname, username};

pub fn get_distro() -> String {
    match distro() {
        Ok(x) => x,
        Err(_) => String::from(NOT_AVAILABLE_STR),
    }
}

pub fn get_hostname() -> String {
    match hostname() {
        Ok(x) => x,
        Err(_) => String::from(NOT_AVAILABLE_STR),
    }
}

pub fn get_kernel() -> String {
    let result = File::open("/proc/version").and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let parts: Vec<&str> = contents.split_whitespace().collect();
        if parts.len() > 2 {
            contents = parts[2].to_string();
        }
        Ok(contents)
    });

    match result {
        Ok(x) => x,
        Err(_) => String::from(NOT_AVAILABLE_STR),
    }
}

pub fn get_pkgs() -> String {
    let home = match env::var("HOME") {
        Ok(p) => p,
        Err(_) => return String::from(ERROR_STR),
    };

    let pacman_count = read_dir("/var/lib/pacman/local")
        .map(|entries| entries.count())
        .unwrap_or(0);

    let xbps_count = read_dir("/var/db/xbps")
        .map(|entries| entries.count())
        .unwrap_or(0);

    let apt_count = read_dir("/var/lib/dpkg/info")
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "list"))
                .count()
        })
        .unwrap_or(0);

    let flatpak_count = {
        let system_count = read_dir("/var/lib/flatpak/app")
            .map(|entries| entries.count())
            .unwrap_or(0);

        let user_count = read_dir(String::from(&home) + "/.local/share/flatpak/app")
            .map(|entries| entries.count())
            .unwrap_or(0);

        system_count + user_count
    };

    let nix_count = {
        let system_handle = thread::spawn(move || {
            Command::new("nix-store")
                .args(["--query", "--requisites", "/run/current-system"])
                .output()
                .ok()
                .and_then(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .count()
                        .into()
                })
                .unwrap_or(0)
        });

        let user_handle = thread::spawn(move || {
            Command::new("nix-store")
                .args([
                    "--query",
                    "--requisites",
                    &(String::from(&home) + "/.nix-profile"),
                ])
                .output()
                .ok()
                .and_then(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .count()
                        .into()
                })
                .unwrap_or(0)
        });

        system_handle.join().unwrap() + user_handle.join().unwrap()
    };

    let mut output = Vec::new();
    if pacman_count != 0 {
        output.push(format!("{} (pacman)", pacman_count));
    }
    if xbps_count != 0 {
        output.push(format!("{} (xbps)", xbps_count));
    }
    if apt_count != 0 {
        output.push(format!("{} (apt)", apt_count));
    }
    if flatpak_count != 0 {
        output.push(format!("{} (flatpak)", flatpak_count));
    }
    if nix_count != 0 {
        output.push(format!("{} (nix)", nix_count));
    }

    if output.is_empty() {
        return String::from(NOT_AVAILABLE_STR);
    }

    output.join(", ")
}

pub fn get_shell() -> String {
    match env::var("SHELL") {
        Ok(x) => String::from(x.rsplit('/').next().unwrap()),
        Err(_) => String::from(NOT_AVAILABLE_STR),
    }
}

pub fn get_uptime() -> String {
    let mut file = match File::open(Path::new("/proc/uptime")) {
        Ok(f) => f,
        Err(_) => return String::from(NOT_AVAILABLE_STR),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return String::from(NOT_AVAILABLE_STR);
    }

    let uptime_seconds = contents.split_whitespace().next();
    let uptime_seconds = match uptime_seconds {
        Some(s) => match s.parse::<f64>() {
            Ok(seconds) => seconds,
            Err(_) => return String::from(NOT_AVAILABLE_STR),
        },
        None => return String::from(NOT_AVAILABLE_STR),
    };

    let uptime_duration = Duration::from_secs_f64(uptime_seconds);

    let days = uptime_duration.as_secs() / 86400;
    let hours = (uptime_duration.as_secs() % 86400) / 3600;
    let minutes = (uptime_duration.as_secs() % 3600) / 60;

    let mut uptime_string = String::new();
    if days > 0 {
        uptime_string.push_str(&format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        if !uptime_string.is_empty() {
            uptime_string.push_str(", ");
        }
        uptime_string.push_str(&format!(
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        ));
    }
    if minutes > 0 {
        if !uptime_string.is_empty() {
            uptime_string.push_str(", ");
        }
        uptime_string.push_str(&format!(
            "{} minute{}",
            minutes,
            if minutes > 1 { "s" } else { "" }
        ));
    }
    if uptime_string.is_empty() {
        uptime_string.push_str("0 minutes");
    }

    uptime_string
}

pub fn get_username() -> String {
    match username() {
        Ok(x) => x,
        Err(_) => String::from(NOT_AVAILABLE_STR),
    }
}

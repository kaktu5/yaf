use crate::NOT_AVAILABLE;
use std::{
    fs::{read_dir, File},
    io::Read,
    path::Path,
    time::Duration,
};
use whoami::fallible::{distro, hostname, username};

pub fn get_username() -> String {
    match username() {
        Ok(x) => String::from(x),
        Err(_) => String::from(NOT_AVAILABLE),
    }
}

pub fn get_hostname() -> String {
    match hostname() {
        Ok(x) => String::from(x),
        Err(_) => String::from(NOT_AVAILABLE),
    }
}

pub fn get_distro() -> String {
    match distro() {
        Ok(x) => String::from(x),
        Err(_) => String::from(NOT_AVAILABLE),
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
        Err(_) => String::from(NOT_AVAILABLE),
    }
}

pub fn get_uptime() -> String {
    let mut file = match File::open(Path::new("/proc/uptime")) {
        Ok(f) => f,
        Err(_) => return String::from(NOT_AVAILABLE),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return String::from(NOT_AVAILABLE);
    }

    let uptime_seconds = contents.split_whitespace().next();
    let uptime_seconds = match uptime_seconds {
        Some(s) => match s.parse::<f64>() {
            Ok(seconds) => seconds,
            Err(_) => return String::from(NOT_AVAILABLE),
        },
        None => return String::from(NOT_AVAILABLE),
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

pub fn get_pkgs() -> String {
    let pacman_count: usize = read_dir("/var/lib/pacman/local")
        .map(|entries| entries.count())
        .unwrap_or(0);

    let xbps_count: usize = read_dir("/var/db/xbps")
        .map(|entries| entries.count())
        .unwrap_or(0);

    let apt_count: usize = read_dir("/var/lib/dpkg/info")
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "list"))
                .count()
        })
        .unwrap_or(0);

    let flatpak_count: usize = {
        let system_count: usize = read_dir("/var/lib/flatpak/app")
            .map(|entries| entries.count())
            .unwrap_or(0);

        let user_count: usize = read_dir(format!(
            "{}/.local/share/flatpak/app",
            std::env::var("HOME").unwrap_or_default()
        ))
        .map(|entries| entries.count())
        .unwrap_or(0);

        system_count + user_count
    };

    let mut output = Vec::new();
    if pacman_count > 0 {
        output.push(format!("{} (pacman)", pacman_count));
    }
    if xbps_count > 0 {
        output.push(format!("{} (xbps)", xbps_count));
    }
    if apt_count > 0 {
        output.push(format!("{} (apt)", apt_count));
    }
    if flatpak_count > 0 {
        output.push(format!("{} (flatpak)", flatpak_count));
    }

    if output.is_empty() {
        return String::from(NOT_AVAILABLE);
    }

    output.join(", ")
}

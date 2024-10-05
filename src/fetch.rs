use log::warn;
use std::{fs::File, io::{Read, Error, ErrorKind}, time::Duration};
use whoami::fallible::{distro, hostname, username};

pub fn get_username() -> String {
    username().unwrap_or_else(|err| {
        warn!("Failed to get username. {:?}", err);
        String::from("unknown")
    })
}

pub fn get_hostname() -> String {
    hostname().unwrap_or_else(|err| {
        warn!("Failed to get hostname. {:?}", err);
        String::from("unknown")
    })
}

pub fn get_distro() -> String {
    distro().unwrap_or_else(|err| {
        warn!("Failed to get distro. {:?}", err);
        String::from("unknown")
    })
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

    result.unwrap_or_else(|err| {
        warn!("Failed to get kernel version. {:?}", err);
        String::from("unknown")
    })
}

pub fn get_uptime() -> String {
    let result = File::open("/proc/uptime").and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let uptime_seconds = contents.split_whitespace()
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Malformed /proc/uptime"))?
            .parse::<f64>()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid uptime format"))?;

        Ok(Duration::from_secs_f64(uptime_seconds))
    });

    let uptime_duration = result.unwrap_or_else(|err| {
        warn!("Failed to get uptime. {:?}", err);
        return Duration::from_secs(0);
    });

    let days = uptime_duration.as_secs() / 86400;
    let hours = (uptime_duration.as_secs() % 86400) / 3600;
    let minutes = (uptime_duration.as_secs() % 3600) / 60;

    let mut uptime_string = String::new();
    if days > 0 {
        uptime_string.push_str(&format!("{} days", days));
    }
    if hours > 0 {
        if !uptime_string.is_empty() {
            uptime_string.push_str(", ");
        }
        uptime_string.push_str(&format!("{} hours", hours));
    }
    if minutes > 0 {
        if !uptime_string.is_empty() {
            uptime_string.push_str(", ");
        }
        uptime_string.push_str(&format!("{} minutes", minutes));
    }

    uptime_string
}

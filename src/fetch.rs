use log::warn;
use std::{fs::File, io::Read};
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

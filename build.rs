use std::{fs::read_to_string, path::Path};

fn main() {
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", get_commit_hash());
}

fn get_commit_hash() -> String {
    let git_head =
        read_to_string(Path::new(".git/HEAD")).unwrap_or_else(|_| String::from("unknown"));
    let ref_path = Path::new(".git").join(git_head.split_whitespace().last().unwrap());
    read_to_string(ref_path)
        .unwrap_or_else(|_| String::from("unknown"))
        .trim()[..7]
        .to_string()
}

use std::process::Command;
fn main() {
    let git_describe_command = Command::new("git")
        .args(&["describe"])
        .output();
    let git_hash = if let Ok(git_describe) = git_describe_command {
        String::from_utf8_lossy(&git_describe.stdout).to_string()
    } else {
        "v0.0.0-UNKNOWN".to_string()
    };
    println!("cargo:rustc-env=GIT_DESCRIBE={}", git_hash);
}

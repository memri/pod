use std::process::Command;
fn main() {
    let output = Command::new("git")
        .args(&["describe"])
        .output()
        .expect("Failed to obtain git information in build.rs");
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_DESCRIBE={}", git_hash);
}

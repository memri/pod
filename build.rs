use std::process::Command;
fn main() {
    let git_describe = Command::new("git")
        .args(&["describe"])
        .output()
        .ok()
        .filter(|out| out.status.success());

    if let Some(git_describe) = git_describe {
        println!(
            "cargo:rustc-env=GIT_DESCRIBE={}",
            String::from_utf8_lossy(&git_describe.stdout)
        );
    };
}

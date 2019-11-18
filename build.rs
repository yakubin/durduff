use std::env;

use std::fs::File;

use std::io::Write;

use std::path::Path;

use std::process;

fn get_git_commit_hash() -> String {
    if !Path::new(".git").is_dir() {
        return "[not built from git repo]".to_string();
    }

    let git_result = process::Command::new("git")
        .arg("rev-list")
        .arg("--all")
        .arg("--max-count=1")
        .output();

    let git_output = match git_result {
        Ok(o) => o,
        Err(_) => panic!("could not run git-rev-list"),
    };

    let git_exit_code = git_output
        .status
        .code()
        .expect("git-rev-list process terminated by signal");

    if !git_output.status.success() {
        panic!("git-rev-list failed. exit code: {}", git_exit_code);
    }

    let git_stdout = String::from_utf8(git_output.stdout).expect("could not parse git output");

    git_stdout.trim().to_string()
}

fn main() {
    let git_commit = get_git_commit_hash();

    let host_triple = env::var("HOST").unwrap();
    let target_triple = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");
    let mut f = File::create(&dest_path).unwrap();

    write!(
        f,
        "
        pub fn print_build_info() {{
            println!(\"git commit: {}\");
            println!(\"host triple: {}\");
            println!(\"target triple: {}\");
            println!(\"profile: {}\");
        }}
    ",
        git_commit, host_triple, target_triple, profile
    )
    .unwrap();

    // rerun if anything changed in git (e.g. created a new commit)
    println!("cargo:rerun-if-changed=.git");

    // rerun if anything changed in the crate tree
    println!("cargo:rerun-if-changed=.");
}

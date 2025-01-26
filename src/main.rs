use std::env;
use std::process::Command;
use std::path::Path;
use anyhow::Result;

fn clone_repo(repo_url: &str, repo_tag: &str) -> Result<()> {
    let sources_dir = Path::new("/phiban/sources");
    env::set_current_dir(sources_dir)?;

    let status = Command::new("git").arg("clone")
        .arg("--single-branch")
        .arg("--branch").arg(repo_tag)
        .arg(repo_url)
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    Ok(())
}

fn handle_musl() -> Result<()> {
    clone_repo("/git_sources/musl", "v1.2.5")?;

    let source_dir = Path::new("/phiban/sources/musl");
    env::set_current_dir(source_dir)?;

    let status = Command::new("./configure")
        .arg("--prefix=/sysroots/phase1/usr")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-j64")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("install")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    Ok(())
}

fn handle_kernel_headers() -> Result<()> {
    clone_repo("/git_sources/linux", "v6.13")?;

    let source_dir = Path::new("/phiban/sources/linux");
    env::set_current_dir(source_dir)?;

    let status = Command::new("make")
        .arg("LLVM=1")
        .arg("-j64")
        .arg("headers")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    // `find` and `cp` are how we install the linux kernel headers as the
    // initial files for our sysroot. AFAIK there is no make target that can
    // help us here.

    // TODO: Refactor into rust (maybe get fancy and reuse uutils/findutils
    let status = Command::new("find")
        .arg("usr/include")
        .arg("-type").arg("f")
        .arg("!")
        .arg("-name")
        .arg("*.h")
        .arg("-delete")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    // TODO: Refactor into rust
    let status = Command::new("cp")
        .arg("-rv")
        .arg("usr/include")
        .arg("/sysroots/phase1/usr/include")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    println!("Kernel headers install successful");
    Ok(())
}

fn handle_make() -> Result<()> {
    clone_repo("/git_sources/make", "4.4-tarball")?;

    let source_dir = Path::new("/phiban/sources/make");
    env::set_current_dir(source_dir)?;

    let status = Command::new("./configure")
        .env("PATH", "/toolchain/bin")
        .arg("--prefix=/sysroots/phase1/usr")
        .arg("--build=x86_64-phiban-linux-musl")
        .arg("--host=x86_64-phiban-linux-musl")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("-j64")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    let status = Command::new("make")
        .arg("install")
        .status()?;

    if !status.success() {
        dbg![status];
        unimplemented!{"We don't handle this failure yet!"}
    }

    Ok(())
}

fn main() -> Result<()> {
    handle_kernel_headers()?;
    handle_musl()?;
    //handle_make()?;

    std::fs::remove_dir_all("/phiban/sources")?;
    Ok(())
}

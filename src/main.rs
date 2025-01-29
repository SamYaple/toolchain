mod macros;

mod make;
mod linux_headers;
mod musl;
mod zlib;
mod openssl;
mod bzip2;
mod pkgconf;
mod libffi;
mod llvm;
mod python;
mod rust;
mod ncurses;
mod xz;
mod readline;
mod sqlite;
mod gdbm;
mod mpdecimal;

use std::env::{remove_var, set_var, set_current_dir};
use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::env;
use std::path::Path;
use anyhow::Result;

pub const TRIPLE: &'static str = "x86_64-phiban-linux-musl";

fn clone_repo(repo_url: &str, repo_tag: &str) -> Result<()> {
    let sources_dir = Path::new("/phiban/sources");
    set_current_dir(sources_dir)?;
    cmd!{"git clone --single-branch --branch {} {}", repo_tag, repo_url};

    Ok(())
}

/// Construct a minimal struture for us to use. not FHS compliant.
///   /etc
///   /bin -> usr/bin
///   /lib -> usr/lib
///   /usr/bin
///   /usr/lib
///   /toolchain -> .
fn init_fs_tree(sysroot: &str) -> Result<()> {
    create_dir(format!{"{sysroot}"})?;
    create_dir(format!{"{sysroot}/phiban"})?;
    create_dir(format!{"{sysroot}/phiban/sources"})?;
    create_dir(format!{"{sysroot}/etc"})?;
    create_dir(format!{"{sysroot}/usr"})?;
    create_dir(format!{"{sysroot}/usr/bin"})?;
    create_dir(format!{"{sysroot}/usr/lib"})?;
    symlink("usr/bin", format!{"{sysroot}/bin"})?;
    symlink("usr/lib", format!{"{sysroot}/lib"})?;
    symlink(".", format!{"{sysroot}/toolchain"})?;
    Ok(())
}

fn main() -> Result<()> {
    let sysroot = "/sysroots/phase0";
    libffi::build_and_install(sysroot)?;
    mpdecimal::build_and_install(sysroot)?;
    xz::build_and_install(sysroot)?;
    ncurses::build_and_install(sysroot)?;
    readline::build_and_install(sysroot)?;
    gdbm::build_and_install(sysroot)?;
    sqlite::build_and_install(sysroot)?;
    python::build_and_install(sysroot)?;
    panic!("at the disco");

    let sysroot = "/sysroots/phase1";
    init_fs_tree(sysroot)?;

    // Base gets your headers, libc, libunwind, libc++, but no tooling 
    linux_headers::build_and_install(sysroot)?;
    musl::build_and_install(sysroot)?;
    llvm::build_and_install_runtimes(sysroot)?;

    unsafe {
          set_var("CFLAGS", format!{"--sysroot={sysroot}"});
        set_var("CXXFLAGS", format!{"--sysroot={sysroot}"});
         set_var("LDFLAGS", format!{"--sysroot={sysroot}"});
    }

    pkgconf::build_and_install(sysroot)?;

    zlib::build_and_install(sysroot)?;
    bzip2::build_and_install(sysroot)?;
    xz::build_and_install(sysroot)?;
    libffi::build_and_install(sysroot)?;
    ncurses::build_and_install(sysroot)?;
    //perl::build_and_install(sysroot)?;

    // I did consider libressl, and everything should work with it these days,
    // but ultimately I am sticking with openssl. Though I may try to avoid the
    // perl dep by pregenerating the asm.
    openssl::build_and_install(sysroot)?;

    // Build tools
    make::build_and_install(sysroot)?;
    //cmake::build_and_install(sysroot)?;

    python::build_and_install(sysroot)?;
    //ninja::build_and_install(sysroot)?;

    unsafe {
          remove_var("CFLAGS");
        remove_var("CXXFLAGS");
         remove_var("LDFLAGS");
    }

    llvm::build_and_install(sysroot)?;
    //rust::build_and_install(sysroot)?;

    // unneeded but i dont want to commit the 20GB of source that might be here
    std::fs::remove_dir_all("/phiban/sources")?;
    Ok(())
}

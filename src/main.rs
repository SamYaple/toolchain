mod macros;

mod bzip2;
mod cmake;
mod gdbm;
mod libffi;
mod linux;
mod llvm;
mod make;
mod mpdecimal;
mod musl;
mod ncurses;
mod openssl;
mod pkgconf;
mod python;
mod readline;
mod rust;
mod shadow;
mod sqlite;
mod util_linux;
mod xz;
mod zlib;
mod bash;
mod ninja;
//mod perl;

use anyhow::Result;
use std::env::{remove_var, set_current_dir, set_var};
use std::fs::{remove_file, create_dir};
use std::os::unix::fs::symlink;
use std::path::Path;

pub const SOURCES_DIR: &'static str = "/phiban/sources";
pub const TRIPLE: &'static str = "x86_64-phiban-linux-musl";

fn clone_repo(
    dest_dir: &str,
    repo_url: &str,
    repo_tag: &str,
    restore_metadata: bool,
) -> Result<()> {
    println!("Cloning from {repo_url} into {dest_dir}");
    if !Path::new(dest_dir).exists() {
        cmd! {"git clone {} {}", repo_url, dest_dir};
    }
    set_current_dir(dest_dir)?;
    cmd! {"git reset --hard"};
    cmd! {"git clean -xdf"};
    cmd! {"git fetch origin +refs/tags/{0}:refs/tags/{0}", repo_tag};
    cmd! {"git checkout {}", repo_tag};
    cmd! {"git reset --hard"};
    cmd! {"git clean -xdf"};
    if restore_metadata {
        cmd! {"gtt restore"};
    }

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
    create_dir(format! {"{sysroot}"})?;
    create_dir(format! {"{sysroot}/etc"})?;
    create_dir(format! {"{sysroot}/usr"})?;
    create_dir(format! {"{sysroot}/usr/bin"})?;
    create_dir(format! {"{sysroot}/usr/lib"})?;
    create_dir(format! {"{sysroot}/tmp"})?;
    symlink("usr/bin", format! {"{sysroot}/bin"})?;
    symlink("usr/lib", format! {"{sysroot}/lib"})?;
    symlink(".", format! {"{sysroot}/toolchain"})?;
    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        set_var("PATH", format! {"/toolchain/usr/bin"});
        set_var("CFLAGS", "--sysroot=/toolchain");
        set_var("CXXFLAGS", "--sysroot=/toolchain");
        set_var("LDFLAGS", "--sysroot=/toolchain");
    }

    let sysroot = "/sysroots/phase1";

    // initial fs structure, headers, libc, libunwind, libc++, but no tooling
    init_fs_tree(sysroot)?;
    linux::build_and_install_headers(sysroot)?;
    musl::build_and_install(sysroot)?;
    llvm::build_and_install_runtimes(sysroot)?;

    // with all the core libraries setup, we can now build the rest of the
    // system in our new sysroot!
    unsafe {
        set_var("CFLAGS", format! {"--sysroot={sysroot} -Oz -march=native"});
        set_var("CXXFLAGS", format! {"--sysroot={sysroot} -Oz -march=native"});
        set_var("LDFLAGS", format! {"--sysroot={sysroot}"});
    }

    zlib::build_and_install(sysroot)?;
    llvm::build_and_install(sysroot)?;

    unsafe {
        set_var("PATH", format! {"{sysroot}/usr/bin:/toolchain/usr/bin"});
    }

    // NOTE: Does pkgconf need to get installed this early? probably not.
    pkgconf::build_and_install(sysroot)?;

    ncurses::build_and_install(sysroot)?;
    readline::build_and_install(sysroot)?;
    bash::build_and_install(sysroot)?;

    // zlib::build_and_install(sysroot)?;
    // bzip2::build_and_install(sysroot)?;
    // xz::build_and_install(sysroot)?;

    // openssl::build_and_install(sysroot)?;

    // libffi::build_and_install(sysroot)?;
    // gdbm::build_and_install(sysroot)?;
    // sqlite::build_and_install(sysroot)?;
    // mpdecimal::build_and_install(sysroot)?;

    // // We can now swap our linker to use all the newly built libraries and tools
    // remove_file("/toolchain")?;
    // symlink(sysroot, "/toolchain")?;
    // unsafe {
    //     set_var("PATH", format! {"/toolchain/usr/bin:/sysroots/phase0/usr/bin"});
    //     set_var("CFLAGS", "--sysroot=/toolchain");
    //     set_var("CXXFLAGS", "--sysroot=/toolchain");
    //     set_var("LDFLAGS", "--sysroot=/toolchain");
    // }

    // make::build_and_install(sysroot)?;
    // //perl::build_and_install(sysroot)?;

    // // HACK: circular dependency solved by building python twice
    // //
    // //   - util-linux wants libpython (from python)
    // //   - python wants libuuid (from util-linux)
    // python::build_and_install(sysroot)?;

    // shadow::build_and_install(sysroot)?;
    // util_linux::build_and_install(sysroot)?;
    // python::build_and_install(sysroot)?;

    // cmake::build_and_install(sysroot)?;
    // ninja::build_and_install(sysroot)?;

    // rust::build_and_install(sysroot)?;

    unsafe {
        remove_var("CFLAGS");
        remove_var("CXXFLAGS");
        remove_var("LDFLAGS");
        set_var("PATH","/toolchain/usr/bin");
    }

    Ok(())
}

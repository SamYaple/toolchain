use std::os::unix::fs::symlink;
use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/ncurses";
pub const SOURCE_URL: &'static str = "file:///git_sources/ncurses";
pub const SOURCE_TAG: &'static str = "v6.5-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    cmd! {"git apply /patches/ncurses/always_use_utf8.patch"};
    cmd! {"./configure --prefix={0}/usr --build={1} --host={1} --with-shared --without-debug --without-normal --with-cxx-shared --enable-pc-files --with-pkg-config-libdir={0}/usr/lib/pkgconfig", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    // TODO: figure out what the fuck is happening to ncurses (though it actually looks simplified
    // and better...)
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libncurses.so"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libncurses.so.6"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libncurses.so.6.5"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libtermcap.so"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libtermcap.so.6"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libtermcap.so.6.5"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libterminfo.so"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libterminfo.so.6"})?;
    symlink("libncursesw.so.6.5", format! {"{sysroot}/usr/lib/libterminfo.so.6.5"})?;
    symlink("ncursesw", format! {"{sysroot}/usr/include/ncurses"})?;
    symlink("ncursesw", format! {"{sysroot}/usr/include/terminfo"})?;
    symlink("ncursesw", format! {"{sysroot}/usr/include/termcap"})?;

    Ok(())
}

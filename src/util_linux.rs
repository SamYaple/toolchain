use crate::clone_repo;
use crate::cmd;
use anyhow::Result;

pub const SOURCE_DIR: &'static str = "/phiban/sources/util-linux";
pub const SOURCE_URL: &'static str = "file:///git_sources/util-linux";
pub const SOURCE_TAG: &'static str = "v2.40.4-tarball+gtt";
pub const RESTORE_METADATA: bool = true;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo(SOURCE_DIR, SOURCE_URL, SOURCE_TAG, RESTORE_METADATA)?;

    //--disable-chfn-chsh --disable-login --disable-nologin --disable-su --disable-setpriv --disable-runuser --disable-pylibmount --disable-liblastlog2 --disable-static --without-python ADJTIME_PATH=/var/lib/hwclock/adjtime

    //cmd!{"cp -av /etc/passwd /etc/groups {}/etc", sysroot};
    cmd! {"./configure --build={1} --host={1} --prefix={0}/usr", sysroot, crate::TRIPLE};
    cmd! {"make -j64"};
    cmd! {"make install"};

    Ok(())
}

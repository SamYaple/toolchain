use std::os::unix::fs::symlink;
use std::env;
use crate::cmd;
use std::path::Path;
use anyhow::Result;
use crate::clone_repo;

pub fn build_and_install(sysroot: &str) -> Result<()> {
    clone_repo("/git_sources/util-linux", "v2.40.4-tarball+gtt")?;
    let source_dir = Path::new("/phiban/sources/util-linux");
    env::set_current_dir(source_dir)?;
    cmd!{"gtt restore"};

//--disable-chfn-chsh --disable-login --disable-nologin --disable-su --disable-setpriv --disable-runuser --disable-pylibmount --disable-liblastlog2 --disable-static --without-python ADJTIME_PATH=/var/lib/hwclock/adjtime

    //cmd!{"cp -av /etc/passwd /etc/groups {}/etc", sysroot};
    cmd!{"./configure --build={1} --host={1} --prefix={0}/usr", sysroot, crate::TRIPLE};
    cmd!{"make -j64"};
    cmd!{"make install"};

    Ok(())
}

mod macros;

mod make;
mod linux_headers;
mod musl;
mod zlib;
mod openssl;
mod bzip2;

use std::env;
use std::path::Path;
use anyhow::Result;

pub const TRIPLE: &'static str = "x86_64-phiban-linux-musl";

fn clone_repo(repo_url: &str, repo_tag: &str) -> Result<()> {
    let sources_dir = Path::new("/phiban/sources");
    env::set_current_dir(sources_dir)?;
    cmd!{"git clone --single-branch --branch {} {}", repo_tag, repo_url};
    Ok(())
}

fn main() -> Result<()> {
    let sysroot = "/sysroots/phase1";
    //linux_headers::build_and_install(sysroot)?;
    musl::build_and_install(sysroot)?;
    zlib::build_and_install(sysroot)?;
    bzip2::build_and_install(sysroot)?;
    openssl::build_and_install(sysroot)?;
    //make::build_and_install(sysroot)?;

    std::fs::remove_dir_all("/phiban/sources")?;
    Ok(())
}

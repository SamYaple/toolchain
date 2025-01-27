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

fn clone_repo(repo_url: &str, repo_tag: &str) -> Result<()> {
    let sources_dir = Path::new("/phiban/sources");
    env::set_current_dir(sources_dir)?;
    cmd!{"git clone --single-branch --branch {} {}", repo_tag, repo_url};
    Ok(())
}

fn main() -> Result<()> {
    //linux_headers::build_and_install()?;
    musl::build_and_install()?;
    zlib::build_and_install()?;
    bzip2::build_and_install()?;
    openssl::build_and_install()?;
    //make::build_and_install()?;

    std::fs::remove_dir_all("/phiban/sources")?;
    Ok(())
}

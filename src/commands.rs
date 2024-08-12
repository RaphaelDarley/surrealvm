use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    os::unix::{self, fs::symlink},
    path::PathBuf,
};

use flate2::read::GzDecoder;
use reqwest::blocking::get;
use semver::Version;
use tar::Archive;

use crate::throw;
use crate::{CPU, OSS};

fn get_home_svm() -> anyhow::Result<(PathBuf, PathBuf)> {
    let Some(home_dir) = homedir::my_home()? else {
        throw!("Error finding home directory")
    };
    let svm_dir = home_dir.join(".surrealvm");
    Ok((home_dir, svm_dir))
}

pub fn setup() -> anyhow::Result<()> {
    let (home_dir, svm_dir) = get_home_svm()?;
    if svm_dir.exists() {
        throw!("directory already exists, try `surrealvm clean` then `surrealvm setup` again");
    }

    fs::create_dir(&svm_dir)?;

    let bashrc_path = home_dir.join(".bashrc");
    let zprofile_path = home_dir.join(".zprofile");
    if bashrc_path.exists() {
        add_path(bashrc_path)?;
        println!(".zprofile updated");
    } else if zprofile_path.exists() {
        add_path(zprofile_path)?;
        println!(".zprofile updated");
    } else {
        eprintln!("didn't add to path, unsupported shell");
    }

    let exe_path = env::current_exe()?;
    symlink(exe_path, svm_dir.join("surreal")).unwrap();

    Ok(())
}

static CONFIG_MARKER: &str = "# added by SurrealVM";
static CONFIG_PATH: &str = "PATH=~/.surrealvm:$PATH";

fn add_path(path: PathBuf) -> anyhow::Result<()> {
    let path_str = format!("{CONFIG_MARKER}\n{CONFIG_PATH}\n");
    let mut rc_f = fs::OpenOptions::new().append(true).open(path)?;
    rc_f.write_all(path_str.as_bytes())?;
    Ok(())
}

fn remove_path(path: PathBuf) -> anyhow::Result<()> {
    let reader = BufReader::new(File::open(&path)?);
    let out: Vec<String> = reader
        .lines()
        .filter_map(|r| r.ok())
        .filter(|l| l != CONFIG_MARKER && l != CONFIG_PATH)
        .collect();

    let mut writer = OpenOptions::new().write(true).truncate(true).open(path)?;
    for line in out {
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}

pub fn clean() -> anyhow::Result<()> {
    let (home_dir, svm_dir) = get_home_svm()?;
    if !svm_dir.exists() {
        throw!("directory doesn't exist, can't clean what isn't there!");
    }

    fs::remove_dir_all(&svm_dir)?;
    let bashrc_path = home_dir.join(".bashrc");
    let zprofile_path = home_dir.join(".zprofile");
    if bashrc_path.exists() {
        remove_path(bashrc_path)?;
        println!(".zprofile updated");
    } else if zprofile_path.exists() {
        remove_path(zprofile_path)?;
        println!(".zprofile updated");
    } else {
        eprintln!("didn't remove from path, unsupported shell");
    }

    Ok(())
}

fn get_latest() -> anyhow::Result<Version> {
    let ver_res = get("https://download.surrealdb.com/latest.txt")?;
    parse_surreal_version(&ver_res.text()?)
}

fn parse_surreal_version(ver_str: &str) -> anyhow::Result<Version> {
    Ok(Version::parse(&ver_str.trim().trim_start_matches('v'))?)
}

pub fn install(ver: String) -> anyhow::Result<()> {
    let (_home_dir, svm_dir) = get_home_svm()?;
    if !svm_dir.exists() {
        throw!("svm directory doesn't exist try: surrealvm setup");
    }

    let ver = match ver.as_str() {
        "latest" => get_latest()?,
        v => parse_surreal_version(v)?,
    };

    let sver = format!("v{ver}");

    let bin_name = format!("surreal-{ver}");
    let sbin_name = format!("surreal-{sver}");

    let tgz_path = svm_dir.join(format!("{sbin_name}.tgz"));
    if tgz_path.exists() {
        throw!("specified version already installed (--force support wip)");
    }

    let url = format!("https://download.surrealdb.com/{sver}/surreal-{sver}.{OSS}-{CPU}.tgz");
    let res = get(url)?;

    let mut tmp_file = File::create_new(&tgz_path)?;

    io::copy(&mut res.bytes()?.as_ref(), &mut tmp_file)?;
    drop(tmp_file);

    let tgz_file = File::open(&tgz_path)?;

    let tar = GzDecoder::new(tgz_file);
    let mut output = Archive::new(tar);

    fs::remove_file(tgz_path)?;

    let bin_out_dir = svm_dir.join(format!("tmp_{sbin_name}"));

    output.unpack(&bin_out_dir)?;
    fs::rename(bin_out_dir.join("surreal"), svm_dir.join(&sbin_name))?;
    fs::remove_dir(bin_out_dir)?;

    symlink(svm_dir.join(sbin_name), svm_dir.join(bin_name))?;

    // TODO; allow --use command to use here

    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    Ok(())
}

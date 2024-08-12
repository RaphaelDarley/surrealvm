use std::{
    env,
    fs::{self, read_link, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
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
    let svm_path = svm_dir.join("surrealvm");
    fs::copy(&exe_path, &svm_path)?;
    let none_path = svm_dir.join("surreal-none");
    symlink(svm_path, &none_path)?;
    symlink(none_path, svm_dir.join("surreal"))?;

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

fn get_named(name: impl AsRef<str>) -> anyhow::Result<Version> {
    let ver_res = get(format!(
        "https://download.surrealdb.com/{}.txt",
        name.as_ref()
    ))?;
    parse_surreal_version(&ver_res.text()?)
}

fn parse_surreal_version(ver_str: impl AsRef<str>) -> anyhow::Result<Version> {
    Ok(Version::parse(
        &ver_str.as_ref().trim().trim_start_matches('v'),
    )?)
}

enum VerSelection {
    Special(SpecialVer),
    Custom(Version),
}
enum SpecialVer {
    None,
    Latest,
    Beta,
    Alpha,
    Nightly,
}
impl VerSelection {
    fn parse(value: impl AsRef<str>) -> anyhow::Result<VerSelection> {
        Ok(match value.as_ref() {
            "none" => VerSelection::Special(SpecialVer::None),
            "latest" => VerSelection::Special(SpecialVer::Latest),
            "beta" => VerSelection::Special(SpecialVer::Beta),
            "alpha" => VerSelection::Special(SpecialVer::Alpha),
            "nightly" => VerSelection::Special(SpecialVer::Nightly),
            v => {
                let tmp = parse_surreal_version(v)?;
                VerSelection::Custom(tmp)
            }
        })
    }
    fn to_sname(&self) -> String {
        match self {
            VerSelection::Special(s) => s.to_name().to_string(),
            VerSelection::Custom(v) => format!("v{v}"),
        }
    }
    fn to_special(&self) -> Option<&'static str> {
        match self {
            VerSelection::Special(s) => Some(s.to_name()),
            VerSelection::Custom(_) => None,
        }
    }
    fn to_version(&self) -> anyhow::Result<Version> {
        Ok(match self {
            VerSelection::Custom(v) => v.to_owned(),
            VerSelection::Special(s) => get_named(s.to_name())?,
        })
    }
}
impl SpecialVer {
    fn to_name(&self) -> &'static str {
        match self {
            SpecialVer::None => "none",
            SpecialVer::Latest => "latest",
            SpecialVer::Beta => "leta",
            SpecialVer::Alpha => "alpha",
            SpecialVer::Nightly => "nightly",
        }
    }
}

fn relink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> anyhow::Result<()> {
    if link.as_ref().exists() {
        fs::remove_file(&link)?;
    }
    symlink(original, link)?;
    Ok(())
}

pub fn install(ver: String) -> anyhow::Result<()> {
    let (_home_dir, svm_dir) = get_home_svm()?;
    if !svm_dir.exists() {
        throw!("svm directory doesn't exist try: surrealvm setup");
    }

    let ver_sel = VerSelection::parse(&ver)?;

    let ver = ver_sel.to_version()?;

    let sver = format!("v{ver}");

    let bin_name = format!("surreal-{ver}");
    let bin_path = svm_dir.join(&bin_name);
    let sbin_name = format!("surreal-{sver}");
    let sbin_path = svm_dir.join(&sbin_name);

    if sbin_path.exists() {
        throw!("specified version already installed (--force support wip)");
    }

    let tgz_path = svm_dir.join(format!("{sbin_name}.tgz"));

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
    fs::rename(bin_out_dir.join("surreal"), &sbin_path)?;
    fs::remove_dir(bin_out_dir)?;

    symlink(&sbin_path, &bin_path)?;

    // TODO; allow --use command to use here

    if let Some(n) = ver_sel.to_special() {
        relink(
            svm_dir.join(sbin_name),
            svm_dir.join(format!("surreal-{n}")),
        )?;
    }

    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    let (_home_dir, svm_dir) = get_home_svm()?;
    if !svm_dir.exists() {
        throw!("svm directory doesn't exist try: surrealvm setup");
    }

    let mut special_acc = Vec::new();
    let mut ver_acc = Vec::new();
    for entry in fs::read_dir(&svm_dir)? {
        let Ok(entry) = entry else {
            continue;
        };

        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(ver) = name.strip_prefix("surreal-v") {
            ver_acc.push(format!("v{ver}"));
        } else if let Some(name) = VerSelection::parse(name.trim_start_matches("surreal-"))
            .ok()
            .map(|vs| vs.to_special())
            .flatten()
        {
            special_acc.push(name.to_string());
        }
    }

    let selected_path = read_link(svm_dir.join("surreal"))?;
    let ver_sel = if let Some(name) = selected_path.file_name() {
        if let Ok(vs) = VerSelection::parse(name.to_string_lossy().trim_start_matches("surreal-")) {
            Some(vs.to_sname())
        } else {
            None
        }
    } else {
        None
    };
    let ver_sel = match ver_sel {
        Some(v) => v,
        None => {
            eprintln!("warning: failed to parse selected version");
            "none".to_string()
        }
    };

    for ver in special_acc.iter().chain(ver_acc.iter()) {
        if ver == &ver_sel {
            println!("{ver} *");
        } else {
            println!("{ver}");
        }
    }

    Ok(())
}

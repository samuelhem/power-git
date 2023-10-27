use std::{fmt::Display, fs, io::Write, path::PathBuf};

use anyhow::Context;
use clap::Parser;

pub mod commands;

#[derive(Parser)]
struct Cli {
    /// Available Options are: init, add, commit, push, pull, status, log, branch, checkout, merge, rebase, reset, tag, fetch, remote, clone, rm, mv
    keyword: Command,
    args: Vec<String>,
    #[clap(short, long)]
    plattform: Plattform,
}

#[derive(Clone)]
enum Command {
    SET,
    INIT,
    UNKNOWN,
    SHOW,
}

#[derive(Clone, Debug)]
pub enum Plattform {
    GITHUB,
    GITLAB,
    BITBUCKET,
    UNSUPPORTED,
}

impl Display for Plattform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plattform::GITHUB => write!(f, "github"),
            Plattform::GITLAB => write!(f, "gitlab"),
            Plattform::BITBUCKET => write!(f, "bitbucket"),
            Plattform::UNSUPPORTED => write!(f, "unsupported"),
        }
    }
}

impl From<&str> for Plattform {
    fn from(s: &str) -> Plattform {
        match s.to_lowercase().as_str() {
            "github" => Plattform::GITHUB,
            "gitlab" => Plattform::GITLAB,
            "bitbucket" => Plattform::BITBUCKET,
            _ => Plattform::UNSUPPORTED,
        }
    }
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "set" => Command::SET,
            "init" => Command::INIT,
            "show" => Command::SHOW,
            _ => Command::UNKNOWN,
        }
    }
}

const CONFIG_FILE: &'static str = "config.json";
const CONFIG_DIR: &'static str = "./.config/power_git/";

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let cfg = init_cfg(None)?;

    match args.keyword {
        Command::SET => {
            commands::SetCommand::new(cfg, args.args, args.plattform)?.parse();
        }
        Command::INIT => {
            commands::InitCommand::new(cfg, args.args, args.plattform)?.parse();
        }
        Command::UNKNOWN => {
            println!("Unknown");
        }
        Command::SHOW => {
            commands::ShowCommand::new(cfg, args.args)?.show();
        }
    }
    Ok(())
}

fn create_config_path(file_name: Option<&str>) -> PathBuf {
    let mut home = dirs::home_dir().unwrap();

    home.push(PathBuf::from(CONFIG_DIR));
    if let Some(file_name) = file_name {
        home.push(PathBuf::from(file_name));
    }
    return home.to_path_buf();
}

fn init_cfg(existing_config: Option<String>) -> anyhow::Result<fs::File> {
    let dir_path = create_config_path(None);
    let file_path = create_config_path(Some(CONFIG_FILE));

    let fh_read = read_config_file(&file_path);
    if let Err(_) = fh_read {
        let mut cfg_dir = fs::DirBuilder::new();
        cfg_dir
            .recursive(true)
            .create(&dir_path)
            .with_context(|| format!("Failed to create config dir: {:?}", &dir_path))?;

        let fh_create = &mut create_config_file(&file_path)?;

        let cfg = serde_json::to_string_pretty(&serde_json::json!([
            {
                "name": "github",
                "cfg": {
                    "url": "",
                    "token": "",
                    "default": false
                }
            },
            {
                "name": "gitlab",
                "cfg": {
                    "url": "",
                    "token": "",
                    "default": false
                }
            },
            {
                "name": "bitbucket",
                "cfg": {
                    "url": "",
                    "token": "",
                    "default": false
                }
            }
        ]))?;
        write_to_config_file(fh_create, cfg)?;
    }

    if let Some(cfg) = existing_config {
        let fh_create = &mut create_config_file(&file_path)?;
        write_to_config_file(fh_create, cfg)?;
    }

    return fh_read;
}

fn create_config_file(file_path: &PathBuf) -> anyhow::Result<fs::File> {
    return fs::File::create(&file_path).with_context(|| {
        format!(
            "Failed to create config file: {}{}",
            CONFIG_DIR, CONFIG_FILE,
        )
    });
}

fn write_to_config_file(file: &mut fs::File, cfg: String) -> anyhow::Result<()> {
    return file
        .write_all(cfg.as_bytes())
        .with_context(|| format!("Failed to write config file: {}{}", CONFIG_DIR, CONFIG_FILE));
}

fn read_config_file(file_path: &PathBuf) -> anyhow::Result<fs::File> {
    return fs::File::open(&file_path)
        .with_context(|| format!("Failed to open config file: {}{}", CONFIG_DIR, CONFIG_FILE));
}

fn get_config(file: &fs::File) -> Vec<commands::GitRepo> {
    return serde_json::from_reader(file).unwrap();
}

pub fn get_config_for_plattform(file: &fs::File, plattform: &Plattform) -> Option<commands::GitRepo> {
    let configs: Vec<commands::GitRepo> = get_config(file);
    return configs
        .iter()
        .find(|x| x.name == plattform.to_string())
        .cloned();
}

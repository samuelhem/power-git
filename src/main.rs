use std::{fs, io::Write, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
struct Cli {
    /// Available Options are: init, add, commit, push, pull, status, log, branch, checkout, merge, rebase, reset, tag, fetch, remote, clone, rm, mv
    keyword: Command,
    args: Vec<String>,
}

#[derive(Clone, Debug)]
enum Plattform {
    GITHUB,
    GITLAB,
    BITBUCKET,
    UNSUPPORTED,
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

#[derive(Clone)]
enum Command {
    SET,
    INIT,
    UNKNOWN,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "set" => Command::SET,
            "init" => Command::INIT,
            _ => Command::UNKNOWN,
        }
    }
}

const CONFIG_FILE: &'static str = "config.json";
const CONFIG_DIR: &'static str = "./.config/power_git/";

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let cfg = init_cfg()?;

    match args.keyword {
        Command::SET => {
            set_command(cfg, args.args);
        }
        Command::INIT => {
            println!("Init");
        }
        Command::UNKNOWN => {
            println!("Unknown");
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

fn init_cfg() -> anyhow::Result<fs::File> {
    let dir_path = create_config_path(None);
    let file_path = create_config_path(Some(CONFIG_FILE));

    let cfg_file = fs::File::open(&file_path)
        .with_context(|| format!("Failed to open config file: {}{}", CONFIG_DIR, CONFIG_FILE));

    if let Err(_) = cfg_file {
        let mut cfg_dir = fs::DirBuilder::new();
        cfg_dir
            .recursive(true)
            .create(&dir_path)
            .with_context(|| format!("Failed to create config dir: {:?}", &dir_path))?;

        let mut cfg_file = fs::File::create(&file_path)
            .with_context(|| {
                format!(
                    "Failed to create config file: {}{}",
                    CONFIG_DIR, CONFIG_FILE
                )
            })?;


        let cfg = serde_json::to_string_pretty(&serde_json::json!({
            "github": {
                "user": "",
                "token": "",
                "default": false
            },
            "gitlab": {
                "user": "",
                "token": "",
                "default": false
            },
            "bitbucket": {
                "user": "",
                "token": "",
                "default": false
            }
        }))?;
        cfg_file.write_all(cfg.as_bytes()).with_context(|| {
            format!("Failed to write config file: {}{}", CONFIG_DIR, CONFIG_FILE)
        })?;
    }

    cfg_file
}

#[derive(Deserialize, Serialize)]
struct GitRepoCfg {
    user: String,
    token: String,
    default: bool,
}

fn set_command(cfg: fs::File, args: Vec<String>) {
    let plattform = Plattform::from(args[0].as_str());
    if let Plattform::UNSUPPORTED = plattform {
        eprintln!(
            "Unsupported Plattform please use one of the following: github, gitlab, bitbucket"
        );
        return;
    }

    let cfg: Vec<GitRepoCfg> = serde_json::from_reader(cfg).unwrap();

}

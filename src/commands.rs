use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{fs, rc::Rc};

use crate::{get_config, get_config_for_plattform, init_cfg, Plattform, git};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitRepo {
    pub name: String,
    pub cfg: GitRepoCfg,
}

impl GitRepo {
    pub fn print(&self) {
        println!("{}: {} -- {}", self.name, self.cfg.url, self.cfg.token);
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitRepoCfg {
    pub url: String,
    pub token: String,
    default: bool,
}


enum GitRepoCfgArgs {
    URL,
    TOKEN,
    DEFAULT,
    ERROR,
}

impl From<&str> for GitRepoCfgArgs {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "url" => GitRepoCfgArgs::URL,
            "token" => GitRepoCfgArgs::TOKEN,
            "default" => GitRepoCfgArgs::DEFAULT,
            _ => GitRepoCfgArgs::ERROR,
        }
    }
}

pub struct SetCommand {
    cfg_file: fs::File,
    args: Vec<String>,
    plattform: Plattform,
}

impl SetCommand {
    pub fn new(
        cfg_file: fs::File,
        args: Vec<String>,
        plattform: Plattform,
    ) -> anyhow::Result<Self> {
        if args.len() < 2 {
            return Err(anyhow!("Please provide all args"));
        }

        if let Plattform::UNSUPPORTED = plattform {
            return Err(anyhow!(
                "Unsupported Plattform please use one of the following: github, gitlab, bitbucket"
            ));
        }
        Ok(SetCommand {
            cfg_file,
            args,
            plattform,
        })
    }

    pub fn parse(&self) {
        let set_args = GitRepoCfgArgs::from(self.args[0].as_str());
        match set_args {
            GitRepoCfgArgs::URL => {
                self.set_url();
            }
            GitRepoCfgArgs::TOKEN => {
                self.set_token();
            }
            GitRepoCfgArgs::DEFAULT => {
                self.set_default();
            }
            GitRepoCfgArgs::ERROR => {
                eprintln!("Please provide a valid argument");
            }
        }
    }

    fn set_url(&self) {
        let configs: Vec<GitRepo> = get_config(&self.cfg_file);
        if let Some(cfg) = configs
            .iter()
            .find(|x| x.name == self.plattform.to_string())
        {
            let idx = configs
                .iter()
                .position(|x| x.name == self.plattform.to_string())
                .unwrap();

            let mut new_cfg = cfg.clone();
            new_cfg.cfg.url = self.args[1].clone();

            let mut new_configs = configs.clone();
            new_configs[idx] = new_cfg;

            let cfg = serde_json::to_string_pretty(&new_configs).unwrap();
            if let Err(err) = init_cfg(Some(cfg)) {
                eprintln!("Failed to set url: {}", err);
                return;
            }
            println!("Url set for {}", self.plattform.to_string());
        }
    }

    fn set_token(&self) {
        let configs: Vec<GitRepo> = get_config(&self.cfg_file);
        if let Some(cfg) = configs
            .iter()
            .find(|x| x.name == self.plattform.to_string())
        {
            let idx = configs
                .iter()
                .position(|x| x.name == self.plattform.to_string())
                .unwrap();

            let mut new_cfg = cfg.clone();
            new_cfg.cfg.token = self.args[1].clone();

            let mut new_configs = configs.clone();
            new_configs[idx] = new_cfg;

            let cfg = serde_json::to_string_pretty(&new_configs).unwrap();
            if let Err(err) = init_cfg(Some(cfg)) {
                eprintln!("Failed to set token: {}", err);
                return;
            }
            println!("Token set for {}", self.plattform.to_string());
        }
    }

    fn set_default(&self) {
        todo!()
    }
}

pub enum ShowArgs {
    CONFIG,
    ERROR,
}

impl From<&str> for ShowArgs {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "config" => ShowArgs::CONFIG,
            _ => ShowArgs::ERROR,
        }
    }
}

pub struct ShowCommand {
    cfg_file: fs::File,
    args: Vec<String>,
}

impl ShowCommand {
    pub fn new(cfg_file: fs::File, args: Vec<String>) -> anyhow::Result<Self> {
        if args.len() != 1 {
            return Err(anyhow!("Please provide a show argument"));
        }

        Ok(ShowCommand { cfg_file, args })
    }

    pub fn show(&self) {
        match self.args[0].as_str().into() {
            ShowArgs::CONFIG => {
                let configs: Vec<GitRepo> = get_config(&self.cfg_file);
                for cfg in configs {
                    cfg.print();
                }
            }
            ShowArgs::ERROR => {
                eprintln!("Please provide a valid show argument");
            }
        }
    }
}

pub struct InitCommand<'a> {
    cfg_file: fs::File,
    args: Vec<String>,
    plattform: Plattform,
    git_client: Rc<dyn git::Initializable<'a>>,
}

impl<'a> InitCommand<'a> {
    pub fn new(
        cfg_file: fs::File,
        args: Vec<String>,
        plattform: Plattform,
        git_client: Rc<dyn git::Initializable<'a>>  
    ) -> anyhow::Result<Self> {
        Ok(InitCommand {
            cfg_file,
            args,
            plattform,
            git_client,
        })
    }

    pub fn parse (&self) {
        let repo_name: Option<String>;
        if self.args.len() == 1 {
            repo_name = Some(self.args[0].clone());
        } else {
            repo_name = None;
        }

        if let Plattform::UNSUPPORTED = self.plattform {
            eprintln!(
                "Unsupported Plattform please use one of the following: github, gitlab, bitbucket"
            );
            return;
        }

        // Get Config for Current Plattform
        let config = get_config_for_plattform(&self.cfg_file, &self.plattform).unwrap();

        if let None = repo_name {
            println!("Intializing Repo in current directory...");
            let output = std::process::Command::new("git")
                .args(["init"])
                .output()
                .expect("failed to execute process");
            println!("{}", output.status);

            println!("Intializing Remote Repo with name {}...", repo_name.as_ref().unwrap());
            let current_dir = std::env::current_dir().unwrap();
            self.git_client.init_repo(String::from(current_dir.to_str().unwrap()));
        }

    }
}

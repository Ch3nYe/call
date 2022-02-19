#![feature(command_access)]
#![allow(dead_code)]
#[macro_use] // allow use macros in crate serde_derive
extern crate serde_derive;

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, Arg};
use std::{env, fs, process};
use yaml_rust::YamlLoader;

use crate::config::{CallConfig, CallSystemConfig};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use log::log;

#[macro_use]
mod call_macro;
mod cmd;
mod config;

fn root_path() -> Result<PathBuf> {
	let mut path = dirs::home_dir().unwrap();
	path.push(".call");
	Ok(path)
}

pub fn create_file(path: &Path, content: &str) -> Result<()> {
	if let Some(p) = path.parent() {
		create_dir_all(p)?;
	}
	let mut file = File::create(&path)?;
	file.write_all(content.as_bytes())?;
	Ok(())
}

pub fn config_file(call_config_root: &PathBuf) -> Result<(PathBuf, PathBuf)> {
	match call_config_root.exists() {
		true => {
			let settings = CallSystemConfig::build(call_config_root).unwrap();
			let template_file = root_path()?.join(settings.template);

			let call_file = env::current_dir()?.join(settings.call_config_path).join("Call.yml");
			Ok((template_file, call_file))
		}
		false => {
			let template_file = root_path()?.join("template.toml");
			let call_file = env::current_dir()?.join("Call.yml");
			Ok((template_file, call_file))
		}
	}
}

fn run() -> Result<bool> {
	let matches = App::new("call") // help message from Cargo.toml:[package]
		.version(crate_version!())
		.author(crate_authors!())
		.about(crate_description!())
		.arg(Arg::with_name("args").required(false).help("command's args, multipule arg should in \"\"").empty_values(false)) // when you use short() long() ,this argument must use -- or - explictly specify, alternatively specify without - or --
		// 是否允许显式指定空值
		// empty_values true[default]: must specify "" or '', command otherwise won't execute.
		.arg(Arg::from_usage("-c --command [command] 'use this command, default use the command in Call.yml'"))// [explicit name] [short] [long] [value names] [help string] name use <>:required or []:optional
		.get_matches();
	// whether or not explicitly specify command:
	let mut runner:String = "".to_string();
	if let Some(cmd) = matches.value_of("command") {
		println!("[+]parse explicit command={}",cmd);
		runner = cmd.parse()?;
	}
	// parse config and run
	let command = matches.value_of("args").unwrap_or("");
	match command {
		_ if command == "i" => cmd::init()?, // ? is for anyhow to handle exceptions,
		_ => {
			let call_config_root = root_path()?.join("config.toml");
			let (_template_file, call_file) = config_file(&call_config_root)?;
			let s = fs::read_to_string(call_file.as_path())?;
			let yml = YamlLoader::load_from_str(s.as_ref())?;
			let mut config = CallConfig::build(yml[0].to_owned())?;
			config.runner = if !runner.is_empty() {runner} else {config.runner};
			cmd::runner(command, &config)?
		}
	}

	Ok(true)
}

fn main() {
	let result = run();
	match result {
		Err(error) => {
			log::error!("Call Error: {}", error);
			process::exit(1);
		}
		Ok(false) => {
			process::exit(1);
		}
		Ok(true) => {
			process::exit(0);
		}
	}
}


// #[cfg(test)]
// mod tests {
// 	use super::*;
//
// 	#[test]
// 	fn internal()->Result<bool> {
// 		let call_config_root = root_path()?;
// 		println!("[-]call_config_root: {}",call_config_root.to_str().unwrap());
// 		Ok(true)
// 	}
// }

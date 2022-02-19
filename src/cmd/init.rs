use console::style;
use indicatif::HumanDuration;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

use anyhow::Result;
use std::io::Write;
use std::time::Instant;
use std::{env, fs};

use crate::config::{CallSystemConfig, INIT_CONFIG, LOOKING_GLASS, SPARKLE};
use crate::{config_file, create_file, root_path};

pub fn init() -> Result<()> {
	let call_config_root = root_path()?.join("config.toml");
	let (template_file, call_file) = config_file(&call_config_root)?;
	let call_file_parent = call_file.parent().unwrap();

	if !call_file_parent.exists() {
		create_dir_all(call_file_parent)?;
	}
	// Time count that is nice to use.
	let started = Instant::now();

	if template_file.exists() {
		println!(
			"{} {} Copy Call.yml file from template.toml",
			style(format!("[1/{}]", 1)).bold().dim(),
			LOOKING_GLASS
		);
		// File Copy Operation
		fs::copy(&template_file, &call_file)?;
	} else {
		println!(
			"{} {} Create Call.yml file...",
			style(format!("[1/{}]", 1)).bold().dim(),
			LOOKING_GLASS
		);

		let config = INIT_CONFIG.trim_start();

		create_file(&call_file, &config)?;
	}

	println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
	Ok(())
}

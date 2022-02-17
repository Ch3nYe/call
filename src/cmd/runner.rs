use anyhow::Result;
use cmd_lib::*;
use ignore::WalkBuilder;
use log::*;
use std::process::{exit, Command, Stdio};

use crate::config::{CallConfig, ServerValue, PAPER, TRUCK};
use console::style;

pub fn runner(command: &str, config: &CallConfig) -> Result<()> {
	let mut include_list: Vec<String> = Vec::new();
	// get all path and file in current path "." , depth=1
	for result in WalkBuilder::new(".").max_depth(Some(1)).build() {
		match result {
			Ok(entry) => {
				let s = entry.path().to_str().unwrap();
				if config.mapping.exclude.iter().any(|v| v != s && "." != s) {
					include_list.append(&mut vec!["--include".to_string(), s.to_string()])
				}
			}
			Err(err) => println!("Call ERROR: {}", err),
		}
	}
	// get active ways in Call.yml,
	for (_key, server_list) in config.active.iter() {
		let dest = config.mapping.dest.as_str();
		let src = config.mapping.src.as_str();

		for server in server_list {
			if let ServerValue::Openssh { // ssh connect with
				host,
				port,
				authentication_type: _,
				username,
			} = server
			{
				for host_ip in host {
					println!(
						"{} {} server({})",
						style(format!("[{}]", "syncing...")).bold().dim(),
						TRUCK,
						host_ip,
					);
					let mut rsync = Command::new(format!("rsync -aq -zz --delete --chmod=755 ssh -p{} --rsync-path=\"mkdir -p {} && rsync\" {} {}@{}:{}"
														 ,port,dest,src,username,host_ip,dest));
					// println!("[-]cmd: {}",rsync.get_program().to_str().unwrap());
					rsync.spawn().expect("[!]rsync execute error");

					println!(
						"{} {} server({}) run: {} {}",
						style(format!("[{}]", "running...")).bold().dim(),
						PAPER,
						host_ip,
						config.runner.as_str(),
						command
					);

					openssh_run(host_ip, port, username, dest, config.runner.as_str(), command);
				}
			};
			if let ServerValue::Password { // sshpass connect
				host,
				port,
				authentication_type: _,
				username,
				password,
			} = server
			{
				for host_ip in host {
					println!(
						"{} {} server({})",
						style(format!("[{}]", "syncing...")).bold().dim(),
						TRUCK,
						host_ip,
					);
					run_cmd!(sshpass -p $password rsync -aq -zz  -e "ssh -p $port" --delete --chmod=755 --info=progress2 --rsync-path="mkdir -p $dest && rsync" $src $username@$host_ip:$dest)?;
					println!(
						"{} {} server({}) run: {} {}",
						style(format!("[{}]", "running...")).bold().dim(),
						PAPER,
						host_ip,
						config.runner.as_str(),
						command
					);
					password_run(host_ip, port, username, password, dest, config.runner.as_str(), command);
				}
			}
			if let ServerValue::Keypair {
				host,
				port,
				authentication_type: _,
				username,
				private_key_file,
				pass_phrase: _,
			} = server
			{
				for host_ip in host {
					println!(
						"{} {} server({})",
						style(format!("[{}]", "syncing...")).bold().dim(),
						TRUCK,
						host_ip,
					);
					let ssh_command = format!("ssh -p{} -i {}", port.to_owned(), private_key_file.trim());
					run_cmd!(rsync -aq -zz  -e "$ssh_command" --delete --chmod=755 --exclude-from=".gitignore" --info=progress2 --rsync-path="mkdir -p $dest && rsync" . $username@$host_ip:$dest)?;
					println!(
						"{} {} server({}) run:  {} {}",
						style(format!("[{}]", "running...")).bold().dim(),
						PAPER,
						host_ip,
						config.runner.as_str(),
						command
					);
					keypair_run(
						host_ip,
						port,
						username,
						private_key_file,
						dest,
						config.runner.as_str(),
						command,
					);
				}
			}
		}
	}

	Ok(())
}

fn openssh_run(host: &str, port: &i64, username: &str, dest_path: &str, runner: &str, command: &str) {
	let mut ssh = Command::new("ssh");

	let run_server = format!("{}@{}", username, host);
	let run_command = format!("cd {} && {} {}", dest_path, runner, command);

	let output = ssh
		.arg("-t")
		.arg(run_server.as_str())
		.arg(format!("-p {}", port))
		.arg(run_command.as_str())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.output()
		.unwrap_or_else(|e| {
			error!("call error: {}", e);
			exit(-5);
		});

	if !output.status.success() {
		eprintln!("Call Error: {:?}", output.stderr);
		exit(output.status.code().unwrap_or(1))
	}
}

fn password_run(host: &str, port: &i64, username: &str, password: &str, dest_path: &str, runner: &str, command: &str) {
	let mut sshpass = Command::new("sshpass");
	let run_server = format!("{}@{}", username, host);
	let run_command = format!("cd {} && {} {}", dest_path, runner, command);

	let output = sshpass
		.args(&[
			"-p",
			password,
			"ssh",
			"-p",
			format!("{}", port).as_str(),
			run_server.as_str(),
			run_command.as_str(),
		])
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.output()
		.unwrap_or_else(|e| {
			error!("Call ERROR: {}", e);
			exit(-5);
		});
	if !output.status.success() {
		eprintln!("Call Error: {:?}", output.stderr);
		exit(output.status.code().unwrap_or(1))
	}
}

fn password_rsync(host: &str, port: &i64, username: &str, password: &str, dest_path: &str) {
	let mut sshpass = Command::new("sshpass");
	let ssh_command = format!("ssh -p {}", port);

	let run_server = format!("{}@{}:{}", username, host, dest_path,);

	let output = sshpass
		.args(&[
			"-p",
			password,
			"rsync",
			"-aq",
			"-zz",
			"-e",
			ssh_command.as_str(),
			"--delete",
			"--chmod=755",
			"--exclude-from=.gitignore",
			"--info=progress2",
			"--rsync-path",
			format!("mkdir -p {} && rsync", dest_path).as_str(),
			run_server.as_str(),
		])
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.output()
		.unwrap_or_else(|e| {
			error!("Call ERROR: {}", e);
			exit(-5);
		});
	if !output.status.success() {
		eprintln!("Call Error: {:?}", output.stderr);
		exit(output.status.code().unwrap_or(1))
	}
}

fn keypair_run(
	host: &str,
	port: &i64,
	username: &str,
	private_key_file: &str,
	dest_path: &str,
	runner: &str,
	command: &str,
) {
	let mut ssh = Command::new("ssh");

	let run_server = format!("{}@{}", username, host);
	let run_command = format!("cd {} && {} {}", dest_path, runner, command);

	let output = ssh
		.args(&[
			"-i",
			format!("{}", private_key_file).as_str(),
			"-p",
			format!("{}", port).as_str(),
			run_server.as_str(),
			run_command.as_str(),
		])
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.stdin(Stdio::inherit())
		.output()
		.unwrap_or_else(|e| {
			error!("Call ERROR: {}", e);
			exit(-5);
		});

	if !output.status.success() {
		eprintln!("Call Error: {:?}", output.stderr);
		exit(output.status.code().unwrap_or(1))
	}
}

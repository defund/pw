use std::io;
use std::io::prelude::*;

use clipboard::{ClipboardContext, ClipboardProvider};
use rand::{Rng, thread_rng};
use rpassword::prompt_password_stdout;

use crate::backend::{self, Entry};

fn prompt_string(prompt: &str) -> io::Result<String> {
	print!("{}", prompt);
	let mut string = String::new();
	io::stdout().flush()?;
	io::stdin().read_line(&mut string)?;
	Ok(string.trim().to_string())
}

pub fn add(entries: &mut Vec<Entry>) -> io::Result<()> {
	let mut rng = thread_rng();

	let mut long;
	loop {
		long = prompt_string("Entry name: ")?;
		if long.is_empty() {
			println!("Entry name can't be empty.");
			continue;
		} else if entries.iter()
			.any(|entry| long == entry.long || long == entry.short) {
			println!("Name already in use.");
			continue;
		}
		break;
	}

	let mut short;
	loop {
		short = prompt_string("Shortened name (optional): ")?;
		if !short.is_empty() && entries.iter()
			.any(|entry| short == entry.long || short == entry.short) {
			println!("Name already in use.");
			continue;
		}
		break;
	}

	let extra = prompt_string("Extra information (optional): ")?;

	let password = if prompt_string("Randomly generate password? [Y/n] ")?
		.to_lowercase() != "n" {
		static LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
		static UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
		static DIGIT: &str = "0123456789";
		static SYMBOL: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

		let mut length: usize = 32;
		let mut charset = String::new();
		if prompt_string("Standard length (32) and charset (lower, upper, digit, symbol)? [Y/n] ")?
			.to_lowercase() != "n" {
			charset.push_str(LOWER);
			charset.push_str(UPPER);
			charset.push_str(DIGIT);
			charset.push_str(SYMBOL);
		} else {
			loop {
				let result = prompt_string("Length: ")?.parse();
				if result.is_err() {
					println!("Invalid length.");
					continue;
				}
				length = result.unwrap();
				break;
			}
			if prompt_string("Include lowercase characters? [Y/n] ")?
				.to_lowercase() != "n" {
				charset.push_str(LOWER);
			}
			if prompt_string("Include uppercase characters? [Y/n] ")?
				.to_lowercase() != "n" {
				charset.push_str(UPPER);
			}
			if prompt_string("Include digits? [Y/n] ")?
				.to_lowercase() != "n" {
				charset.push_str(DIGIT);
			}
			if prompt_string("Include symbols? [Y/n] ")?
				.to_lowercase() != "n" {
				charset.push_str(SYMBOL);
			}
			charset.push_str(&prompt_string("Additional characters (optional): ")?);
		}
		(0..length).map(|_| {
			let n = rng.gen_range(0, charset.len());
			charset.chars().nth(n).unwrap()
		}).collect()
	} else {
		let mut password;
		let mut confirm;
		loop {
			password = prompt_password_stdout("Password: ")?;
			confirm = prompt_password_stdout("Enter again: ")?;
			if password != confirm {
				println!("Passwords don't match.");
				continue;
			}
			break;
		}
		password
	};

	let mut master;
	let mut confirm;
	loop {
		master = prompt_password_stdout("Master key: ")?;
		if master.is_empty() {
			println!("Master key can't be empty.");
			continue;
		}
		confirm = prompt_password_stdout("Enter again: ")?;
		if master != confirm {
			println!("Master keys don't match.");
			continue;
		}
		break;
	}

	let mut salt = [0u8; 16];
	rng.fill(&mut salt);
	let mut entry = Entry {
		long: long,
		short: short,
		extra: extra,
		salt: salt,
		sealed: password.as_bytes().to_vec(),
	};
	backend::seal(&mut entry, &master).unwrap();
	entries.push(entry);

	println!("Success! Added entry.");
	Ok(())
}

pub fn del(entries: &mut Vec<Entry>) -> io::Result<()> {
	let mut name;
	loop {
		name = prompt_string("Entry name (or shortened): ")?;
		if name.is_empty() {
			println!("Name can't be empty.");
			continue;
		}
		break;
	}

	for (i, entry) in entries.iter().enumerate() {
		if name == entry.long || name == entry.short {
			println!("{}", entry);
			if prompt_string("Delete this entry? [y/N]: ")?
				.to_lowercase() == "y" {
				entries.remove(i);
				println!("Success! Removed entry.");
			}
			return Ok(());
		}	
	}

	println!("No entry found.");
	Ok(())
}

pub fn gen(entries: &Vec<Entry>, name: String) -> io::Result<()> {
	for entry in entries {
		if name == entry.long || name == entry.short {
			let mut master;
			loop {
				master = prompt_password_stdout("Master key: ")?;
				if master.is_empty() {
					println!("Master key can't be empty.");
					continue;
				}
				match backend::open(&entry, &master) {
					Ok(password) => {
						ClipboardProvider::new()
							.map(|mut clipboard: ClipboardContext| {
								clipboard.set_contents(String::from_utf8(password).unwrap())
							})
							.expect("Unable to set clipboard contents")
							.expect("Unable to access clipboard");
						println!("Success! Password copied to clipboard.");
					},
					Err(_) => {
						println!("Either master key was incorrect or entry is tampered.");
						continue;
					}
				}
				break;
			}
			return Ok(());
		}
	}

	println!("No entry found.");
	Ok(())
}

pub fn list(entries: &Vec<Entry>) {
	for entry in entries {
		println!("{}", entry);
	}
}

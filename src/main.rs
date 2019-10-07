mod backend;
mod command;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
	let path = match env::var("PW_PATH") {
		Ok(string) => Path::new(&string).to_path_buf(),
		Err(_) => dirs::home_dir()
			.expect("Home directory not found")
			.join(".pw.json")
	};

	let mut entries: Vec<backend::Entry> = if path.is_file() {
		let packed = fs::read_to_string(&path)
			.expect(&format!("Unable to read from {}", path.display()));
		serde_json::from_str(&packed)
			.expect(&format!("Malformed {}", path.display()))
	} else {
		Vec::new()
	};

	let mut args = env::args();
	let program = args.next().unwrap();
	match args.next() {
		Some(arg) => {
			if arg == "help" {
				println!("USAGE: {} <arg>", program);
				println!("match arg {{");
				println!("        \"\" => list entries");
				println!("     \"add\" => add an entry");
				println!("     \"del\" => delete an entry");
				println!("    \"help\" => display this message");
				println!("         _ => generate password for entry with this name");
				println!("}}");
				println!("data is located in {}", path.display());
			} else if arg == "add" {
				command::add(&mut entries).unwrap();
				entries.sort_unstable_by(|x, y| x.long.cmp(&y.long));
				let json = serde_json::to_string(&entries).unwrap();
				fs::write(&path, json)
					.expect(&format!("Unable to write to {}", path.display()));
			} else if arg == "del" {
				command::del(&mut entries).unwrap();
				let json = serde_json::to_string(&entries).unwrap();
				fs::write(&path, json)
					.expect(&format!("Unable to write to {}", path.display()));
			} else {
				command::gen(&entries, arg).unwrap();
			}
		}
		None => {
			command::list(&entries);
		}
	};
}

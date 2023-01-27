use std::{env, fs, io};
use std::path::{Path, PathBuf};
use std::fmt::Write;

use dunce::canonicalize;


fn main() {
	println!("cargo:rerun-if-changed=../tcd-ui/dist");

	let out_dir = env::var("OUT_DIR").unwrap();
	let files_routes = Path::new(&out_dir).join("dist_files.rs");

	let ui_dist = Path::new("../tcd-ui/dist");
	if !ui_dist.is_dir() {
		fs::write(files_routes, "pub fn add_routes(_: &mut FireBuilder) {}")
			.unwrap();
		return
	}

	// (uri, path)
	let mut files = vec![];

	read_dir(&mut files, "/", ui_dist).unwrap();


	let mut s = String::new();
	write!(s, "use fire::{{memory_file, fs::MemoryFile, FireBuilder}};\n\n")
		.unwrap();

	// add index
	let index_path = canonicalize("../tcd-ui/dist/index.html").unwrap();
	write!(s, "const INDEX: MemoryFile = \
		memory_file!(\"/\", {index_path:?});\n").unwrap();

	for (i, (uri, path)) in files.iter().enumerate() {
		write!(s, "const FILE_{i}: MemoryFile = \
			memory_file!({uri:?}, {path:?});\n").unwrap();
	}

	write!(s, "\npub fn add_routes(fire: &mut FireBuilder) {{\n").unwrap();

	// add index
	write!(s, "\tfire.add_route(INDEX);\n").unwrap();

	for (i, _) in files.iter().enumerate() {
		write!(s, "\tfire.add_route(FILE_{i});\n").unwrap();
	}

	write!(s, "}}\n").unwrap();

	
	fs::write(files_routes, s).unwrap();
}

fn read_dir(
	files: &mut Vec<(String, PathBuf)>,
	// should end with a /
	uri: &str,
	dir: impl AsRef<Path>
) -> io::Result<()> {
	for entry in fs::read_dir(dir).unwrap() {
		let entry = entry.unwrap();
		let name = entry.file_name().into_string().unwrap();

		if entry.file_type().unwrap().is_dir() {
			let uri = format!("{uri}{name}/");
			read_dir(files, &uri, entry.path())?;
			continue
		}

		files.push((
			format!("{uri}{name}"),
			canonicalize(entry.path()).unwrap()
		));
	}

	Ok(())
}
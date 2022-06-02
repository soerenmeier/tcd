
fn run() {
	run("dev");
}

fn run(kind) {
	if kind == "dev" {
		_run_dev();
	} else if kind == "release" {
		_run_release();
	} else {
		print("unknown kind");
	}
}

fn _run_dev() {
	let cargo = cmd(["cargo", "run", "--release"]);
	cargo.dir(".\\tcd-server");

	let npm = cmd(["cmd", "/c", "npm run dev"]);
	npm.dir(".\\tcd-ui");

	cmd::execute_parallel([cargo, npm]);
}

fn _run_release() {
	let npm = cmd(["cmd", "/c", "npm run build"]);
	npm.dir(".\\tcd-ui");
	npm.execute();

	let cargo = cmd(["cargo", "run", "--release", "--features", "self-host"]);
	cargo.dir(".\\tcd-server");
	cargo.execute();
}

fn build() {
	print("execute build");
	let npm = cmd(["cmd", "/c", "npm run build"]);
	npm.dir(".\\tcd-ui");
	npm.execute();

	let cargo = cmd(["cargo", "build", "--release", "--features", "self-host"]);
	cargo.dir(".\\tcd-server");
	cargo.execute();
}
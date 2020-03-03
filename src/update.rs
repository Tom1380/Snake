use {
	sha2::{Digest, Sha256},
	std::{
		fs::File,
		io::{self, Read},
	},
};

pub enum UpdateFailure {
	NoNewVersion,
	Failed,
}

pub fn update() -> Result<(), UpdateFailure> {
	// let mut tmpfile = tempfile::tempfile().unwrap();
	// let _ = reqwest::get("http://167.172.50.64")
	// 	.expect("request failed")
	// 	.copy_to(&mut tmpfile);
	let mut sha256 = Sha256::new();
	sha256.input(
		&File::open("snake.exe")
			.map_err(|_| UpdateFailure::Failed)?
			.bytes()
			.map(|r| r.unwrap())
			.collect::<Vec<u8>>(),
	);
	let old_hash = format!("{:x}", sha256.result());
	let new_hash = reqwest::get("http://167.172.50.64/hash")
		.map_err(|_| UpdateFailure::Failed)?
		.text()
		.map_err(|_| UpdateFailure::Failed)?;
	if old_hash == new_hash {
		return Err(UpdateFailure::NoNewVersion);
	}
	println!("{}\n{}", old_hash, new_hash);
	// let mut zip = zip::ZipArchive::new(tmpfile).unwrap();
	// let mut exe = zip.by_index(0).unwrap();
	// println!("Filename: {}", exe.name());
	// let mut out = File::create("snake1.exe").expect("failed to create file");
	// io::copy(&mut exe, &mut out).expect("failed to copy content");
	// let mut sha256 = Sha256::new();
	// sha256.input(
	// 	&File::open("snake1.exe")
	// 		.expect("bruh")
	// 		.bytes()
	// 		.map(|r| r.unwrap())
	// 		.collect::<Vec<u8>>(),
	// );
	// let new_hash = sha256.result();
	// if new_hash == old_hash {
	// 	println!("UGUALE");
	// } else {
	// 	println!("DIVERSO");
	// }
	Ok(())
}

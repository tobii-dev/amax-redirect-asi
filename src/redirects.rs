use std::{
	cmp::max,
	fmt::Debug,
	io::{BufRead, BufReader, Error},
	path::Path,
};
use windows::{
	Win32::System::{
		Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
	},
};

use std::ffi::{c_uchar, c_void};
use std::fs::File;

struct Redirect {
	/// Offset (Bytes) from .exe module base address.
	offset: isize,
	/// Original host, maybe useful for restoring or something...
	org: String,
	/// New host (to redirect to).
	dst: String,
}

impl Debug for Redirect {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let offset = std::format!("{:#010X}", &self.offset);
		let org = &self.org;
		let dst = &self.dst;
		write!(f, "Redirect @{offset} \"{org}\" -> \"{dst}\"")?;
		Ok(())
	}
}

pub fn init(ptr_base: *mut c_void) {
	let path_config = std::path::Path::new(".")
		.join("amax")
		.join("config")
		.join("amax-redirect.cfg");
	let redirects = read_redirects(&path_config).unwrap_or_else(|err| {
		let path_config = path_config.display();
		log::error!("Problem reading [{path_config}]. Does the file exist? Error: {err:?}");
		panic!("Could't read redirects config from [{path_config}].");
	});
	set_redirects(ptr_base, redirects);
}

fn read_redirects(path: &Path) -> Result<Vec<Redirect>, Error> {
	let mut r: Vec<Redirect> = vec![];
	let fd = File::open(path)?;
	let lines = BufReader::new(fd)
		.lines()
		.map(|l| l.unwrap())
		.filter(|l| !l.starts_with('#'));
	for (n, line) in lines.enumerate() {
		let mut words = line.split_whitespace();
		if let (Some(offset), Some(org), Some(dst)) = (words.next(), words.next(), words.next()) {
			let offset = parse_int::parse(offset).unwrap(); // parse_int crate insead of libc::strtol()
			let org = org.to_string();
			let dst = dst.to_string();
			let redirect = Redirect { offset, org, dst };
			r.push(redirect);
		} else {
			let path = path.display();
			log::warn!("[REDIRECTS] Ignoring malformed line in {path}:{n} \"{line}\"")
		}
	}
	Ok(r)
}

fn set_redirects(ptr_base: *mut c_void, redirects: Vec<Redirect>) {
	for redirect in redirects {
		log::debug!("[REDIRECTS] {redirect:#?}");
		// The pointer to the original string inside Blur.exe:
		let ptr = ptr_base.wrapping_offset(redirect.offset) as *mut c_uchar;

		// Size of the overwrite
		let size = max(redirect.org.len(), redirect.dst.len());
		// Save the original PAGE_PROTECTION_FLAGS of this memory chunk
		let src_protecc = &mut PAGE_PROTECTION_FLAGS::default();
		unsafe {
			// Yes, this is really necessary, even though we are already in Blur.exe address space...
			let _ = VirtualProtect(
				ptr as *const c_void,
				redirect.org.len(),
				PAGE_EXECUTE_READWRITE,
				src_protecc,
			);
		};

		// To overwrite the dst bits:
		for (i, dst_ch) in redirect.dst.chars().enumerate() {
			// C++ does make crazy pointer stuff a lot easier...
			let p = ptr.wrapping_offset(i.try_into().unwrap());
			unsafe {
				// (Over)write string of unsigned char
				p.write(dst_ch as c_uchar);
			};
		}

		// Set the rest of the original string to NULL, just in case
		for i in redirect.dst.len()..redirect.org.len() {
			let p = ptr.wrapping_offset(i.try_into().unwrap());
			unsafe {
				p.write(0); // null
			};
		}

		// And clean up: restore the original PAGE_PROTECTION_FLAGS:
		unsafe {
			let _ = VirtualProtect(
				ptr as *const c_void,
				size,
				*src_protecc,
				std::ptr::null_mut(),
			);
		};
	}
}

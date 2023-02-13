use std::cmp::max;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

use windows::core::PCSTR;

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
        write!(f, "Redirect @+{offset} \"{org}\" -> \"{dst}\"")?;
        Ok(())
    }
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(
    _dll_module: HINSTANCE,
    call_reason: u32,
    _lp_reserved: *mut c_void,
) -> i32 {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(init);
            true.into()
        }
        DLL_PROCESS_DETACH => {
            // Restore?
            true.into()
        },
        _ => false.into(),
    }
}

fn read_redirects(path: &Path) -> Result<Vec<Redirect>, Error> {
    let mut r: Vec<Redirect> = vec![];
    let fd = File::open(path)?;
    let lines = BufReader::new(fd)
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| !l.starts_with('#'));
    for line in lines {
        let mut words = line.split_whitespace();
        if let (Some(offset), Some(org), Some(dst)) = (words.next(), words.next(), words.next()) {
            let offset = parse_int::parse(offset).unwrap(); // parse_int crate insead of libc::strtol()
            let org = org.to_string();
            let dst = dst.to_string();
            let redirect = Redirect { offset, org, dst };
            r.push(redirect);
        } else {
            eprintln!("[REDIRECTS] Ignoring malformed line in config: [{line}]");
        }
    }
    Ok(r)
}

fn set_redirects(ptr_module_base: *mut c_void, redirects: Vec<Redirect>) {
    for redirect in redirects {
        println!("Setting redirect: {redirect:#?}");
        // The pointer to the original string inside Blur.exe:
        let ptr = ptr_module_base.wrapping_offset(redirect.offset) as *mut c_uchar;

        // Size of the overwrite
        let size = max(redirect.org.len(), redirect.dst.len());
        // Save the original PAGE_PROTECTION_FLAGS of this memory chunk
        let src_protecc = &mut PAGE_PROTECTION_FLAGS::default();
        unsafe {
            // Yes, this is really necessary, even though we are already in Blur.exe address space...
            VirtualProtect(
                ptr as *const c_void,
                redirect.org.len(),
                PAGE_EXECUTE_READWRITE,
                src_protecc,
            );
            // But not bothering with checking if the call to VirtualProtect succeeds...
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
            VirtualProtect(
                ptr as *const c_void,
                size,
                *src_protecc,
                std::ptr::null_mut(),
            );
        };
    }
}

fn init() {
    let exe_handle = unsafe {
        // Calling GetModuleHandle() with NULL param returns the base address of the daddy .exe
        GetModuleHandleA(PCSTR::null())
    };
    let exe_handle = exe_handle.unwrap();
    // windows::Win32::Foundation::HINSTANCE is weird;
    // I don't know an "elegant" way to get to the pointer.
    let ptr_module_base = exe_handle.0 as *mut c_void;

    // Hardcoded...
    let redirects = read_redirects(Path::new("./amax/cfg/amax-redirect.cfg")).unwrap();
    //redirects.iter().for_each(|r| println!("{r:#?}"));
    set_redirects(ptr_module_base, redirects);
}

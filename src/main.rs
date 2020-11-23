#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(link_args)]

#[allow(unused_attributes)]
#[link_args = "-nostartfiles -static"]
extern "C" {}

use core::mem::size_of;
use core::str;
use libc::c_int;

mod fd;
mod print;
mod start;
mod string;
mod syscall;
mod unwind_resume;
mod exit;
use exit::exit;
use core::fmt::{self, Write};
use core::slice::from_raw_parts as mkslice;

use crate::print::PrintBuffer;
pub use crate::start::_start;

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// # Safety
///
/// This function performs unsafe pointer aritmethic
pub unsafe fn strlen(mut s: *const u8) -> usize {
    let mut count = 0;
    while *s != b'\0' {
        count += 1;
        s = s.add(1);
    }
    count
}

const NIX_LD: &'static [u8] = b"NIX_LD=";
const NIX_LD_LIB_PATH: &'static [u8] = b"NIX_LD_LIBRARY_PATH=";

struct LdConfig {
    exe: Option<&'static [u8]>,
    lib_path: Option<&'static [u8]>,
}

unsafe fn slice_from_ptr(ptr: *const u8) -> &'static [u8] {
    mkslice(ptr, strlen(ptr))
}

unsafe fn process_env(mut envp: *const *const u8) -> LdConfig {
    let mut config = LdConfig {
        exe: None,
        lib_path: None,
    };
    while !(*envp).is_null() {
        let var = slice_from_ptr(*envp);
        if var.starts_with(NIX_LD) {
            config.exe = Some(&var[NIX_LD.len()..]);
        };
        if var.starts_with(NIX_LD_LIB_PATH) {
            config.lib_path = Some(&var[NIX_LD_LIB_PATH.len()..]);
        };

        envp = envp.add(1);
    }
    config
}

struct PrintableBytes<'a> {
  data: &'a [u8]
}

impl<'a> fmt::Display for PrintableBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        unsafe {
            write!(f, "{}", str::from_utf8_unchecked(self.data))
        }
    }
}

unsafe fn exe_name(args: &[*const u8]) -> &[u8] {
    if args.len() > 0 {
        slice_from_ptr(args[0])
    } else {
        b""
    }
}

#[cfg(target_pointer_width = "32")]
type ElfHeader = libc::Elf32_Phdr;

#[cfg(target_pointer_width = "64")]
type ElfHeader = libc::Elf64_Phdr;

/// # Safety
///
/// This function performs unsafe pointer aritmethic
#[no_mangle]
pub unsafe fn main(stack_top: *const u8) {
    let argc = *(stack_top as *const c_int);
    let argv = stack_top.add(size_of::<*const c_int>()) as *const *const u8;
    let envp = argv.add(argc as usize + 1) as *const *const u8;

    let args = mkslice(argv, argc as usize);

    let ld_config = process_env(envp);

    let mut buf = [0u8; 4096];
    let mut buf = PrintBuffer::new(&mut buf[..]);

    if ld_config.exe.is_none() {
        eprint!(
            buf,
            "Cannot execute binary {}: No NIX_LD environment variable set",
            PrintableBytes { data: exe_name(args) }
        );
        syscall::exit(1);
    }
    let fd = fd::open(ld_config.exe.unwrap(), libc::O_RDONLY);

    if let Some(lib_path) = ld_config.lib_path {
        eprint!(buf, "ld_library_path: {}\n", PrintableBytes{data: lib_path});
    }

    syscall::exit(0);
    exit(0);
}

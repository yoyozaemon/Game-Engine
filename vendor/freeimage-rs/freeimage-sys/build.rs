#[cfg(windows)]
extern crate cc;
extern crate fs_extra;

use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;
use std::mem::drop;
use std::path::PathBuf;

#[cfg(target_os="macos")]
use std::ffi::OsString;
#[cfg(target_os="macos")]
use std::os::unix::ffi::OsStringExt;

#[cfg(target_os="macos")]
fn build_macos() {
	let freeimage_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let freeimage_native_dir = Path::new(&freeimage_dir).join("FreeImage");
    let out_dir = env::var("OUT_DIR").unwrap();
	let freeimage_copy = Path::new(&out_dir).join("FreeImage");
	drop(fs_extra::dir::remove(&freeimage_copy));
	fs_extra::dir::copy(freeimage_native_dir, &out_dir, &fs_extra::dir::CopyOptions::new()).unwrap();
	let xcode_select_out: OsString = OsString::from_vec(Command::new("xcode-select")
                .arg("-print-path")
                .output().unwrap()
		        .stdout);
    let xcode_path = xcode_select_out.into_string().unwrap();
	let xcode_path = xcode_path.lines().next().unwrap();
    let sdks_path = Path::new(&xcode_path).join("Platforms/MacOSX.platform/Developer/SDKs");
    let last_sdk_entry = match fs::read_dir(&sdks_path){
        Ok(sdks) => sdks.last().unwrap().unwrap(),
        Err(_) => panic!("Couldn't find SDK at {}, probably xcode is not installed",sdks_path.to_str().unwrap())
    };

    let sdk = last_sdk_entry.path().as_path().file_stem().unwrap().to_str().unwrap().to_string();
    if sdk.contains("MacOSX"){
        let version = &sdk[6..];
        let output = Command::new("make")
		    .current_dir(&freeimage_copy)
		    .env("MACOSX_SDK",version)
		    .arg("-j4")
		    .output()
			.unwrap();
		
		if !output.status.success(){
			panic!("{}", String::from_utf8(output.stdout).unwrap());
		}

	    let out_dir = env::var("OUT_DIR").unwrap();
	    let dest_path = Path::new(&out_dir).join("libfreeimage.a");
	    fs::copy(freeimage_copy.join("libfreeimage.a"),dest_path).unwrap();
	    println!("cargo:rustc-flags= -L native={}",out_dir);

    }else{
        panic!("Couldn't find SDK at {}, probably xcode is not installed",sdks_path.to_str().unwrap())
    }
}

fn build_linux() {
	let freeimage_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let freeimage_native_dir = Path::new(&freeimage_dir).join("FreeImage");
    let out_dir = env::var("OUT_DIR").unwrap();
	let freeimage_copy = Path::new(&out_dir).join("FreeImage");
	drop(fs_extra::dir::remove(&freeimage_copy));
	fs_extra::dir::copy(freeimage_native_dir, &out_dir, &fs_extra::dir::CopyOptions::new()).unwrap();
    let output = Command::new("make")
	    .current_dir(&freeimage_copy)
	    .arg("-j4")
	    .output()
		.unwrap();
		
	if !output.status.success(){
		panic!("{}", String::from_utf8(output.stdout).unwrap());
	}
	
    let dest_path = Path::new(&out_dir).join("libfreeimage.a");
    fs::copy(freeimage_copy.join("Dist/libfreeimage.a"),dest_path).unwrap();
    println!("cargo:rustc-flags= -L native={}",out_dir);
}

fn build_emscripten() {
	let freeimage_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let freeimage_native_dir = Path::new(&freeimage_dir).join("FreeImage");
    let out_dir = env::var("OUT_DIR").unwrap();
	let freeimage_copy = Path::new(&out_dir).join("FreeImage");
	drop(fs_extra::dir::remove(&freeimage_copy));
	fs_extra::dir::copy(freeimage_native_dir, &out_dir, &fs_extra::dir::CopyOptions::new()).unwrap();
    let output = Command::new("emmake")
		.arg("make")
	    .current_dir(&freeimage_copy)
	    .arg("-j4")
	    .output()
		.unwrap();
		
	if !output.status.success(){
		panic!("{}", String::from_utf8(output.stdout).unwrap());
	}
		
    let dest_path = Path::new(&out_dir).join("libfreeimage.a");
    fs::copy(freeimage_copy.join("Dist/libfreeimage.a"),dest_path).unwrap();
    println!("cargo:rustc-flags= -L native={}",out_dir);
}

#[cfg(windows)]
fn retarget_ms_proj(target: &str, proj: &str, freeimage_native_dir: &PathBuf){
	let mut devenv = cc::windows_registry::find(target, "devenv.exe")
		.expect("Couldn't find devenv, perhaps you need to install visual studio?");
	let output = devenv
		.arg(proj)
		.arg("/upgrade")
		.current_dir(&freeimage_native_dir)
		.output()
		.expect("Couldn't Freeiamge visual studio update solution");
	if !output.status.success(){
		panic!("{}", String::from_utf8(output.stdout).unwrap());
	}
}

#[cfg(windows)]
fn build_windows(target: &str) {
	let freeimage_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let freeimage_native_dir = Path::new(&freeimage_dir).join("FreeImage");
    let out_dir = env::var("OUT_DIR").unwrap();
	let freeimage_copy = Path::new(&out_dir).join("FreeImage");
	drop(fs_extra::dir::remove(&freeimage_copy));
	fs_extra::dir::copy(freeimage_native_dir, &out_dir, &fs_extra::dir::CopyOptions::new()).unwrap();
	let freeimage_proj = "FreeImage.2017.sln";

	retarget_ms_proj(target, freeimage_proj, &freeimage_copy);

	let mut msbuild = cc::windows_registry::find(target, "msbuild.exe")
		.expect("Couldn't find msbuild, perhaps you need to install visual studio?");

	#[cfg(debug_assertions)]
	let config = "Debug";

	#[cfg(not(debug_assertions))]
	let config = "Release";

	let platform = if target.contains("x86_64") {
		"x64"
	} else if target.contains("thumbv7a") {
		"arm"
	} else if target.contains("aarch64") {
		"ARM64"
	} else if target.contains("i686") {
		"Win32"
	} else {
		panic!("unsupported msvc target: {}", target);
	};

	let output = msbuild.arg(freeimage_proj)
		.arg(&format!("-property:Configuration={}", config))
		.arg(&format!("-property:Platform={}", platform))
		.current_dir(&freeimage_copy)
		.output()
		.unwrap();

	if !output.status.success(){
		panic!("{}", String::from_utf8(output.stdout).unwrap());
	}

	#[cfg(debug_assertions)]
	let libname = "FreeImaged";

	#[cfg(not(debug_assertions))]
	let libname = "FreeImage";

	let out_dir = env::var("OUT_DIR").unwrap();
	let src_dir = freeimage_copy
		.join(platform)
		.join(config);

	let lib_name = format!("{}.lib", libname);
	let dst_path = Path::new(&out_dir).join(&lib_name);
	let src_path = src_dir.join(&lib_name);
	fs::copy(src_path, dst_path).unwrap();

	let dll_name = format!("{}.dll", libname);
	let dst_path = Path::new(&out_dir).join(&dll_name);
	let src_path = src_dir.join(&dll_name);
	fs::copy(src_path, dst_path).unwrap();

	println!("cargo:rustc-flags= -L native={}",out_dir);
}

fn main(){
	let target_triple = env::var("TARGET").unwrap();
	if target_triple.contains("linux") {
		build_linux()
	}else if target_triple.contains("darwin") {
		#[cfg(target_os="macos")]
		build_macos()
	}else if target_triple.contains("emscripten") {
		build_emscripten()
	}else if target_triple.contains("windows"){
		#[cfg(windows)]
		build_windows(&target_triple)
	}else{
		panic!("target OS {} not suported yet", target_triple);
	}
}

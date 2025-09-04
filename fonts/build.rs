use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use zip::ZipArchive;

fn main() {
    // Output font path
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_font = out_dir.join("SourceHanSansSC-Regular.otf");

    // If already exists (incremental build), skip
    if target_font.exists() {
        println!("cargo:rerun-if-changed=build.rs");
        return;
    }

    // Allow overriding via env: FONT_TTF
    if let Ok(path) = env::var("FONT_TTF") {
        let src = PathBuf::from(path);
        if let Err(e) = fs::copy(&src, &target_font) {
            eprintln!("warning: failed to copy FONT_TTF: {e}");
        } else {
            println!("cargo:rerun-if-env-changed=FONT_TTF");
            return;
        }
    }

    // Do NOT allow skipping: we require a real font for deterministic text rendering.

    // Try to download SC zip (pinned version) then extract the Regular OTF
    let zip_url = "https://github.com/adobe-fonts/source-han-sans/releases/download/2.005R/09_SourceHanSansSC.zip";
    let zip_path = out_dir.join("SourceHanSansSC.zip");
    let mut ok = false;
    let status = Command::new("curl")
        .args(["-L", "-f", "-o", zip_path.to_str().unwrap(), zip_url])
        .status();
    if let Ok(st) = status
        && st.success()
    {
        ok = true;
    }
    if !ok {
        let status = Command::new("wget")
            .args(["-O", zip_path.to_str().unwrap(), zip_url])
            .status();
        if let Ok(st) = status
            && st.success()
        {
            ok = true;
        }
    }
    if !ok {
        panic!(
            "Failed to download {}. Provide FONT_TTF env var or allow network.",
            zip_url
        );
    }

    // Extract desired OTF from zip
    let mut data = Vec::new();
    {
        let mut f = fs::File::open(&zip_path).expect("zip open failed");
        f.read_to_end(&mut data).expect("zip read failed");
    }
    let reader = std::io::Cursor::new(data);
    let mut zip = ZipArchive::new(reader).expect("zip parse failed");
    // Try common paths: either at root or under OTF/
    let mut extracted = false;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let name = file.name().to_string();
        if name.ends_with("SourceHanSansSC-Regular.otf") {
            let mut buf = Vec::new();
            std::io::copy(&mut file, &mut buf).expect("extract copy failed");
            fs::write(&target_font, &buf).expect("write otf failed");
            extracted = true;
            break;
        }
    }
    if !extracted {
        panic!("Regular OTF not found in zip: expected SourceHanSansSC-Regular.otf");
    }

    println!("cargo:rerun-if-changed=build.rs");
}

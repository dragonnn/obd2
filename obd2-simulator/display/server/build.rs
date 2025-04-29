use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    //println!("cargo:rustc-link-search={}", out.display());
    println!(
        "cargo:rustc-link-search={}",
        "/home/mateusz/Kia/kia-obd2-esp32c3-v2/obd2-simulator/display"
    );

    Ok(())
}

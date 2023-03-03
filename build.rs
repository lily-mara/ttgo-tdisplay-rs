use std::{fs::File, io::Write, path::PathBuf};

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;

    println!("cargo:rerun-if-changed=images");

    let out_path = PathBuf::from(std::env::var("OUT_DIR")?).join("images.rs");

    let mut outfile = File::create(out_path)?;

    outfile.write_all(b"pub(crate) const IMAGE_DATA: &[&[u8]] = &[\n")?;

    for result in glob::glob("./images/*.bmp")? {
        let path = result?.canonicalize()?;

        outfile.write_fmt(format_args!(
            "    include_bytes!(\"{}\"),\n",
            path.display()
        ))?;
    }

    outfile.write_all(b"];\n")?;

    Ok(())
}

//! Build script for xero-toolkit GUI
//!
//! Handles resource optimization (PNG/SVG) and GLib resource compilation.

use std::{fs, path::Path};

use oxipng::{InFile, Options, OutFile};
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resources");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let optimized_dir = Path::new(&out_dir).join("optimized_resources");

    copy_dir_recursive(Path::new("resources"), &optimized_dir)
        .expect("Failed to copy resources directory");

    let stats = optimize_images(&optimized_dir);

    println!(
        "cargo:warning=Optimized {} files: {} -> {} bytes ({}% reduction)",
        stats.count,
        stats.original,
        stats.optimized,
        stats.reduction_percent()
    );

    let gresource_xml = optimized_dir.join("resources.gresource.xml");
    glib_build_tools::compile_resources(
        &[optimized_dir.to_str().unwrap()],
        gresource_xml.to_str().unwrap(),
        "xyz.xerolinux.xero-toolkit.gresource",
    );
}

#[derive(Default)]
struct OptimizationStats {
    count: usize,
    original: usize,
    optimized: usize,
}

impl OptimizationStats {
    fn add(&mut self, original: usize, optimized: usize) {
        self.count += 1;
        self.original += original;
        self.optimized += optimized;
    }

    fn reduction_percent(&self) -> i64 {
        if self.original == 0 {
            return 0;
        }
        let saved = self.original as i64 - self.optimized as i64;
        (saved * 100) / self.original as i64
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn optimize_images(dir: &Path) -> OptimizationStats {
    let mut stats = OptimizationStats::default();

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };

        match ext.to_lowercase().as_str() {
            "png" => {
                if let Some((orig, opt)) = optimize_png(path) {
                    stats.add(orig, opt);
                }
            }
            "svg" => {
                if let Some((orig, opt)) = optimize_svg(path) {
                    stats.add(orig, opt);
                }
            }
            _ => {}
        }
    }

    stats
}

fn optimize_png(path: &Path) -> Option<(usize, usize)> {
    let options = Options {
        strip: oxipng::StripChunks::Safe,
        optimize_alpha: true,
        ..Options::from_preset(2)
    };

    let infile = InFile::Path(path.to_path_buf());
    let outfile = OutFile::Path {
        path: Some(path.to_path_buf()),
        preserve_attrs: false,
    };

    oxipng::optimize(&infile, &outfile, &options).ok()
}

fn optimize_svg(path: &Path) -> Option<(usize, usize)> {
    let original = fs::read_to_string(path).ok()?;
    let original_size = original.len();

    let optimized = svgtidy::optimize(&original);
    let optimized_size = optimized.len();

    fs::write(path, &optimized).ok()?;

    Some((original_size, optimized_size))
}

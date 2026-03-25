fn main() {
    // Generate icon.ico from icon.png and embed it into the exe (Windows only)
    #[cfg(target_os = "windows")]
    embed_icon();

    println!("cargo:rerun-if-changed=icon.png");
}

#[cfg(target_os = "windows")]
fn embed_icon() {
    use image::imageops::FilterType;
    use image::ImageFormat;
    use std::io::BufWriter;

    let ico_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("app.ico");

    // Load the source PNG
    let src = image::open("assets/icon.png").expect("icon.png not found");

    let file = std::fs::File::create(&ico_path).unwrap();
    let mut writer = BufWriter::new(file);

    // Build ICO with multiple resolutions using the ico crate workaround:
    // image crate's ICO encoder writes one image; we write each size as a
    // separate PNG inside the ICO container using the `ico` approach below.
    // Simplest: write a single 256px ICO (Windows scales it fine).
    let resized = src.resize_exact(256, 256, FilterType::Lanczos3);
    resized
        .write_to(&mut writer, ImageFormat::Ico)
        .expect("Failed to write ico");

    // Tell winres to embed the generated ICO
    let mut res = winres::WindowsResource::new();
    res.set_icon(ico_path.to_str().unwrap());
    res.compile().unwrap();
}

use std::path::Path;

pub fn generate_thumbnails(recipe_dir: &Path) -> Result<(), anyhow::Error> {
    for entry in walkdir::WalkDir::new(recipe_dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        let fname = path.display().to_string();
        if !fname.ends_with(".jpg") {
            continue;
        }

        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        if file_stem.ends_with("_thumbnail") {
            println!("SKIP {}", path.display());
            continue;
        }

        let thumbnail = path
            .parent()
            .unwrap()
            .join(format!("{}_thumbnail.jpg", file_stem));

        if thumbnail.exists() {
            println!("SKIP {}", path.display());
            continue;
        }

        println!("generating thumbnail for {}", path.display());

        let source = image::open(fname).unwrap();

        let thumbnail_img = image::imageops::thumbnail(&source, 200, 200);

        thumbnail_img.save(thumbnail).unwrap();
    }

    Ok(())
}

use anyhow::{Context, Result};
use std::env::args;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use tera::Tera;
fn main() -> Result<()> {
    let mut args = args().skip(1);
    let templates_dir = &args
        .next()
        .context("'templates_dir' argument is missing.")?;
    let output_dir = &args.next().context("'output_dir' argument is missing.")?;
    let output_dir = Path::new(output_dir);

    create_dir_all(output_dir).context("Failed to create output directory.")?;

    let tera = Tera::new(&format!("{templates_dir}/**/*.html"))
        .context("Failed to build Tera instance.")?;

    let templates_dir = Path::new(templates_dir);

    for page in templates_dir
        .join("pages")
        .read_dir()
        .context("Failed to read pages directory.")?
        .flatten()
        .flat_map(|f| f.file_name().into_string())
    {
        let page_path = Path::new("pages").join(&page);
        let page_path = page_path
            .to_str()
            .context(format!("File {page} has a name that is not valid Unicode."))?;

        let output_path = output_dir.join(&page);

        let mut output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)
            .context("Failed to open file {output_path}")?;

        let context = tera::Context::new();
        tera.render_to(page_path, &context, &mut output_file)
            .context(format!("Failed to render template {page}."))?;
    }

    Ok(())
}

use anyhow::{Context, Result};
use std::env::args;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use tera::Tera;

fn build_page<D: AsRef<Path>>(
    page: &str,
    destination: D,
    tera: &Tera,
    context: tera::Context,
) -> Result<()> {
    let page_path = Path::new("pages").join(page);
    let page_path = page_path
        .to_str()
        .context(format!("File {page} has a name that is not valid Unicode."))?;

    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(destination)
        .context("Failed to open file {output_path}")?;

    tera.render_to(page_path, &context, &mut output_file)
        .context(format!("Failed to render template {page}."))
}

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

    build_page(
        "home.html",
        output_dir.join("home.html"),
        &tera,
        tera::Context::new(),
    )
    .context("Failed to build home.")?;

    Ok(())
}

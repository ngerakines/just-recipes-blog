#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use url::Url;

#[cfg(feature = "server")]
use axum::{http::StatusCode, service, Router};
#[cfg(feature = "server")]
use std::{convert::Infallible, net::SocketAddr};
#[cfg(feature = "server")]
use tower_http::services::ServeDir;

use jrb::model::{Recipe, SiteView};
use jrb::site::build_site;

#[cfg(feature = "validate")]
use jrb::validate::validate_recipes;

#[cfg(feature = "convert")]
use jrb::image::generate_thumbnails;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn parse_url(src: &str) -> Result<String, anyhow::Error> {
    let mut base_url = Url::parse(src)?;

    if base_url.scheme() != "https" && base_url.scheme() != "http" {
        return Err(anyhow!("invalid url schema: {}", src));
    }
    if !base_url.has_host() {
        return Err(anyhow!("invalid url host: {}", src));
    }
    base_url.set_fragment(None);
    base_url.set_query(None);

    if !base_url.path().ends_with('/') {
        return Err(anyhow!("invalid url path: {}", src));
    }

    Ok(base_url.to_string())
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "The justrecipes.blog builder", version=built_info::PKG_VERSION)]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "recipes")]
    /// The directory that contains recipe yaml files.
    recipe_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "templates")]
    /// The directory that contains website template files.
    templates_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "static")]
    /// The directory that contains static assets like css, js, icons, etc.
    static_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "public")]
    /// The directory that contains static and generated content.
    public_dir: PathBuf,

    #[structopt(long, default_value = "en_US")]
    locales: Vec<String>,

    #[structopt(long, parse(try_from_str = parse_url), default_value = "http://localhost:8080/")]
    /// The base URL for the generated site.
    public_url: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    /// Build the website.
    Build {},

    #[cfg(feature = "server")]
    /// Serve the generated website.
    Server {
        #[structopt(long, default_value = "0.0.0.0:8080")]
        listen: String,
    },

    #[cfg(feature = "validate")]
    /// Validate recipe files.
    Validate {},

    #[cfg(feature = "convert")]
    /// Create thumbnails for recipe images.
    Convert {},

    /// Generate and stub a new recipe file.
    Init {
        #[structopt(long)]
        id: Option<String>,

        #[structopt(long)]
        name: Option<String>,

        #[structopt(long)]
        mock: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    debug!("{:?}", opt);

    match opt.cmd {
        Command::Build {} => cmd_build(
            &opt.recipe_dir,
            &opt.static_dir,
            &opt.templates_dir,
            &opt.public_dir,
            &opt.locales,
            &opt.public_url,
        ),

        #[cfg(feature = "server")]
        Command::Server { listen } => cmd_server(&opt.public_dir, &listen).await,

        #[cfg(feature = "validate")]
        Command::Validate {} => cmd_validate(&opt.recipe_dir).await,

        #[cfg(feature = "convert")]
        Command::Convert {} => cmd_convert(&opt.recipe_dir).await,

        Command::Init { id, name, mock } => cmd_init(&opt.recipe_dir, id, name, mock),
    }
}

fn cmd_build(
    recipe_dir: &Path,
    static_dir: &Path,
    templates_dir: &Path,
    public_dir: &Path,
    site_locales: &[String],
    public_url: &str,
) -> Result<(), anyhow::Error> {
    build_site(
        recipe_dir,
        static_dir,
        templates_dir,
        public_dir,
        site_locales,
        SiteView::new(public_url, built_info::PKG_VERSION),
    )
}

#[cfg(feature = "validate")]
async fn cmd_validate(recipe_dir: &Path) -> Result<(), anyhow::Error> {
    Ok(validate_recipes(recipe_dir)?)
}

#[cfg(feature = "convert")]
async fn cmd_convert(recipe_dir: &Path) -> Result<(), anyhow::Error> {
    Ok(generate_thumbnails(recipe_dir)?)
}

#[cfg(feature = "server")]
async fn cmd_server(public_dir: &Path, listen: &str) -> Result<(), anyhow::Error> {
    let app = Router::new().nest(
        "/",
        service::get(ServeDir::new(public_dir)).handle_error(|error: std::io::Error| {
            Ok::<_, Infallible>((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            ))
        }),
    );

    let addr: SocketAddr = listen.parse().expect("invalid listen address");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

#[cfg(unix)]
pub async fn shutdown_signal() {
    use std::io;
    use tokio::signal::unix::SignalKind;

    async fn terminate() -> io::Result<()> {
        tokio::signal::unix::signal(SignalKind::terminate())?
            .recv()
            .await;
        Ok(())
    }

    tokio::select! {
        _ = terminate() => {},
        _ = tokio::signal::ctrl_c() => {},
    }
}

#[cfg(windows)]
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("faild to install interupt handler");
}

fn cmd_init(
    recipe_dir: &Path,
    recipe_id: Option<String>,
    name: Option<String>,
    mock: bool,
) -> Result<(), anyhow::Error> {
    let recipe = Recipe::init(recipe_id, name, mock);
    let yaml_out = serde_yaml::to_string(&recipe)?;

    fs::write(
        recipe_dir.join(format!("{}.yml", recipe.slug.localized(None)?)),
        &yaml_out,
    )
    .expect("Unable to write file");

    println!("{}", &yaml_out);
    Ok(())
}

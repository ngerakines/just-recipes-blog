#[macro_use]
extern crate log;

use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[cfg(feature = "server")]
use axum::{http::StatusCode, service, Router};
#[cfg(feature = "server")]
use std::{convert::Infallible, net::SocketAddr};
#[cfg(feature = "server")]
use tower_http::services::ServeDir;

use uuid::Uuid;

use jrb::model::{Recipe, SiteView};
use jrb::site::build_site;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "The justrecipes.blog builder", version=built_info::PKG_VERSION)]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "recipes")]
    recipe_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "templates")]
    templates_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "static")]
    static_dir: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "public")]
    public_dir: PathBuf,

    #[structopt(long, default_value = "en_US")]
    locales: Vec<String>,

    #[structopt(long, default_value = "http://localhost:8080/")]
    public_url: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    Build {},

    #[cfg(feature = "server")]
    Server {
        #[structopt(long, default_value = "0.0.0.0:8080")]
        listen: String,
    },

    Init {
        #[structopt(long)]
        id: Option<Uuid>,

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
        SiteView::new(public_url, &built_info::PKG_VERSION.to_string()),
    )
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
    recipe_id: Option<Uuid>,
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

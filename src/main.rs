use std::path::PathBuf;
use pretty_env_logger;
use structopt::StructOpt;

mod requirements;
mod package;
mod errors;
mod godot;


#[derive(StructOpt, Debug)]
#[structopt(version = "0.1", author = "Hagsteel", about = "Unofficial Godot tool")]
enum Opts {
    Install {
        #[structopt(short, long)]
        requirements: Option<PathBuf>,

        #[structopt(name = "path")]
        path: Option<String>,
    },
    Verify {
        #[structopt(short, long)]
        manifest: PathBuf,
    },
    Init {
        #[structopt(name = "project name")]
        name: String,
    }
}


fn main() {
    pretty_env_logger::init();
    let opt = Opts::from_args();
    match opt {
        Opts::Install { requirements, path } => package::install_packages(requirements, path, false),
        Opts::Verify { manifest } => package::verify(manifest),
        Opts::Init { name } => godot::init(name),
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let resp = reqwest::get("https://httpbin.org/ip")
//         .await?
//         .text()
//         .await?;
//     println!("{:#?}", resp);
//     Ok(())
// }


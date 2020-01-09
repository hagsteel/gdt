use std::path::PathBuf;
use pretty_env_logger;
use structopt::StructOpt;

mod requirements;
mod package;
mod errors;
mod godot;


#[derive(StructOpt, Debug)]
#[structopt(version = "0.1", author = "Hagsteel", about = "Unofficial Godot tool\nNote!!! always read manifest files before installing")]
enum Opts {
    #[structopt(about = "Install one or more packages")]
    Install {
        #[structopt(short, long, help = "file containing list of packages")]
        requirements: Option<PathBuf>,

        #[structopt(name = "path", help = "path to a single package")]
        path: Option<String>,
    },
    #[structopt(about = "Verify a manifest file")]
    Verify {
        #[structopt(short, long, help = "path to manifest file")]
        manifest: PathBuf,
    },
    #[structopt(about = "Init a Godot project")]
    Init {
        #[structopt(name = "project name", help = "godot project name")]
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


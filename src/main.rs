use std::fs::File;
use std::path::Path;
use structopt::StructOpt;

mod global_conf;
use global_conf::*;

#[derive(Debug, StructOpt)]
/// Provide file input
pub struct Opt {
    #[structopt(name = "CONF_PATH", required = true)]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = Opt::from_args();

    let path = file.path;
    if !Path::new(&path).exists() {
        panic!("No {} found", path);
    }

    let file = File::open(&path)?;
    let config = Config::from_file(file)?;
    println!("{}\n {}", path, config.summary());
    Ok(())
}

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
    let opts = Opt::from_args();
    let path = Path::new(&opts.path);

    if !path.exists() {
        panic!("Path {} does not exist", &opts.path);
    }

    if path.is_file() {
        let file = File::open(&path)?;
        let config = Config::from_file(file)?;
        println!("{}", config.summary());
    } else if path.is_dir() {
        for entry in path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let file = File::open(&entry.path())?;
                println!("{:?}", file);
                let config = Config::from_file(file)?;
                println!("{}", config.summary());
            }
        }
    }

    Ok(())
}

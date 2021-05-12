use std::fs::File;
use std::path::Path;
use structopt::StructOpt;

use regions::Region;

mod global_conf;
use global_conf::*;

#[derive(Debug, StructOpt)]
/// Provide file input
pub struct Opt {
    #[structopt(name = "CONF_PATH", required = true)]
    path: String,
    #[structopt(long, short)]
    region: Region,
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

        let channels = opts.region.get_uplink_frequencies();

        for (index, channel) in channels.iter().enumerate() {
            if let Some(config_frequency) = config.frequency(index) {
                if &(config_frequency as usize) != channel {
                    println!(
                        "Channel {} mismatch! Expected {}, but got {}",
                        index, channel, config_frequency
                    );
                }
            } else {
                println!(
                    "Channel {} mismatch! Expected {}, but channel not configured",
                    index, channel
                );
            }
        }
    }
    Ok(())
}

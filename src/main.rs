use clap::Parser;

/// Example from the doc
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A name
    #[arg(short, long)]
    name: String,
    /// A count
    #[arg(short, long, default_value_t = 1)]
    count: u8
}

fn main() {
    pretty_env_logger::init();

    log::info!("Entered 'main'");
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello {}", args.name);
    }
    log::info!("Exiting 'main'");
}

use clap::Parser;
use edit_data::{run, Options};

fn main() {
    let options = Options::parse();

    run(options);
}

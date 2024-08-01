use clap::Parser;

use migrate_data::{convert_data, Options};

fn main() {
    let options = Options::parse();

    convert_data(options.v1, options.v2, options.data_type);
}

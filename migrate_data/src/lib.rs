mod account;
mod article;

use std::path::PathBuf;
use std::{fmt::Display, fs};

use clap::{Args, Parser, Subcommand, ValueEnum};

pub const PREVIOUS_VERSION: Version = Version::V0_1_0;
pub const CURRENT_VERSION: Version = Version::V0_2_0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Version {
    V0_1_0,
    V0_2_0,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V0_2_0 => write!(f, "v0-2-0"),
            Version::V0_1_0 => write!(f, "v0-1-0"),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "migrate_data",
    about = "script to migrate the data of the msj server from one version to another"
)]

pub struct Options {
    /// The version converting from
    #[arg(long, default_value_t = PREVIOUS_VERSION)]
    pub v1: Version,

    /// The version converting to
    #[arg(long, default_value_t = CURRENT_VERSION)]
    pub v2: Version,

    /// type of data being converted
    #[command(subcommand)]
    pub data_type: DataType,
}

#[derive(Subcommand, Debug)]
pub enum DataType {
    Account(AccountArgs),
    Article(ArticleArgs),
}

#[derive(Args, Debug)]
pub struct AccountArgs {
    /// The path to the file containing the accounts (accounts.dat)
    input_file: PathBuf,

    output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct ArticleArgs {
    /// The path to the dir containing the artilces
    input_dir: PathBuf,

    output_dir: PathBuf,
}

pub fn convert_data(v1: Version, v2: Version, data_type: DataType) {
    match data_type {
        DataType::Account(args) => {
            let accounts = account::read_accounts(args.input_file);
            let converted_accounts = account::convert_accounts(v1, v2, accounts);
            fs::write(args.output_file, converted_accounts).expect("faield to write accounts file");
        }
        DataType::Article(args) => {
            let articles = article::read_articles(args.input_dir);
            let converted_articles = article::convert_articles(v1, v2, articles);
            article::write_articles(args.output_dir, converted_articles);
        }
    }
}

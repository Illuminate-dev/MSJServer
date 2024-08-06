use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use server::{Account, Perms};

#[derive(Parser, Debug)]
#[clap(
    name = "edit_data",
    about = "script to edit the data of the msj server"
)]

pub struct Options {
    /// The version converting from

    /// type of data being converted
    #[command(subcommand)]
    pub data_type: DataType,
}

#[derive(Subcommand, Debug)]
pub enum DataType {
    Account(AccountArgs),
}

#[derive(Args, Debug)]
pub struct AccountArgs {
    /// The path to the file containing the accounts (accounts.dat)
    #[arg(short, long)]
    file: PathBuf,

    #[command(subcommand)]
    subcommand: AccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    ChangePerm(ChangePermArgs),
}

#[derive(Args, Debug)]
pub struct ChangePermArgs {
    username: String,

    #[arg(value_parser=parse_perm_var)]
    perm: PermsWrapper,
}

#[derive(Debug, Clone)]
pub struct PermsWrapper(Perms);

fn parse_perm_var(perm: &str) -> Result<PermsWrapper, std::io::Error> {
    Perms::try_from(perm)
        .map(PermsWrapper)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid perm var"))
}

pub fn run(options: Options) {
    match options.data_type {
        DataType::Account(args) => match args.subcommand {
            AccountCommand::ChangePerm(cpargs) => {
                let mut accounts: Vec<Account> =
                    bincode::deserialize(read_accounts(&args.file).as_slice())
                        .expect("failed to deserialize accounts");

                let mut found = false;

                for account in accounts.iter_mut() {
                    if account.username == cpargs.username {
                        account.permission = cpargs.perm.0;
                        found = true;
                        break;
                    }
                }

                if !found {
                    eprintln!("account not found");
                } else {
                    write_accounts(&args.file, accounts).expect("failed to write to file");
                }
            }
        },
    }
}

fn read_accounts(file_path: &PathBuf) -> Vec<u8> {
    std::fs::read(file_path).expect("failed to read accounts file")
}

fn write_accounts(file_path: &PathBuf, accounts: Vec<Account>) -> Result<(), std::io::Error> {
    std::fs::write(
        file_path,
        bincode::serialize(&accounts).expect("failed to serialize accounts"),
    )
}

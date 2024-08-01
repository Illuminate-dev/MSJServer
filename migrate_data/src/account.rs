use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Version;

#[derive(Debug, Serialize, Deserialize)]
pub enum Perms {
    Admin,
    Editor,
    User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountV0_1_0 {
    pub username: String,
    pub permission: Perms,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub fn read_accounts(file_path: PathBuf) -> Vec<u8> {
    let data = fs::read(file_path).expect("failed to read accounts file");
    data
}

pub fn convert_accounts(v1: Version, v2: Version, accounts: Vec<u8>) -> Vec<u8> {
    let accounts: Vec<AccountV0_1_0> =
        bincode::deserialize(accounts.as_slice()).expect("failed to deserialize accounts");
    bincode::serialize(&accounts).expect("failed to serialize accounts")
}

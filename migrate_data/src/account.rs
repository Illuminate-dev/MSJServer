use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Version, VERSION_COUNT};

type ConversionFn = fn(Vec<u8>) -> Vec<u8>;

// [v1->v2,v2->v3, etc]
const UPGRADE_FN: [Option<ConversionFn>; VERSION_COUNT - 1] = [None];

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
    fs::read(file_path).expect("failed to read accounts file")
}

pub fn convert_accounts(mut v1: Version, v2: Version, mut accounts: Vec<u8>) -> Vec<u8> {
    while v1 < v2 {
        let f = UPGRADE_FN[u8::from(v1) as usize];

        if let Some(f) = f {
            accounts = f(accounts);
        }

        v1 += 1;
    }
    let accounts: Vec<AccountV0_1_0> =
        bincode::deserialize(accounts.as_slice()).expect("failed to deserialize accounts");
    bincode::serialize(&accounts).expect("failed to serialize accounts")
}

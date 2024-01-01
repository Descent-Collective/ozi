use std::time::{SystemTime, UNIX_EPOCH};

use ethers::{
    signers::{LocalWallet, Signer},
    types::Bytes,
};

pub fn get_unix_timestamp() -> u64 {
    let current_time = SystemTime::now();
    let unix_timestamp = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    unix_timestamp
}

pub async fn sign_message(wallet: LocalWallet, message: Vec<u8>) -> Bytes {
    let f = wallet.sign_message(&message);
    let signature = f.await.expect("Failed to sign message");
    let signature_bytes = Bytes::from(signature.to_vec());
    if signature_bytes.len() != 65 {
        panic!("Invalid signature length");
    }
    signature_bytes
}

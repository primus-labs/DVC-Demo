#![no_main]
pico_sdk::entrypoint!(main);
use anyhow::Result;
use pico_sdk::io::commit;
use serde_json::json;
use zktls_att_verification::{
    attestation_data::{AttestationConfig, AttestationData, verify_attestation_data},
    tls_data::JsonData,
};

mod errors;
use errors::{ZkErrorCode, ZktlsError};

// NOTE-1: Set your own base url(s). maybe multis
const BASE_URLS: [&str; 1] = ["https://www.okx.com/api/v5/public/instruments"];

fn verify_attestation() -> Result<(AttestationData, AttestationConfig, Vec<Vec<JsonData>>), ZktlsError> {
    let attestation_data: String = pico_sdk::io::read_as();

    //
    // 0. Make attestation config
    let v: serde_json::Value = serde_json::from_str(&attestation_data)
        .map_err(|e| zkerr!(ZkErrorCode::ParseAttestationData, e.to_string()))?;
    let attestor_addr = v
        .get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("attestor"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| zkerr!(ZkErrorCode::GetAttestorAddressFail))?;
    let attestion_confg = json!({
        "attestor_addr": attestor_addr,
        "url": BASE_URLS
    });
    commit(&attestion_confg);

    // 1. Verify the attestation
    let res = verify_attestation_data(&attestation_data, &attestion_confg.to_string())
        .map_err(|e| zkerr!(ZkErrorCode::VerifyAttestation, e.to_string()))?;

    Ok(res)
}

fn app_main() -> Result<(), ZktlsError> {
    let (attestation_data, _, messages) = verify_attestation()?;

    //
    // NOTE-2: Do some valid checks
    // Please handle it according to your actual business requirements.
    // Here is just a demonstration.
    //
    let msg_len = messages[0].len();
    let requests = attestation_data.public_data[0].attestation.request.clone();
    let requests_len = requests.len();
    ensure_zk!(requests_len == msg_len, zkerr!(ZkErrorCode::InvalidMessagesLength));

    let mut json_path = vec![];
    json_path.push("$.balances[*].asset");
    json_path.push("$.balances[*].free");
    json_path.push("$.balances[*].locked");

    let json_value = messages[0][0]
        .get_json_values(&json_path)
        .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;
    // println!("{:#?}", json_value);
    ensure_zk!(
        json_value.len() % json_path.len() == 0 && json_value.len() >= 3,
        zkerr!(ZkErrorCode::InvalidJsonValueSize)
    );

    const BASE_ASSET: &str = "ETH";
    const BASE_VALUE: f64 = 0.1;
    let mut balance = 0.0;
    let size = json_value.len() / json_path.len();
    for j in 0..size {
        let asset = json_value[j].trim_matches('"').to_ascii_uppercase();
        if asset == BASE_ASSET {
            let free: f64 = json_value[size + j].trim_matches('"').parse().unwrap_or(0.0);
            let locked: f64 = json_value[size * 2 + j].trim_matches('"').parse().unwrap_or(0.0);
            balance = free + locked;
            break;
        }
    }

    ensure_zk!(balance > BASE_VALUE, zkerr!(ZkErrorCode::Unsatisfied));
    commit(&BASE_ASSET);
    commit(&BASE_VALUE);

    Ok(())
}

pub fn main() {
    let mut code = 0;
    if let Err(e) = app_main() {
        println!("Error: {} {}", e.icode(), e.msg());
        code = e.icode();
    } else {
        println!("OK");
    }
    commit(&code);
}

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
    json_path.push("$.data[*].baseCcy");
    json_path.push("$.data[*].minSz");

    let json_value = messages[0][0]
        .get_json_values(&json_path)
        .map_err(|e| zkerr!(ZkErrorCode::GetJsonValueFail, e.to_string()))?;

    const BASE_CCY: &str = "BTC";
    let base_ccy = json_value[0].trim_matches('"').to_ascii_uppercase();
    ensure_zk!(base_ccy == BASE_CCY, zkerr!(ZkErrorCode::NotMatch));
    commit(&BASE_CCY);

    const BASE_VALUE: f64 = 0.00001;
    let min_sz = json_value[1].trim_matches('"').parse::<f64>().unwrap_or(0.0);
    ensure_zk!(min_sz > BASE_VALUE, zkerr!(ZkErrorCode::Unsatisfied));
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

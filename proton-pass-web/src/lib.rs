mod common;
mod creditcard;
mod login;
mod passkey;
mod password;
mod utils;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use common::WasmBoolDict;
use creditcard::WasmCreditCardType;
use login::WasmLogin;
use passkey::WasmCreatePasskeyData;
use passkey::{PasskeyManager, WasmGeneratePasskeyResponse, WasmResolvePasskeyChallengeResponse};
use password::{
    WasmPassphraseConfig, WasmPasswordScore, WasmPasswordScoreList, WasmPasswordScoreResult, WasmRandomPasswordConfig,
};
use proton_pass_common::password::{get_generator, PassphraseConfig, RandomPasswordConfig};

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_pass_common::library_version()
}

#[wasm_bindgen]
pub fn is_email_valid(email: String) -> bool {
    proton_pass_common::email::is_email_valid(&email)
}

#[wasm_bindgen]
pub fn twofa_domain_eligible(domain: String) -> bool {
    proton_pass_common::twofa::TwofaDomainChecker::twofa_domain_eligible(&domain)
}

#[wasm_bindgen]
pub fn twofa_domains_eligible(domains: Vec<String>) -> WasmBoolDict {
    let mut dict: HashMap<String, bool> = HashMap::new();

    for domain in domains {
        let elligible = proton_pass_common::twofa::TwofaDomainChecker::twofa_domain_eligible(&domain);
        dict.insert(domain, elligible);
    }

    WasmBoolDict(dict)
}

#[wasm_bindgen]
pub fn validate_alias_prefix(prefix: String) -> Result<(), JsError> {
    match proton_pass_common::alias_prefix::validate_alias_prefix(&prefix) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[wasm_bindgen]
pub fn validate_login_obj(login: WasmLogin) -> Result<(), JsError> {
    match proton_pass_common::login::validate_login(login.into()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[wasm_bindgen]
pub fn generate_password(config: WasmRandomPasswordConfig) -> Result<String, JsError> {
    let mut generator = get_generator();
    let cfg: RandomPasswordConfig = config.into();
    generator.generate_random(&cfg).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn random_words(word_count: u32) -> Result<Vec<String>, JsError> {
    let mut generator = get_generator();
    generator.random_words(word_count as usize).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn generate_passphrase(words: Vec<String>, config: WasmPassphraseConfig) -> Result<String, JsError> {
    let mut generator = get_generator();
    let cfg: PassphraseConfig = config.into();

    generator
        .generate_passphrase_from_words(words, &cfg)
        .map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn generate_random_passphrase(config: WasmPassphraseConfig) -> Result<String, JsError> {
    let mut generator = get_generator();
    let cfg: PassphraseConfig = config.into();

    generator.generate_passphrase(&cfg).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn analyze_password(password: String) -> WasmPasswordScoreResult {
    proton_pass_common::password::check_score(&password).into()
}

#[wasm_bindgen]
pub fn check_password_score(password: String) -> WasmPasswordScore {
    proton_pass_common::password::check_score(&password)
        .password_score
        .into()
}

#[wasm_bindgen]
pub fn check_password_scores(passwords: Vec<String>) -> WasmPasswordScoreList {
    WasmPasswordScoreList(
        passwords
            .iter()
            .map(|password| {
                proton_pass_common::password::check_score(password)
                    .password_score
                    .into()
            })
            .collect(),
    )
}

#[wasm_bindgen]
pub fn calculate_password_score(password: String) -> f64 {
    proton_pass_common::password::numeric_score(&password)
}

#[wasm_bindgen]
pub fn create_new_user_invite_signature_body(email: String, vault_key: js_sys::Uint8Array) -> js_sys::Uint8Array {
    let vault_key_as_vec = vault_key.to_vec();
    let res = proton_pass_common::invite::create_signature_body(&email, vault_key_as_vec);
    utils::vec_to_uint8_array(res)
}

#[wasm_bindgen]
pub fn detect_credit_card_type(card_number: String) -> WasmCreditCardType {
    let detector = creditcard::CreditCardDetector::default();
    let detected = detector.detect(&card_number);
    detected.into()
}

#[wasm_bindgen]
pub async fn generate_passkey(domain: String, request: String) -> Result<WasmGeneratePasskeyResponse, JsError> {
    Ok(PasskeyManager::generate_passkey(domain, request).await?)
}

#[wasm_bindgen]
pub async fn resolve_passkey_challenge(
    domain: String,
    passkey: js_sys::Uint8Array,
    request: String,
) -> Result<WasmResolvePasskeyChallengeResponse, JsError> {
    let passkey_as_vec = passkey.to_vec();
    Ok(PasskeyManager::resolve_challenge(domain, passkey_as_vec, request).await?)
}

#[wasm_bindgen]
pub fn parse_create_passkey_data(request: String) -> Result<WasmCreatePasskeyData, JsError> {
    Ok(PasskeyManager::parse_create_request(request)?)
}

#[wasm_bindgen]
pub fn get_root_domain(input: String) -> Result<String, JsError> {
    Ok(proton_pass_common::domain::get_root_domain(&input)?)
}

#[wasm_bindgen]
pub fn get_domain(input: String) -> Result<String, JsError> {
    Ok(proton_pass_common::domain::get_domain(&input)?)
}

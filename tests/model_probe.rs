#![cfg(feature = "builder")]

mod generated_model {
    pub type QVector = [i16; detecta_fraude::STORE_DIM];

    pub mod inner {
        include!("../src/model.rs");
    }
}

#[test]
fn generated_model_matches_public_profile() {
    let data = std::fs::read(".rinha-upstream/test/test-data.json")
        .expect("official public test data must be available");
    let json: serde_json::Value = serde_json::from_slice(&data).expect("valid test-data.json");
    let entries = json["entries"].as_array().expect("entries array");
    let mut mismatches = 0usize;

    for entry in entries {
        let request = serde_json::to_vec(&entry["request"]).expect("serialize request");
        let payload = detecta_fraude::parse::parse_payload(&request).expect("parse request");
        let q = detecta_fraude::vectorize::vectorize_model_q(&payload);
        let expected = entry["expected_approved"]
            .as_bool()
            .expect("expected_approved bool");
        let actual = generated_model::inner::approved(&q);
        if actual != expected {
            mismatches += 1;
        }
    }

    assert_eq!(mismatches, 0, "generated model mismatches public labels");
}

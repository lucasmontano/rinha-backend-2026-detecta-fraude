#![cfg(feature = "builder")]

use detecta_fraude::index::build::Builder;
use detecta_fraude::index::{IndexReader, LABEL_FRAUD, LABEL_LEGIT};
use detecta_fraude::{QVector, STORE_DIM};

fn vector(first_dim: i16) -> QVector {
    let mut q = [0i16; STORE_DIM];
    q[0] = first_dim;
    q
}

#[test]
fn index_returns_fraud_count_from_five_nearest_vectors() {
    let mut builder = Builder::new();
    builder.add(vector(0), LABEL_FRAUD);
    builder.add(vector(100), LABEL_FRAUD);
    builder.add(vector(200), LABEL_LEGIT);
    builder.add(vector(300), LABEL_FRAUD);
    builder.add(vector(400), LABEL_LEGIT);
    builder.add(vector(10_000), LABEL_FRAUD);

    let path = std::env::temp_dir().join(format!(
        "rinha-knn-test-{}-{}.bin",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    builder.write_to(&path).expect("write test index");

    let index = IndexReader::open(&path).expect("open test index");
    let fraud_count = index.fraud_count(&vector(0));
    let _ = std::fs::remove_file(&path);

    assert_eq!(fraud_count, 3);
}

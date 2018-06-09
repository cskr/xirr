extern crate csv;
extern crate xirr;

use xirr::*;

const MAX_ERROR: f64 = 1e-10;

#[test]
fn test_single_redemption() {
    let payments = load_payments("tests/samples/single_redemption.csv");
    let actual = compute(payments).unwrap();
    assert!((actual - 0.1361695793742).abs() <= MAX_ERROR);
}

#[test]
fn test_random() {
    let payments = load_payments("tests/samples/random.csv");
    let actual = compute(payments).unwrap();
    assert!((actual - 0.6924974337277).abs() <= MAX_ERROR);
}

#[test]
fn test_same_sign() {
    let result_negative = compute(vec![
        Payment { date: "2016-06-11".parse().unwrap(), amount: -100.0 },
        Payment { date: "2018-06-11".parse().unwrap(), amount: -200.0 }
    ]);
    assert!(result_negative.is_err());

    let result_positive = compute(vec![
        Payment { date: "2016-06-11".parse().unwrap(), amount: 100.0 },
        Payment { date: "2018-06-11".parse().unwrap(), amount: 200.0 }
    ]);
    assert!(result_positive.is_err());
}

fn load_payments(file: &str) -> Vec<Payment> {
    csv::ReaderBuilder::new().has_headers(false)
        .from_path(file).unwrap().records()
        .map(|r| r.unwrap())
        .map(|r| {
            Payment {
                date: r[1].parse().unwrap(),
                amount: r[0].parse().unwrap(),
            }
        })
        .collect()
}

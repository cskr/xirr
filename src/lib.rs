// Copyright 2018 Chandra Sekar S
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # XIRR
//!
//! `xirr` implements the XIRR function found in spreadsheet applications like LibreOffice Calc.
//!
//! # Example
//!
//! ```
//! use xirr::*;
//!
//! let xirr = compute(vec![
//!      Payment { date: "2015-06-11".parse().unwrap(), amount: -1000.0 },
//!      Payment { date: "2015-07-21".parse().unwrap(), amount: -9000.0 },
//!      Payment { date: "2015-10-17".parse().unwrap(), amount: -3000.0 },
//!      Payment { date: "2018-06-10".parse().unwrap(), amount: 20000.0 }
//!  ]);
//!
//!  assert_eq!(0.1635371584432641, xirr.unwrap());
//! ```

extern crate chrono;

use chrono::prelude::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;

const MAX_ERROR: f64 = 1e-10;

/// A payment made or received on a particular date.
///
/// `amount` must be negative for payment made and positive for payment received.
pub struct Payment {
    pub amount: f64,
    pub date: NaiveDate,
}

/// Calculates the internal rate of return of a series of irregular payments.
///
/// It tries to identify the rate of return using Newton's method with an initial guess of 0.1.
/// If that does not provide a solution, it attempts with guesses from -0.99 to 0.99
/// in increments of 0.01.
///
/// # Errors
///
/// This function will return [`InvalidPaymentsError`](struct.InvalidPaymentsError.html)
/// if both positive and negative payments are not provided.
pub fn compute(payments: Vec<Payment>) -> Result<f64, InvalidPaymentsError> {
    validate(&payments)?;

    let mut rate = compute_with_guess(&payments, 0.1);
    let mut guess = -0.99;
    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&payments, guess);
        guess += 0.01;
    }

    Ok(rate)
}

/// An error returned when the payments provided to [`compute`](fn.compute.html) do not contain
/// both negative and positive payments.
#[derive(Debug)]
pub struct InvalidPaymentsError;

impl Display for InvalidPaymentsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for InvalidPaymentsError {
    fn description(&self) -> &str {
        "negative and positive payments are required"
    }
}


fn compute_with_guess(payments: &Vec<Payment>, guess: f64) -> f64 {
    let mut r = guess;
    let mut e = 1.0;
    while e > MAX_ERROR {
        let r1 = r - xirr(payments, r) / dxirr(payments, r);
        e = (r1 - r).abs();
        r = r1
    }

    r
}

fn xirr(payments: &Vec<Payment>, rate: f64) -> f64 {
    let mut sorted: Vec<_> = payments.iter().collect();
    sorted.sort_by_key(|p| p.date);

    let mut result = 0.0;
    for p in &sorted {
        let exp = get_exp(p, sorted[0]);
        result += p.amount / (1.0 + rate).powf(exp)
    }
    result
}

fn dxirr(payments: &Vec<Payment>, rate: f64) -> f64 {
    let mut sorted: Vec<&Payment> = payments.iter().collect();
    sorted.sort_by_key(|p| p.date);

    let mut result = 0.0;
    for p in &sorted {
        let exp = get_exp(p, sorted[0]);
        result -= p.amount * exp / (1.0 + rate).powf(exp + 1.0)
    }
    result
}

fn validate(payments: &Vec<Payment>) -> Result<(), InvalidPaymentsError> {
    let positive = payments.iter().any(|p| p.amount > 0.0);
    let negative = payments.iter().any(|p| p.amount < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError)
    }
}

fn get_exp(p: &Payment, p0: &Payment) -> f64 {
    (p.date - p0.date).num_days() as f64 / 365.0
}

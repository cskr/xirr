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

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! # XIRR
//!
//! `xirr` implements the XIRR function found in spreadsheet applications like LibreOffice Calc.
//!
//! # Example
//!
//! ```
//! use jiff::civil::Date;
//! use xirr::*;
//!
//! let payments = vec![
//!     Payment { date: "2015-06-11".parse().unwrap(), amount: -1000.0 },
//!     Payment { date: "2015-07-21".parse().unwrap(), amount: -9000.0 },
//!     Payment { date: "2018-06-10".parse().unwrap(), amount: 20000.0 },
//!     Payment { date: "2015-10-17".parse().unwrap(), amount: -3000.0 },
//! ];
//!
//!  assert_eq!(0.1635371584432641, compute::<Date>(&payments).unwrap());
//! ```
//!
//! If you use chrono, enable the `chrono` feature and replace
//! [`jiff::civil::Date`](::jiff::civil::Date) with [`chrono::NaiveDate`](::chrono::NaiveDate).

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "jiff")]
mod jiff;

const MAX_ERROR: f64 = 1e-10;
const MAX_COMPUTE_WITH_GUESS_ITERATIONS: u32 = 50;

/// A payment made or received on a particular date.
///
/// `amount` must be negative for payment made and positive for payment received.
#[derive(Copy, Clone)]
pub struct Payment<T: PaymentDate> {
    pub amount: f64,
    pub date: T,
}

/// Calculates the internal rate of return of a series of irregular payments.
///
/// It tries to identify the rate of return using Newton's method with an initial guess of 0.1.
/// If that does not provide a solution, it attempts with guesses from -0.99 to 0.99
/// in increments of 0.01 and returns NaN if that fails too.
///
/// # Errors
///
/// This function will return [`InvalidPaymentsError`](struct.InvalidPaymentsError.html)
/// if both positive and negative payments are not provided.
pub fn compute<T: PaymentDate>(payments: &Vec<Payment<T>>) -> Result<f64, InvalidPaymentsError> {
    validate(payments)?;

    let mut sorted: Vec<_> = payments.iter().collect();
    sorted.sort_by_key(|p| &p.date);

    let mut rate = compute_with_guess(&sorted, 0.1);
    let mut guess = -0.99;

    while guess < 1.0 && (rate.is_nan() || rate.is_infinite()) {
        rate = compute_with_guess(&sorted, guess);
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
        "negative and positive payments are required".fmt(f)
    }
}

impl Error for InvalidPaymentsError {}

fn compute_with_guess<T: PaymentDate>(payments: &Vec<&Payment<T>>, guess: f64) -> f64 {
    let mut r = guess;
    let mut e = 1.0;

    for _ in 0..MAX_COMPUTE_WITH_GUESS_ITERATIONS {
        if e <= MAX_ERROR {
            return r;
        }

        let r1 = r - xirr(payments, r) / dxirr(payments, r);
        e = (r1 - r).abs();
        r = r1;
    }

    f64::NAN
}

fn xirr<T: PaymentDate>(payments: &Vec<&Payment<T>>, rate: f64) -> f64 {
    let mut result = 0.0;
    for p in payments {
        let exp = get_exp(p, payments[0]);
        result += p.amount / (1.0 + rate).powf(exp)
    }
    result
}

fn dxirr<T: PaymentDate>(payments: &Vec<&Payment<T>>, rate: f64) -> f64 {
    let mut result = 0.0;
    for p in payments {
        let exp = get_exp(p, payments[0]);
        result -= p.amount * exp / (1.0 + rate).powf(exp + 1.0)
    }
    result
}

fn validate<T: PaymentDate>(payments: &Vec<Payment<T>>) -> Result<(), InvalidPaymentsError> {
    let positive = payments.iter().any(|p| p.amount > 0.0);
    let negative = payments.iter().any(|p| p.amount < 0.0);

    if positive && negative {
        Ok(())
    } else {
        Err(InvalidPaymentsError)
    }
}

fn get_exp<T: PaymentDate>(p: &Payment<T>, p0: &Payment<T>) -> f64 {
    p.date.days_since(p0.date) as f64 / 365.0
}

/// A trait representing the date on which a payment was made.
///
/// This trait is implemented for [`jiff::civil::Date`](::jiff::civil::Date)
/// and [`chrono::NaiveDate`](::chrono::NaiveDate).
pub trait PaymentDate: Ord + Sized + Copy {
    /// Calculates the number days from the `other` date to this date.
    fn days_since(self, other: Self) -> i32;
}

use chrono::NaiveDate;

use crate::PaymentDate;

impl PaymentDate for NaiveDate {
    fn days_since(self, other: Self) -> i32 {
        (self - other).num_days() as i32
    }
}

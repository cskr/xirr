use jiff::civil::Date;

use crate::PaymentDate;

impl PaymentDate for Date {
    fn days_since(self, other: Self) -> i32 {
        (self - other).get_days()
    }
}

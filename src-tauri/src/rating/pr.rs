use crate::model::Totals;
use crate::rating::expected::ExpectedValues;

pub fn ship_rating(ship_id: u64, totals: Totals, expected: &ExpectedValues) -> Option<f64> {
    let _ = (ship_id, totals, expected);
    todo!("leaf: ratings")
}

pub fn personal_rating(ships: &[(u64, Totals)], expected: &ExpectedValues) -> Option<f64> {
    let _ = (ships, expected);
    todo!("leaf: ratings")
}

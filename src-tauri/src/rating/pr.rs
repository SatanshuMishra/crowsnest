use crate::model::Totals;
use crate::rating::expected::ExpectedValues;

const DAMAGE_WEIGHT: f64 = 700.0;
const FRAGS_WEIGHT: f64 = 300.0;
const WINS_WEIGHT: f64 = 150.0;

#[derive(Debug, Clone, Copy, Default)]
struct Sums {
    damage: f64,
    frags: f64,
    wins: f64,
    expected_damage: f64,
    expected_frags: f64,
    expected_wins: f64,
}

impl Sums {
    fn plus(self, other: Sums) -> Sums {
        Sums {
            damage: self.damage + other.damage,
            frags: self.frags + other.frags,
            wins: self.wins + other.wins,
            expected_damage: self.expected_damage + other.expected_damage,
            expected_frags: self.expected_frags + other.expected_frags,
            expected_wins: self.expected_wins + other.expected_wins,
        }
    }

    fn rating(self) -> Option<f64> {
        if self.expected_damage <= 0.0 || self.expected_frags <= 0.0 || self.expected_wins <= 0.0 {
            return None;
        }
        let damage = normalise(self.damage / self.expected_damage, 0.4);
        let frags = normalise(self.frags / self.expected_frags, 0.1);
        let wins = normalise(self.wins / self.expected_wins, 0.7);
        Some(DAMAGE_WEIGHT * damage + FRAGS_WEIGHT * frags + WINS_WEIGHT * wins)
    }
}

fn normalise(ratio: f64, floor: f64) -> f64 {
    ((ratio - floor) / (1.0 - floor)).max(0.0)
}

fn ship_sums(ship_id: u64, totals: Totals, expected: &ExpectedValues) -> Option<Sums> {
    if totals.battles == 0 {
        return None;
    }
    let values = expected.ships.get(&ship_id)?;
    let battles = totals.battles as f64;
    Some(Sums {
        damage: totals.damage as f64,
        frags: totals.frags as f64,
        wins: totals.wins as f64,
        expected_damage: battles * values.damage,
        expected_frags: battles * values.frags,
        expected_wins: battles * values.win_rate / 100.0,
    })
}

pub fn ship_rating(ship_id: u64, totals: Totals, expected: &ExpectedValues) -> Option<f64> {
    ship_sums(ship_id, totals, expected)?.rating()
}

pub fn personal_rating(ships: &[(u64, Totals)], expected: &ExpectedValues) -> Option<f64> {
    ships
        .iter()
        .filter_map(|(ship_id, totals)| ship_sums(*ship_id, *totals, expected))
        .reduce(Sums::plus)?
        .rating()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rating::expected::parse_expected;

    fn fixture() -> ExpectedValues {
        ExpectedValues {
            ships: parse_expected(include_str!("fixtures/expected.json")).unwrap(),
        }
    }

    fn totals(battles: u64, wins: u64, damage: u64, frags: u64) -> Totals {
        Totals {
            battles,
            wins,
            damage,
            frags,
            ..Totals::default()
        }
    }

    fn assert_close(actual: Option<f64>, expected: f64) {
        let actual = actual.expect("a rating");
        assert!((actual - expected).abs() < 0.01, "{actual} vs {expected}");
    }

    #[test]
    fn single_ship_matches_hand_computation() {
        let rating = ship_rating(1000, totals(10, 6, 1_200_000, 15), &fixture());
        assert_close(rating, 1650.0);
    }

    #[test]
    fn two_ship_account_matches_hand_computation() {
        let ships = [
            (1000, totals(10, 6, 1_200_000, 15)),
            (2000, totals(20, 6, 800_000, 6)),
        ];
        assert_close(personal_rating(&ships, &fixture()), 1128.205);
    }

    #[test]
    fn exactly_expected_scores_1150() {
        let rating = ship_rating(1000, totals(100, 50, 10_000_000, 100), &fixture());
        assert_close(rating, 1150.0);
    }

    #[test]
    fn poor_play_is_floored_at_zero() {
        assert_close(ship_rating(1000, totals(10, 0, 0, 0), &fixture()), 0.0);
    }

    #[test]
    fn zero_battles_gives_none() {
        assert_eq!(ship_rating(1000, totals(0, 0, 0, 0), &fixture()), None);
        assert_eq!(
            personal_rating(&[(1000, totals(0, 0, 0, 0))], &fixture()),
            None
        );
    }

    #[test]
    fn ships_without_expected_values_are_skipped() {
        let with_unknown = [
            (1000, totals(10, 6, 1_200_000, 15)),
            (3000, totals(50, 25, 1, 1)),
            (9999, totals(50, 25, 1, 1)),
        ];
        assert_close(personal_rating(&with_unknown, &fixture()), 1650.0);
        assert_eq!(ship_rating(3000, totals(5, 1, 1, 1), &fixture()), None);
    }

    #[test]
    fn nothing_qualifying_gives_none() {
        assert_eq!(personal_rating(&[], &fixture()), None);
        assert_eq!(
            personal_rating(&[(9999, totals(5, 1, 1, 1))], &fixture()),
            None
        );
    }
}

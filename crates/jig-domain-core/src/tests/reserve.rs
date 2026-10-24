use super::*;

#[test]
fn most_costly_matches_smiths_rounding_on_a_large_window() {
    let mut model = config(1).defaults.charters[0].policy.clone().model;
    model.window = 200_000;
    model.output = 8_192;
    model.input_price = 3;
    model.cached_price = 1;
    model.output_price = 15;
    model.price_unit = 1_000;
    assert_eq!(run::most_costly(&model), Some(723));
}

#[test]
fn most_costly_takes_the_highest_input_rate() {
    let mut model = config(1).defaults.charters[0].policy.clone().model;
    model.window = 10;
    model.output = 1;
    model.input_price = 2;
    model.cached_price = 3;
    model.cache_write_price = Some(4);
    model.output_price = 1;
    model.price_unit = 1;
    assert_eq!(run::most_costly(&model), Some(41));
}

#[test]
fn reserve_floor_counts_the_reserve_and_one_completion_more() {
    let mut policy = config(1).defaults.charters[0].policy.clone();
    policy.reserve_turns = 2;
    assert_eq!(reserve_floor(&policy), Some(3));
}

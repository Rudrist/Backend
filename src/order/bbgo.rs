use rand::{thread_rng, Rng};
#[allow(dead_code)]
pub fn handle_order(
    _base: &String,
    _quote: &String,
    _order_type: &String,
    _price: &String,
    _quantity: &String,
) -> i32 {
    let mut rng = thread_rng();

    return rng.gen_range(0..1000000);
}

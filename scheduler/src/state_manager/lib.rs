use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn get_random_hash(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

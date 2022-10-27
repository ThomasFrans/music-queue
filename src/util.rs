use rand::thread_rng;
use rand::seq::SliceRandom;

pub fn shuffled_vec(size: usize) -> Vec<usize> {
    let mut vec: Vec<usize> = (0..size).collect();
    vec.shuffle(&mut thread_rng());
    vec
}

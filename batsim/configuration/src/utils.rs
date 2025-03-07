use rand::Rng;

pub fn sample_p(p: Option<f32>, rng: &mut impl Rng) -> bool {
    match p {
        None => true,
        Some(p) if p > rng.gen() => true,
        _ => false,
    }
}

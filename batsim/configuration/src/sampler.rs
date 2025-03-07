use rand::rngs::SmallRng;
use rand::SeedableRng;

pub fn new(seed: Option<u64>) -> SmallRng {
    match seed {
        None => SmallRng::from_entropy(),
        Some(seed) => SmallRng::seed_from_u64(seed),
    }
}

#[cfg(test)]
mod tests {

    use rand::Rng;

    use super::*;

    #[test]
    fn sample_no_seed() {
        let mut rng = new(None);
        let _n: f32 = rng.gen();
    }
    #[test]
    fn sample_with_seed() {
        let mut rng = new(Some(1234));
        let _n: f32 = rng.gen();
    }
    #[test]
    fn sample_consistently_with_seed() {
        let mut rng_a = new(Some(1234));
        let mut rng_b = new(Some(1234));
        for _ in 0..10 {
            assert_eq!(rng_a.gen::<f32>(), rng_b.gen::<f32>());
        }
    }
}

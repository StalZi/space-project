use rand::RngExt;
use space_engine::utils::Cube;

pub fn generate_random_cubes(count: usize) -> Vec<Cube> {
    let mut rng = rand::rng();
    (0..count)
        .map(|_| {
            Cube::default()
                .position(
                    rng.random_range(-50.0..50.0),
                    rng.random_range(-50.0..50.0),
                    rng.random_range(-50.0..50.0),
                )
                .rotation(
                    rng.random_range(0.0..360.0),
                    rng.random_range(0.0..360.0),
                    rng.random_range(0.0..360.0),
                )
                .size(
                    rng.random_range(0.5..5.0),
                    rng.random_range(0.5..5.0),
                    rng.random_range(0.5..5.0),
                )
                .color(
                    rng.random_range(0.0..=255.0),
                    rng.random_range(0.0..=255.0),
                    rng.random_range(0.0..=255.0),
                    255.0,
                )
        })
        .collect()
}

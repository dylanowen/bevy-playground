use rand::Rng;

pub struct Chunk<const WIDTH: usize, const HEIGHT: usize> {
    pub grid: [[bool; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> Chunk<WIDTH, HEIGHT> {
    pub fn random<R: Rng>(rng: &mut R) -> Chunk<WIDTH, HEIGHT> {
        let mut grid = [[false; WIDTH]; HEIGHT];
        #[allow(clippy::needless_range_loop)]
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                grid[y][x] = rng.gen_bool(0.5);
            }
        }

        Chunk { grid }
    }
}

use rand::Rng;

pub struct Chunk<const WIDTH: usize, const HEIGHT: usize> {
    pub grid: [[bool; WIDTH]; HEIGHT],
}

#[allow(clippy::needless_range_loop)]
impl<const WIDTH: usize, const HEIGHT: usize> Chunk<WIDTH, HEIGHT> {
    pub fn random<R: Rng>(rng: &mut R) -> Chunk<WIDTH, HEIGHT> {
        let mut grid = [[false; WIDTH]; HEIGHT];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                grid[y][x] = rng.gen_bool(0.5);
            }
        }

        Chunk { grid }
    }

    pub fn arena() -> Chunk<WIDTH, HEIGHT> {
        let mut grid = [[true; WIDTH]; HEIGHT];
        for x in 0..WIDTH {
            grid[x][0] = false;
            grid[x][HEIGHT - 1] = false;
        }
        for y in 0..HEIGHT {
            grid[0][y] = false;
            grid[WIDTH - 1][y] = false;
        }

        Chunk { grid }
    }
}

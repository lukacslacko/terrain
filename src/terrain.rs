pub fn height_map(w: usize, h: usize) -> Vec<Vec<f32>> {
    // Create a height map with Perlin noise.
    let mut height_map = vec![vec![0.0; w]; h];
    let perlin = Perlin::new(42, 0.1, 2.0, 0.5);
    for y in 0..h {
        for x in 0..w {
            let nx = x as f32 / w as f32 - 0.5;
            let ny = y as f32 / h as f32 - 0.5;
            height_map[y][x] = perlin.noise(nx, ny);
        }
    }
    height_map
}
struct Perlin {
    seed: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
}

impl Perlin {
    fn new(seed: u32, frequency: f32, lacunarity: f32, persistence: f32) -> Self {
        Perlin {
            seed,
            frequency,
            lacunarity,
            persistence,
        }
    }

    fn noise(&self, x: f32, y: f32) -> f32 {
        // Generate multi-layered Perlin noise based on the input coordinates.
        let mut total = 0.0;
        let mut frequency = self.frequency;
        let mut amplitude = self.persistence;

        for _ in 0..4 {
            total += self.noise(x * frequency, y * frequency) * amplitude;
            frequency *= self.lacunarity;
            amplitude *= self.persistence;
        }

        total
    }
}

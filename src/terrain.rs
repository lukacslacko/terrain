pub fn height_map(w: usize, h: usize) -> Vec<Vec<f32>> {
    let mut height_map = vec![vec![0.0; w]; h];
    let perlin = Perlin {
        seed: 126,
        frequency: 4.0,
        lacunarity: 2.13,
        persistence: 0.3,
        octaves: 4,
    };
    for y in 0..h {
        for x in 0..w {
            let nx = x as f32 / w.max(h) as f32 - 0.5;
            let ny = y as f32 / w.max(h) as f32 - 0.5;
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
    octaves: u32,
}

impl Perlin {
    fn noise(&self, x: f32, y: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = self.frequency;
        let mut amplitude = self.persistence;

        for _ in 0..self.octaves {
            total += self.single_layer(x * frequency, y * frequency) * amplitude;
            frequency *= self.lacunarity;
            amplitude *= self.persistence;
        }

        total
    }

    fn single_layer(&self, x: f32, y: f32) -> f32 {
        // Generate a single layer of Perlin noise.
        let x = x + self.seed as f32;
        let y = y + self.seed as f32;

        let xi = x.floor() as i32;
        let yi = y.floor() as i32;

        let xf = x - xi as f32;
        let yf = y - yi as f32;

        use std::f32::consts::PI;

        let fade = |x: f32| 3.0 * x * x - 2.0 * x * x * x;
        let grad = |h: f32, x, y| x * (2.0 * PI * h).cos() + y * (2.0 * PI * h).sin();
        let lerp = |t: f32, a: f32, b: f32| a + t * (b - a);

        let hash = |x: i32, y: i32| {
            let mut a = x as u32;
            let mut b = y as u32;
            for _ in 0..5 {
                a = a.wrapping_add(0x9e3779b9);
                b = b.wrapping_add(0x5e3719b9);
                a = a.wrapping_mul(0x2bd1e995);
                b = b.wrapping_mul(0x54d1e935);
                a ^= a >> 11;
                b ^= b >> 13;
                (a, b) = (b, (a ^ b));
            }
            b as f32 / u32::MAX as f32
        };

        let aa = hash(xi, yi);
        let ab = hash(xi, yi + 1);
        let ba = hash(xi + 1, yi);
        let bb = hash(xi + 1, yi + 1);

        let u = fade(xf);
        let v = fade(yf);
        lerp(
            v,
            lerp(u, grad(aa, xf, yf), grad(ba, xf - 1.0, yf)),
            lerp(u, grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0)),
        )
    }
}

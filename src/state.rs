use crate::dijkstra::{Dijkstra, DijkstraUpdate};
use crate::terrain::height_map;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct ImageHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct GameTime {
    pub _time: f32,
}

#[derive(Resource)]
pub struct MapState {
    pub dijkstra: Dijkstra,
    stations: HashSet<(usize, usize)>,
    min_height: f32,
    max_height: f32,
}

impl MapState {
    pub fn new(width: usize, height: usize) -> Self {
        let mut height_map = height_map(width, height);
        let mut points_with_height = Vec::new();
        let mut lake_id: Vec<Vec<usize>> = vec![vec![0; width]; height];
        let mut actual_lake_id = HashMap::new();
        let mut overflown_lakes = HashSet::new();
        actual_lake_id.insert(0, 0);
        let mut lake_level = HashMap::new();
        let mut next_lake_id = 1;
        let mut is_water = vec![vec![false; width]; height];
        for col in 0..width {
            for row in 0..height {
                points_with_height.push((height_map[row][col], row, col));
            }
        }
        points_with_height.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let is_edge =
            |row: usize, col: usize| row == 0 || col == 0 || row == height - 1 || col == width - 1;
        let mut points_handled = 0;
        for (h, row, col) in points_with_height {
            points_handled += 1;
            let neighbor_lake_ids = [
                (row.saturating_sub(1), col),
                ((row + 1).min(height - 1), col),
                (row, col.saturating_sub(1)),
                (row, (col + 1).min(width - 1)),
            ]
            .iter()
            .filter(|&(r, c)| height_map[*r][*c] < h)
            .map(|&(r, c)| actual_lake_id[&lake_id[r][c]])
            .collect::<HashSet<_>>();
            if is_edge(row, col) {
                lake_id[row][col] = 0;
                for lake_id in neighbor_lake_ids.iter() {
                    if *lake_id == 0 {
                        continue;
                    }
                    overflown_lakes.insert(*lake_id);
                }
                continue;
            }
            if neighbor_lake_ids.is_empty() {
                lake_id[row][col] = next_lake_id;
                is_water[row][col] = true;
                println!("Lakes: {}", next_lake_id);
                actual_lake_id.insert(next_lake_id, next_lake_id);
                lake_level.insert(next_lake_id, h);
                next_lake_id += 1;
                continue;
            }
            if neighbor_lake_ids.contains(&0) {
                let mut river = false;
                for other_lake_id in neighbor_lake_ids.iter() {
                    if *other_lake_id == 0 {
                        continue;
                    }
                    if overflown_lakes.contains(other_lake_id) {
                        continue;
                    }
                    overflown_lakes.insert(*other_lake_id);
                    river = true;
                }
                if river {
                    let mut r = row;
                    let mut c = col;
                    while !is_edge(r, c) {
                        is_water[r][c] = true;
                        // let neighbors = [
                        //     (r.saturating_sub(1), c),
                        //     ((r + 1).min(height - 1), c),
                        //     (r, c.saturating_sub(1)),
                        //     (r, (c + 1).min(width - 1)),
                        // ];
                        let mut neighbors = Vec::new();
                        for dr in -1..=1 {
                            for dc in -1..=1 {
                                neighbors
                                    .push(((r as isize + dr) as usize, (c as isize + dc) as usize));
                            }
                        }
                        let non_same_lake_lower_neighbors = neighbors
                            .iter()
                            .filter(|&&(nr, nc)| {
                                height_map[nr][nc] < height_map[r][c]
                                    && (actual_lake_id[&lake_id[nr][nc]] == 0
                                        || !neighbor_lake_ids
                                            .contains(&actual_lake_id[&lake_id[nr][nc]]))
                            })
                            .collect::<Vec<_>>();
                        if non_same_lake_lower_neighbors.is_empty() {
                            break;
                        }
                        let lowest_neighbor = non_same_lake_lower_neighbors
                            .iter()
                            .map(|&&(nr, nc)| {
                                (
                                    (height_map[nr][nc] - height_map[r][c])
                                        / ((nr as f32 - r as f32) * (nr as f32 - r as f32)
                                            + (nc as f32 - c as f32) * (nc as f32 - c as f32))
                                            .sqrt(),
                                    nr,
                                    nc,
                                )
                            })
                            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                            .unwrap();
                        r = lowest_neighbor.1;
                        c = lowest_neighbor.2;
                    }
                }
                lake_id[row][col] = 0;
                continue;
            }
            let non_overflow_neighbor_lake_ids = neighbor_lake_ids
                .iter()
                .filter(|&&x| !overflown_lakes.contains(&x))
                .cloned()
                .collect::<HashSet<_>>();
            if non_overflow_neighbor_lake_ids.is_empty() {
                lake_id[row][col] = 0;
                continue;
            }
            // This point is part of one ore more lakes.
            let smallest_lake_id = non_overflow_neighbor_lake_ids
                .iter()
                .cloned()
                .min()
                .unwrap();
            lake_id[row][col] = smallest_lake_id;
            is_water[row][col] = true;
            lake_level.insert(smallest_lake_id, h);
            for other_lake_id in non_overflow_neighbor_lake_ids {
                if other_lake_id == smallest_lake_id {
                    continue;
                }
                actual_lake_id.insert(other_lake_id, smallest_lake_id);
            }
        }

        for row in 0..height {
            for col in 0..width {
                if lake_id[row][col] != 0 {
                    height_map[row][col] = lake_level[&actual_lake_id[&lake_id[row][col]]];
                }
            }
        }

        let min_height = height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::min)
            .unwrap();
        let max_height = height_map
            .iter()
            .flatten()
            .cloned()
            .reduce(f32::max)
            .unwrap();
        let dijkstra = Dijkstra {
            width,
            height,
            height_map,
            is_water,
            road_level: vec![vec![0; width]; height],
            house_level: vec![vec![0; width]; height],
        };
        MapState {
            dijkstra,
            min_height,
            max_height,
            stations: HashSet::new(),
        }
    }

    pub fn process_dijsktra_update(&mut self, update: &DijkstraUpdate, image: &mut Image) {
        for (start, end) in update.path.iter() {
            let dist = (((start.0 as isize - end.0 as isize).pow(2)
                + (start.1 as isize - end.1 as isize).pow(2)) as f32)
                .sqrt();
            let height_diff =
                self.dijkstra.height_map[start.0][start.1] - self.dijkstra.height_map[end.0][end.1];
            let steepness = 2000.0 * (height_diff / dist).abs();
            let (r,g,b) = match steepness {
                0.0..=0.5 => (255, 255, 255),
                0.5..=1.0 => (255, 128, 0),
                1.0..=2.0 => (255, 0, 0),
                _ => (255, 0, 255),
            };
            let (mut row, mut col) = *start;
            loop {
                self.dijkstra.road_level[row][col] += 1;
                let pixel = image
                    .pixel_bytes_mut(UVec3::new(col as u32, row as u32, 0))
                    .unwrap();
                pixel[0] = r as u8;
                pixel[1] = g as u8;
                pixel[2] = b as u8;
                if (row, col) == *end {
                    break;
                }
                row = (row as isize + (end.0 as isize - row as isize).signum()) as usize;
                col = (col as isize + (end.1 as isize - col as isize).signum()) as usize;
            }
        }
        for &(row, col) in update.houses.iter() {
            self.stations.insert((row, col));
        }
        for &(row, col) in self.stations.iter() {
            for dr in -4..=4 {
                for dc in -4..=4 {
                    if dr * dr + dc * dc > 16 {
                        continue;
                    }
                    let r =
                        (row as isize + dr).clamp(0, self.dijkstra.height as isize - 1) as usize;
                    let c = (col as isize + dc).clamp(0, self.dijkstra.width as isize - 1) as usize;
                    let pixel = image
                        .pixel_bytes_mut(UVec3::new(c as u32, r as u32, 0))
                        .unwrap();
                    pixel[0] = 255;
                    pixel[1] = 0;
                    pixel[2] = 0;
                }
            }
        }
    }

    pub fn near_station(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        let mut closest = None;
        let mut closest_distance = 100;
        for (station_row, station_col) in self.stations.iter() {
            let distance_squared =
                station_row.abs_diff(row).pow(2) + station_col.abs_diff(col).pow(2);
            if distance_squared < closest_distance {
                closest_distance = distance_squared;
                closest = Some((*station_row, *station_col));
            }
        }
        closest
    }

    pub fn render_image(&self, image: &mut Image) {
        for i in 0..self.dijkstra.width {
            for j in 0..self.dijkstra.height {
                let pixel = image
                    .pixel_bytes_mut(UVec3::new(i as u32, j as u32, 0))
                    .unwrap();
                let level = |x: f32| (x * 30.0).floor();
                let value = (self.dijkstra.height_map[j][i] - self.min_height)
                    / (self.max_height - self.min_height);
                let value_north = if j > 0 {
                    (self.dijkstra.height_map[j - 1][i] - self.min_height)
                        / (self.max_height - self.min_height)
                } else {
                    value
                };
                let shadow = 30.0 * (value - value_north) + 1.0;
                let should_draw_level_lines = true;
                let lerp = |a: i32, b: i32, t: f32, s: f32| {
                    (shadow * s * (a as f32 + t * (b as f32 - a as f32))).clamp(0.0, 255.0) as u8
                };
                let rgb = |v, s| (lerp(0, 100, v, s), lerp(150, 50, v, s), lerp(0, 50, v, s));
                if self.dijkstra.is_water[j][i] {
                    pixel[0] = 0;
                    pixel[1] = 0;
                    pixel[2] = 255;
                } else if should_draw_level_lines
                    && i + 1 < self.dijkstra.width
                    && j + 1 < self.dijkstra.height
                    && (level(self.dijkstra.height_map[j][i])
                        != level(self.dijkstra.height_map[j + 1][i])
                        || level(self.dijkstra.height_map[j][i])
                            != level(self.dijkstra.height_map[j][i + 1]))
                {
                    let (r, g, b) = rgb(value, 0.85);
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                } else {
                    // use std::f32::consts::PI;
                    // let value = 2.0
                    //     * PI
                    //     * ((self.height_map[j][i] - self.min_height) / (self.max_height - self.min_height));
                    // pixel[0] = ((value.sin() + 1.0) * 127.5) as u8;
                    // pixel[1] = (((value + 2.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;
                    // pixel[2] = (((value + 4.0 * PI / 3.0).sin() + 1.0) * 127.5) as u8;

                    let (r, g, b) = rgb(value, 1.0);
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                }
            }
        }
    }
}

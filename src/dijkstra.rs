use crossbeam_channel::Sender;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Dijkstra {
    pub width: usize,
    pub height: usize,
    pub height_map: Vec<Vec<f32>>,
    pub is_water: Vec<Vec<bool>>,
    pub road_level: Vec<Vec<i32>>,
    pub house_level: Vec<Vec<i32>>,
}

pub struct DijkstraUpdate {
    pub path: Vec<(usize, usize)>,
    pub houses: Vec<(usize, usize)>,
}

impl Dijkstra {
    pub fn connect(&mut self, tx: Sender<DijkstraUpdate>) {
        let mut rng = rand::rng();
        let mut houses = HashSet::new();
        for _ in 0..5 {
            houses.insert(loop {
                let r = rng.random_range(0..self.height);
                let c = rng.random_range(0..self.width);
                if self.is_water[r][c] {
                    continue;
                }
                break (r, c);
            });
        }
        loop {
            let path = self.connect_once(
                *houses.iter().choose(&mut rng).unwrap(),
                *houses.iter().choose(&mut rng).unwrap(),
                &tx,
            );
            if path.is_empty() {
                continue;
            }
            for _ in 0..1 {
                let midpoint = path.choose(&mut rng).unwrap();
                let mut maybe_new_houses = Vec::new();
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let inr = midpoint.0 as isize + dr;
                        let inc = midpoint.1 as isize + dc;
                        if inr < 0
                            || inc < 0
                            || inr >= self.height as isize
                            || inc >= self.width as isize
                        {
                            continue;
                        }
                        let nr = inr as usize;
                        let nc = inc as usize;
                        if !self.is_water[nr][nc]
                            && self.house_level[nr][nc] == 0
                            && self.road_level[nr][nc] == 0
                        {
                            maybe_new_houses.push((nr, nc));
                        }
                    }
                }
                if maybe_new_houses.is_empty() {
                    continue;
                }
                let new_house = maybe_new_houses.choose(&mut rng).unwrap().clone();
                houses.insert(new_house);
                self.house_level[new_house.0][new_house.1] = 1;
            }
            let random_house = houses.iter().choose(&mut rng).unwrap().clone();
            self.house_level[random_house.0][random_house.1] = 0;
            houses.remove(&random_house);
            let moved_house = loop {
                let r = rng.random_range(0..self.height);
                let c = rng.random_range(0..self.width);
                if self.is_water[r][c] {
                    continue;
                }
                break (r, c);
            };
            self.house_level[moved_house.0][moved_house.1] = 1;
            houses.insert(moved_house);
        }
    }

    fn connect_once(
        &mut self,
        a: (usize, usize),
        b: (usize, usize),
        tx: &Sender<DijkstraUpdate>,
    ) -> Vec<(usize, usize)> {
        if self.is_water[a.0][a.1] || self.is_water[b.0][b.1] {
            return Vec::new();
        }
        if a == b {
            return Vec::new();
        }
        println!("Connecting {:?} to {:?}", a, b);
        let cost_of_step_on_road = OrderedFloat(1.0);
        let cost_of_build_road = OrderedFloat(100.0);
        let cost_of_build_bridge = OrderedFloat(1000.0);
        let cost_of_climb_multiplier = OrderedFloat(3000.0);

        let mut dist = HashMap::new();
        let mut come_from = HashMap::new();
        let mut visited = HashSet::new();
        let mut queue = PriorityQueue::new();

        dist.insert(a, OrderedFloat(0.0));
        queue.push(a, OrderedFloat(0.0));

        while let Some((current, current_dist)) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            if current == b {
                break;
            }
            let mut neighbors = Vec::new();
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let factor = ((dr as f32) * (dr as f32) + (dc as f32) * (dc as f32)).sqrt();
                    let inr = current.0 as isize + dr;
                    let inc = current.1 as isize + dc;
                    if inr < 0
                        || inc < 0
                        || inr >= self.height as isize
                        || inc >= self.width as isize
                    {
                        continue;
                    }
                    let nr = inr as usize;
                    let nc = inc as usize;
                    if self.house_level[nr][nc] != 0 && b != (nr, nc) {
                        continue;
                    }
                    let mut steepness_cost = OrderedFloat(
                        (self.height_map[nr][nc] - self.height_map[current.0][current.1]).abs(),
                    ) * cost_of_climb_multiplier * factor;
                    steepness_cost = steepness_cost * steepness_cost;
                    if self.road_level[nr][nc] != 0 {
                        neighbors.push((cost_of_step_on_road * factor + steepness_cost, (nr, nc)));
                        continue;
                    }
                    if self.is_water[nr][nc] {
                        neighbors.push((cost_of_build_bridge * factor + steepness_cost, (nr, nc)));
                        continue;
                    }
                    neighbors.push((cost_of_build_road * factor + steepness_cost, (nr, nc)));
                }
            }
            for (cost, neighbor) in neighbors {
                let new_dist = current_dist - cost;
                if new_dist > *dist.get(&neighbor).unwrap_or(&OrderedFloat(f32::MIN)) {
                    dist.insert(neighbor, new_dist);
                    come_from.insert(neighbor, current);
                    queue.push(neighbor, new_dist);
                }
            }
        }
        let mut curr = b;
        let mut path = Vec::new();
        while let Some((r, c)) = come_from.get(&curr) {
            if *r == a.0 && *c == a.1 {
                break;
            }
            path.push((*r, *c));
            curr = (*r, *c);
            self.road_level[*r][*c] = 1;
        }
        self.house_level[a.0][a.1] = 1;
        self.house_level[b.0][b.1] = 1;
        self.road_level[a.0][a.1] = 0;
        self.road_level[b.0][b.1] = 0;
        println!("Path length: {}", path.len());
        let _ = tx.send(DijkstraUpdate {
            path: path.clone(),
            houses: vec![a, b],
        });
        return path;
    }
}

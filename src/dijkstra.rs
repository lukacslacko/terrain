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
    pub fn connect(&self, tx: Sender<DijkstraUpdate>) {
        let mut rng = rand::rng();
        loop {
            self.connect_once(
                (
                    rng.random_range(0..self.height),
                    rng.random_range(0..self.width),
                ),
                (
                    rng.random_range(0..self.height),
                    rng.random_range(0..self.width),
                ),
                &tx,
            );
        }
    }

    fn connect_once(&self, a: (usize, usize), b: (usize, usize), tx: &Sender<DijkstraUpdate>) {
        println!("Connecting {:?} to {:?}", a, b);
        let cost_of_step_on_road = OrderedFloat(1.0);
        let cost_of_build_road = OrderedFloat(10.0);
        let cost_of_build_bridge = OrderedFloat(100.0);
        let cost_of_climb_multiplier = OrderedFloat(10000.0);

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
                    let steepness_cost = OrderedFloat(
                        (self.height_map[nr][nc] - self.height_map[current.0][current.1]).abs(),
                    ) * cost_of_climb_multiplier;
                    if self.road_level[nr][nc] != 0 {
                        neighbors.push((cost_of_step_on_road + steepness_cost, (nr, nc)));
                        continue;
                    }
                    if self.is_water[nr][nc] {
                        neighbors.push((cost_of_build_bridge + steepness_cost, (nr, nc)));
                        continue;
                    }
                    neighbors.push((cost_of_build_road + steepness_cost, (nr, nc)));
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
            path.push((*r, *c));
            if *r == a.0 && *c == a.1 {
                break;
            }
            curr = (*r, *c);
        }
        println!("Path length: {}", path.len());
        let _ = tx.send(DijkstraUpdate {
            path,
            houses: vec![],
        });
    }
}

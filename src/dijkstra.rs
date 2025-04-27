use crossbeam_channel::{Receiver, Sender};
use ordered_float::{OrderedFloat, Pow};
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

#[derive(Clone)]
pub struct DijkstraCommand {
    pub a: (usize, usize),
    pub b: (usize, usize),
}

pub struct DijkstraUpdate {
    pub path: Vec<((usize, usize), (usize, usize))>,
    pub houses: Vec<(usize, usize)>,
}

impl Dijkstra {
    pub fn connect_selected(
        &mut self,
        command_rx: &Receiver<DijkstraCommand>,
        tx: Sender<DijkstraUpdate>,
    ) {
        while let Ok(command) = command_rx.recv() {
            println!("Received command to connect {:?} to {:?}", command.a, command.b);
            let path = self.connect_once(command.a, &vec![&command.b], &tx);
            if path.is_empty() {
                continue;
            }
            for (a, b) in path.iter() {
                self.road_level[a.0][a.1] = 1;
                self.road_level[b.0][b.1] = 1;
            }
        }
    }

    pub fn connect_randoms_forever(&mut self, tx: Sender<DijkstraUpdate>) {
        let mut rng = rand::rng();
        let mut houses = HashSet::new();
        let mut lakeside_points = Vec::new();
        for r in 5..(self.height - 5) {
            for c in 5..(self.width - 5) {
                if self.is_water[r][c] {
                    continue;
                }
                'outer: for dr in -5..=5 {
                    for dc in -5..=5 {
                        if r as isize + dr < 0
                            || c as isize + dc < 0
                            || r as isize + dr >= self.height as isize
                            || c as isize + dc >= self.width as isize
                        {
                            continue;
                        }
                        let nr = r as isize + dr;
                        let nc = c as isize + dc;
                        if self.is_water[nr as usize][nc as usize] {
                            lakeside_points.push((r, c));
                            break 'outer;
                        }
                    }
                }
            }
        }
        lakeside_points.shuffle(&mut rng);
        const PATHS_AT_ONCE: usize = 1;
        for _ in 0..2 * PATHS_AT_ONCE {
            houses.insert(lakeside_points.pop().unwrap());
        }
        loop {
            let path = self.connect_once(
                *houses.iter().choose(&mut rng).unwrap(),
                &houses.iter().choose_multiple(&mut rng, PATHS_AT_ONCE),
                &tx,
            );
            // let path = self.connect_once(
            //     loop {
            //         let r = rng.random_range(0..self.height);
            //         let c = rng.random_range(0..self.width);
            //         if self.is_water[r][c] {
            //             continue;
            //         }
            //         break (r, c);
            //     },
            //     &vec![&loop {
            //         let r = rng.random_range(0..self.height);
            //         let c = rng.random_range(0..self.width);
            //         if self.is_water[r][c] {
            //             continue;
            //         }
            //         break (r, c);
            //     }],
            //     &tx,
            // );
            if path.is_empty() {
                continue;
            }
            for _ in 0..PATHS_AT_ONCE {
                let midpoint = path.choose(&mut rng).unwrap();
                let mut maybe_new_houses = Vec::new();
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let inr = midpoint.1.0 as isize + dr;
                        let inc = midpoint.1.1 as isize + dc;
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
            for _ in 0..PATHS_AT_ONCE {
                let random_house = houses.iter().choose(&mut rng).unwrap().clone();
                self.house_level[random_house.0][random_house.1] = 0;
                houses.remove(&random_house);
                let moved_house = lakeside_points.pop().unwrap();
                self.house_level[moved_house.0][moved_house.1] = 1;
                houses.insert(moved_house);
            }
        }
    }

    fn connect_once(
        &mut self,
        a: (usize, usize),
        b: &Vec<&(usize, usize)>,
        tx: &Sender<DijkstraUpdate>,
    ) -> Vec<((usize, usize), (usize, usize))> {
        if self.is_water[a.0][a.1] {
            return Vec::new();
        }
        let good_targets = b
            .iter()
            .filter(|&&b| !self.is_water[b.0][b.1])
            .map(|&&b| b)
            .collect::<HashSet<_>>();
        let mut targets_connected = HashSet::new();
        println!("Connecting {:?} to {:?}", a, b);
        let cost_of_step_on_road = OrderedFloat(1.0);
        let cost_of_build_road = OrderedFloat(10.0);
        let cost_of_build_bridge = OrderedFloat(30.0);
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
            if good_targets.contains(&current) {
                targets_connected.insert(current);
                if targets_connected.len() == good_targets.len() {
                    break;
                }
            }
            let mut neighbors = Vec::new();
            for dr in -4..=4 {
                for dc in -4..=4 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    if dr * dr + dc * dc > 16 {
                        continue;
                    }
                    let factor = ((dr as f32) * (dr as f32) + (dc as f32) * (dc as f32)).pow(0.4);
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
                    if self.house_level[nr][nc] != 0 && !good_targets.contains(&(nr, nc)) {
                        continue;
                    }
                    let mut steepness_cost = OrderedFloat(
                        (self.height_map[nr][nc] - self.height_map[current.0][current.1]).abs(),
                    ) * cost_of_climb_multiplier
                        * factor;
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
        let mut path = Vec::new();
        for &current in targets_connected.iter() {
            let mut curr = current;
            while let Some((r, c)) = come_from.get(&curr) {
                if *r == a.0 && *c == a.1 {
                    break;
                }
                path.push((curr, (*r, *c)));
                curr = (*r, *c);
                self.road_level[*r][*c] = 1;
            }
            self.house_level[current.0][current.1] = 1;
            self.road_level[current.0][current.1] = 0;
        }
        self.house_level[a.0][a.1] = 1;
        self.road_level[a.0][a.1] = 0;
        println!("Path length: {}", path.len());
        let mut houses = vec![a];
        houses.extend(good_targets.iter().cloned());
        let _ = tx.send(DijkstraUpdate {
            path: path.clone(),
            houses,
        });
        return path;
    }
}

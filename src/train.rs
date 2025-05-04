use bevy::prelude::*;

#[derive(Component)]
pub struct Train {
    pub path: Vec<(usize, usize)>,
    pub index: usize,
    pub forward: bool,
}

impl Train {
    pub fn new(path: Vec<(usize, usize)>) -> Self {
        Train {
            path,
            index: 0,
            forward: true,
        }
    }
}

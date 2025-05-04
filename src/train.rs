pub struct Train {
    path: Vec<(usize, usize)>,
    index: usize,
    forward: bool,
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

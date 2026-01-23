use octa_force::glam::UVec3;

/// Tracks vertex indices to avoid emitting duplicate vertices during marching cubes mesh generation
pub struct IndexCache {
    size_x: usize,
    layers: [Vec<[u32; 4]>; 2],
    rows: [Vec<[u32; 3]>; 2],
    cells: [[u32; 2]; 2],
    current_cell: [u32; 12],
}

impl IndexCache {
    /// Create a new IndexCache for the given chunk size
    pub fn new(size: UVec3) -> IndexCache {
        IndexCache {
            size_x: size.x as usize,
            layers: [vec![[0; 4]; (size.x * size.y) as usize], vec![[0; 4]; (size.x * size.y) as usize]],
            rows: [vec![[0; 3]; size.x as usize], vec![[0; 3]; size.x as usize]],
            cells: [[0; 2]; 2],
            current_cell: [0; 12],
        }
    }

    /// Put an index in the cache at the given (x, y, edge) coordinate
    pub fn put(&mut self, x: usize, y: usize, edge: usize, index: u32) {
        match edge {
            4..=7 => self.layers[1][y * self.size_x + x][edge - 4] = index,
            _ => (),
        }

        match edge {
            6 => self.rows[1][x][0] = index,
            11 => self.rows[1][x][1] = index,
            10 => self.rows[1][x][2] = index,
            _ => (),
        }

        match edge {
            5 => self.cells[1][0] = index,
            10 => self.cells[1][0] = index,
            _ => (),
        }

        self.current_cell[edge] = index;
    }

    /// Retrieve an index from the cache at the given (x, y, edge) coordinate
    pub fn get(&mut self, x: usize, y: usize, edge: usize) -> u32 {
        let result = match edge {
            0..=3 => self.layers[0][y * self.size_x + x][edge],
            4 => self.rows[0][x][0],
            8 => self.rows[0][x][1],
            9 => self.rows[0][x][2],
            7 => self.cells[0][1],
            11 => self.cells[0][1],
            _ => 0,
        };

        if result > 0 {
            result
        } else {
            self.current_cell[edge]
        }
    }

    /// Update the cache when mesh extraction moves to the next cell
    pub fn advance_cell(&mut self) {
        self.cells.swap(0, 1);
        for i in self.current_cell.iter_mut() {
            *i = 0;
        }
    }

    /// Update the cache when mesh extraction moves to the next row
    pub fn advance_row(&mut self) {
        self.rows.swap(0, 1);
        for i in self.cells[0].iter_mut() {
            *i = 0;
        }
    }

    /// Update the cache when mesh extraction moves to the next layer
    pub fn advance_layer(&mut self) {
        self.layers.swap(0, 1);
        for i in self.cells[0].iter_mut() {
            *i = 0;
        }
        for i in self.rows[0].iter_mut() {
            *i = [0; 3];
        }
    }
}

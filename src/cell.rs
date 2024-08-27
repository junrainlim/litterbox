use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Cell {
    pub alive: u32,
    pub color: [f32; 4],
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            alive: 0,
            color: [0., 0., 0., 1.],
        }
    }
}


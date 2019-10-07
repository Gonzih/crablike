use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
pub const TORCH_RADIUS: i32 = 10;

pub struct Tcod {
    pub root: Root,
    pub fov: FovMap,
    pub con: Offscreen,
}

impl Tcod {
    pub fn compute_fov(&mut self, x: i32, y: i32) {
        self.fov.compute_fov(x, y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    pub fn populate_fov_map(&mut self, w: i32, h: i32, game: &mut super::Game) {
        for y in 0..h {
            for x in 0..w {
                &self.fov.set(
                    x,
                    y,
                    !game.map[x as usize][y as usize].block_sight,
                    !game.map[x as usize][y as usize].blocked,
                );
            }
        }
    }

    pub fn blit(&mut self, width: i32, height: i32) {
        blit(
            &self.con,
            (0, 0),
            (width, height),
            &mut self.root,
            (0, 0),
            1.0,
            1.0,
        );
    }
}

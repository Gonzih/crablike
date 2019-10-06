use tcod::console::*;

pub struct Tcod {
    pub root: Root,
    pub con: Offscreen,
}

impl Tcod {
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

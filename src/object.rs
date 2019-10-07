use rand::Rng;
use tcod::colors::*;
use tcod::console::*;
use super::gamemap::Rect;

const MAX_ROOM_MONSTERS: i32 = 3;

pub struct Object {
    pub x: i32,
    pub y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

pub fn place_objects(room: &Rect, objects: &mut Vec<Object>) {
    let num_mosters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_mosters {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        let monster = if rand::random::<f32>() < 0.8 {
            //create an orc
            Object::new(x, y, 'o', DESATURATED_GREEN)
        } else {
            Object::new(x, y, 'T', DARKER_GREEN)
        };

        objects.push(monster);
    }
}

use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod object;
use object::*;

mod mytcod;
use mytcod::*;

mod gamemap;
use gamemap::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

struct Game {
    map: Map,
    player: Object,
    objects: Vec<Object>,
}

impl Game {
    fn render_all(&self, tcod: &mut Tcod) {
        self.player.draw(&mut tcod.con);

        for object in &self.objects {
            object.draw(&mut tcod.con);
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let color = self.map[x as usize][y as usize].color();
                tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    fn move_player_by(&mut self, x: i32, y: i32) {
        let nx = self.player.x + x;
        let ny = self.player.y + y;

        if !self.map[nx as usize][ny as usize].blocked {
            self.player.move_by(x, y);
        }
    }
}

fn main() {
    println!("Starting Crabline game 🦀");

    tcod::system::set_fps(LIMIT_FPS);

    let (new_map, (player_x, player_y)) =  make_map();

    let player = Object::new(player_x, player_y, '@', WHITE);

    let mut game = Game{
        map: new_map,
        player: player,
        objects: vec![],
    };

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Crablike")
        .init();

    let mut tcod = Tcod {
        root: root,
        con: con,
    };

    while !tcod.root.window_closed() {
        tcod.con.clear();

        game.render_all(&mut tcod);

        tcod.blit(SCREEN_WIDTH, SCREEN_HEIGHT);

        tcod.root.flush();

        let exit = handle_keys(&mut tcod, &mut game);
        if exit {
            break;
        }
    }
}

fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> bool {
    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key { code: Up, .. } => game.move_player_by(0, -1),
        Key { code: Down, .. } => game.move_player_by(0, 1),
        Key { code: Left, .. } => game.move_player_by(-1, 0),
        Key { code: Right, .. } => game.move_player_by(1, 0),
        Key { code: Escape, .. } => return true,
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },

        _ => {}
    }

    false
}

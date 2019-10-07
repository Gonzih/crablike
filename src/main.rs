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

use tcod::map::{FovAlgorithm, Map as FovMap};

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

pub struct Game {
    pub map: Map,
    player: Object,
    objects: Vec<Object>,
}

impl Game {
    fn render_all(&mut self, tcod: &mut Tcod, fov_recompute: bool) {
        if fov_recompute {
            tcod.compute_fov(self.player.x, self.player.y);
        }

        self.player.draw(&mut tcod.con);

        for object in &self.objects {
            if tcod.fov.is_in_fov(object.x, object.y) {
                object.draw(&mut tcod.con);
            }
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let visible = tcod.fov.is_in_fov(x, y);

                let explored = &mut self.map[x as usize][y as usize].explored;
                if visible {
                    *explored = true;
                }

                if *explored {
                    let color = self.map[x as usize][y as usize].color(visible);
                    tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
                }
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
    let npc = Object::new(SCREEN_WIDTH/2 -5 , SCREEN_HEIGHT/2, '$', WHITE);

    let mut game = Game{
        map: new_map,
        player: player,
        objects: vec![npc],
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
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        con: con,
    };

    tcod.populate_fov_map(MAP_WIDTH, MAP_HEIGHT, &mut game);

    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        let fov_recompute = previous_player_position != (game.player.x, game.player.y);

        game.render_all(&mut tcod, fov_recompute);

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

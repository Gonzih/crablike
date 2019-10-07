use std::cmp;

use tcod::colors::*;
use tcod::console::*;

mod object;
use object::*;

mod mytcod;
use mytcod::*;

mod gamemap;
use gamemap::*;

use tcod::map::Map as FovMap;

const PLAYER: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

pub struct Game {
    pub map: Map,
    messages: Messages,
}

impl Game {
    fn render_all(&mut self, tcod: &mut Tcod, objects: &Vec<Object>, fov_recompute: bool) {
        if fov_recompute {
            tcod.compute_fov(objects[PLAYER].x, objects[PLAYER].y);
        }

        let mut to_draw: Vec<_> = objects
            .iter()
            .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
            .collect();
        to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));

        for object in &to_draw {
            object.draw(&mut tcod.con);
        }

        objects[PLAYER].draw(&mut tcod.con);

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let visible = tcod.fov.is_in_fov(x, y);

                let explored = &mut self.map[x as usize][y as usize].explored;
                if visible {
                    *explored = true;
                }

                if *explored {
                    let color = self.map[x as usize][y as usize].color(visible);
                    tcod.con
                        .set_char_background(x, y, color, BackgroundFlag::Set);
                }
            }
        }

        if let Some(fighter) = objects[PLAYER].fighter {
            tcod.root.print_ex(
                1,
                SCREEN_HEIGHT - 2,
                BackgroundFlag::None,
                TextAlignment::Left,
                format!("HP: {}/{}", fighter.hp, fighter.max_hp),
            )
        }
    }

    fn move_player_by(&mut self, objects: &mut Vec<Object>, x: i32, y: i32) {
        let nx = objects[PLAYER].x + x;
        let ny = objects[PLAYER].y + y;

        if !self.is_tile_blocked(objects, nx, ny) {
            objects[PLAYER].move_by(x, y);
        }
    }

    fn player_move_or_attack(&mut self, objects: &mut Vec<Object>, x: i32, y: i32) -> PlayerAction {
        let nx = objects[PLAYER].x + x;
        let ny = objects[PLAYER].y + y;

        let target_id = objects
            .iter()
            .position(|object| object.fighter.is_some() && object.pos() == (nx, ny));

        match target_id {
            Some(target_id) => {
                let (player, monster) = mut_two(PLAYER, target_id, objects);
                player.attack(monster, self);
            },
            None => {
                self.move_player_by(objects, x, y);
            }
        }

        PlayerAction::TookTurn
    }

    fn is_tile_blocked(&self, objects: &Vec<Object>, x: i32, y: i32) -> bool {
        is_blocked(x, y, &self.map, objects)
    }
}

fn main() {
    println!("Starting Crabline game 🦀");

    tcod::system::set_fps(LIMIT_FPS);

    let mut player = Object::new(0, 0, '@', "player", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: DeathCallback::Player,
    });

    let mut objects = vec![player];

    let mut game = Game {
        map: make_map(&mut objects),
        messages: Messages::new(),
    };

    game.messages.add(
        "Welcome, prepare to get died!",
        RED,
    );

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let panel = Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT);

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
        panel: panel,
    };

    tcod.populate_fov_map(MAP_WIDTH, MAP_HEIGHT, &mut game);

    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);

        game.render_all(&mut tcod, &objects, fov_recompute);

        tcod.panel.set_default_background(BLACK);
        tcod.panel.clear();

        let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
        let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);

        tcod.render_bar(
            1,
            1,
            BAR_WIDTH,
            "HP",
            hp,
            max_hp,
            LIGHT_RED,
            DARKER_RED,
        );

        tcod.print_messages(&game.messages);

        tcod.blit_con(SCREEN_WIDTH, SCREEN_HEIGHT);
        tcod.blit_panel(SCREEN_WIDTH, SCREEN_HEIGHT, PANEL_Y);
        tcod.root.flush();

        let action = handle_keys(&mut tcod, &mut objects, &mut game);

        match action {
            PlayerAction::Exit => break,
            PlayerAction::TookTurn if objects[PLAYER].alive => {
                for id in 0..objects.len() {
                    if objects[id].ai.is_some() {
                        ai_take_turn(id, &tcod, &mut objects, &mut game);
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_keys(tcod: &mut Tcod, objects: &mut Vec<Object>, game: &mut Game) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key { code: Up, .. } => game.player_move_or_attack(objects, 0, -1),
        Key { code: Down, .. } => game.player_move_or_attack(objects, 0, 1),
        Key { code: Left, .. } => game.player_move_or_attack(objects, -1, 0),
        Key { code: Right, .. } => game.player_move_or_attack(objects, 1, 0),
        Key { code: Escape, .. } => PlayerAction::Exit,
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            PlayerAction::DidntTakeTurn
        }

        _ => PlayerAction::DidntTakeTurn,
    }
}

fn ai_take_turn(monster_id: usize, tcod: &Tcod, objects: &mut Vec<Object>, game: &mut Game) {
    let (monster_x, monster_y) = objects[monster_id].pos();

    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(monster_id, player_x, player_y, &game.map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            let (player, monster) = mut_two(PLAYER, monster_id, objects);
            monster.attack(player, game);
        }
    }
}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

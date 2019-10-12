use std::cmp;

use tcod::colors::*;
use tcod::console::*;

mod object;
use object::*;

mod mytcod;
use mytcod::*;

mod gamemap;
use gamemap::*;

use tcod::input::{self, Event, Key};
use tcod::map::Map as FovMap;

use rand::Rng;

const PLAYER: usize = 0;
const HEAL_AMOUNT: i32 = 4;
const LIGHTNING_DAMAGE: i32 = 40;
const LIGHTNING_RANGE: i32 = 5;
const CONFUSE_RANGE: i32 = 8;
const CONFUSE_NUM_TURNS: i32 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

pub struct Game {
    pub map: Map,
    inventory: Vec<Object>,
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
        self.move_object_by(PLAYER, objects, x, y);
    }

    fn move_object_by(&mut self, id: usize, objects: &mut Vec<Object>, x: i32, y: i32) {
        let nx = objects[id].x + x;
        let ny = objects[id].y + y;

        if !self.is_tile_blocked(objects, nx, ny) {
            objects[id].move_by(x, y);
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
            }
            None => {
                self.move_player_by(objects, x, y);
            }
        }

        PlayerAction::TookTurn
    }

    fn is_tile_blocked(&self, objects: &Vec<Object>, x: i32, y: i32) -> bool {
        is_blocked(x, y, &self.map, objects)
    }

    fn pick_item_up(&mut self, object_id: usize, objects: &mut Vec<Object>) {
        if self.inventory.len() >= 26 {
            self.messages.add(
                format!(
                    "Your inventory is full, cannot pick up {}.",
                    objects[object_id].name
                ),
                RED,
            )
        } else {
            let item = objects.swap_remove(object_id);
            self.messages
                .add(format!("You picked up a {}!", item.name), GREEN);
            self.inventory.push(item);
        }
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
        inventory: vec![],
    };

    game.messages.add("Welcome, prepare to get died!", RED);

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
        key: Default::default(),
    };

    tcod.populate_fov_map(MAP_WIDTH, MAP_HEIGHT, &mut game);

    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);

        match input::check_for_event(input::KEY_PRESS) {
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }

        game.render_all(&mut tcod, &objects, fov_recompute);

        tcod.panel.set_default_background(BLACK);
        tcod.panel.clear();

        let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
        let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);

        tcod.render_bar(1, 1, BAR_WIDTH, "HP", hp, max_hp, LIGHT_RED, DARKER_RED);

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
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;
    match (tcod.key, tcod.key.text(), player_alive) {
        (Key { code: Up, .. }, _, true) => game.player_move_or_attack(objects, 0, -1),
        (Key { code: Down, .. }, _, true) => game.player_move_or_attack(objects, 0, 1),
        (Key { code: Left, .. }, _, true) => game.player_move_or_attack(objects, -1, 0),
        (Key { code: Right, .. }, _, true) => game.player_move_or_attack(objects, 1, 0),
        (Key { code: Escape, .. }, _, _) => Exit,
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }

        (Key { code: Text, .. }, "g", true) => {
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                game.pick_item_up(item_id, objects);
            };
            TookTurn
        }

        (Key { code: Text, .. }, "i", true) => {
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next an item to use it, or any other to cancel.\n",
                &mut tcod.root,
            );
            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, objects);
            }
            TookTurn
        }

        _ => DidntTakeTurn,
    }
}

fn ai_take_turn(monster_id: usize, tcod: &Tcod, objects: &mut Vec<Object>, game: &mut Game) {
    use Ai::*;
    if let Some(ai) = objects[monster_id].ai.take() {
        let new_ai = match ai {
            Basic => ai_basic(monster_id, tcod, objects, game),
            Confused {
                previous_ai,
                num_turns,
            } => ai_confused(monster_id, tcod, objects, game, previous_ai, num_turns),
        };
        objects[monster_id].ai = Some(new_ai);
    }
}

fn ai_basic(monster_id: usize, tcod: &Tcod, objects: &mut Vec<Object>, game: &mut Game) -> Ai {
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

    Ai::Basic
}

fn ai_confused(
    monster_id: usize,
    _tcod: &Tcod,
    objects: &mut Vec<Object>,
    game: &mut Game,
    previous_ai: Box<Ai>,
    num_turns: i32,
) -> Ai {
    if num_turns >= 0 {
        game.move_object_by(
            monster_id,
            objects,
            rand::thread_rng().gen_range(-1, 2),
            rand::thread_rng().gen_range(-1, 2),
        );
        Ai::Confused {
            previous_ai: previous_ai,
            num_turns: num_turns - 1,
        }
    } else {
        game.messages.add(
            format!("The {} is no longer confused!", objects[monster_id].name),
            RED,
        );

        *previous_ai
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

enum UseResult {
    UsedUp,
    Cancelled,
}

fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    use Item::*;

    if let Some(item) = game.inventory[inventory_id].item {
        let on_use = match item {
            Heal => cast_heal,
            Lightning => cast_lightning,
            Confuse => cast_confuse,
        };
        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                game.inventory.remove(inventory_id);
            }
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!("The {} cannot be used.", game.inventory[inventory_id].name),
            WHITE,
        )
    }
}

fn cast_heal(
    _inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == fighter.max_hp {
            game.messages.add("You are alraedy at full health.", RED);
            return UseResult::Cancelled;
        }

        game.messages.add("Your wounds are closing!", LIGHT_VIOLET);
        objects[PLAYER].heal(HEAL_AMOUNT);
        return UseResult::UsedUp;
    }

    UseResult::Cancelled
}

fn cast_lightning(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    let monster_id = closest_monster(tcod, objects, LIGHTNING_RANGE);
    if let Some(monster_id) = monster_id {
        game.messages.add(
            format!(
                "A lightting bolt strikes the {} with a loud thunder! for {} hit points.",
                objects[monster_id].name, LIGHTNING_DAMAGE,
            ),
            LIGHT_BLUE,
        );
        objects[monster_id].take_damage(LIGHTNING_DAMAGE, game);
        UseResult::UsedUp
    } else {
        game.messages
            .add("No enemy is close enough to strike.", RED);
        UseResult::Cancelled
    }
}

fn closest_monster(tcod: &Tcod, objects: &[Object], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32;

    for (id, object) in objects.iter().enumerate() {
        if (id != PLAYER)
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.x, object.y)
        {
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                closest_enemy = Some(id);
                closest_dist = dist;
            }
        }
    }

    closest_enemy
}

fn cast_confuse(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    let monster_id = closest_monster(tcod, objects, CONFUSE_RANGE);
    if let Some(monster_id) = monster_id {
        game.messages.add(
            format!(
                "{} looks confused",
                objects[monster_id].name,
            ),
            LIGHT_GREEN,
        );
        let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);
        objects[monster_id].ai = Some(Ai::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        UseResult::UsedUp
    } else {
        game.messages
            .add("No enemy is close enough to strike.", RED);
        UseResult::Cancelled
    }
}

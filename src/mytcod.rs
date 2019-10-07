use tcod::console::*;
use tcod::colors::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
pub const LIMIT_FPS: i32 = 20;
pub const BAR_WIDTH: i32 = 20;
pub const PANEL_HEIGHT: i32 = 7;
pub const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
pub const TORCH_RADIUS: i32 = 10;
pub const MSG_X: i32 = BAR_WIDTH + 2;
pub const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
pub const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

pub struct Messages {
    messages: Vec<(String, Color)>,
}

impl Messages {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color));
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
    }
}

pub struct Tcod {
    pub root: Root,
    pub fov: FovMap,
    pub con: Offscreen,
    pub panel: Offscreen,
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

    pub fn blit_con(&mut self, width: i32, height: i32) {
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

    pub fn blit_panel(&mut self, width: i32, height: i32, y: i32) {
        blit(
            &self.panel,
            (0, 0),
            (width, height),
            &mut self.root,
            (0, y),
            1.0,
            1.0,
        );
    }

    pub fn render_bar(
        &mut self,
        x: i32,
        y: i32,
        total_width: i32,
        name: &str,
        value: i32,
        maximum: i32,
        bar_color: Color,
        back_color: Color,
    ) {
        let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

        self.panel.set_default_background(back_color);
        self.panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

        self.panel.set_default_background(bar_color);
        if bar_width > 0 {
            self.panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
        }

        self.panel.set_default_foreground(WHITE);
        self.panel.print_ex(
            x + total_width / 2,
            y,
            BackgroundFlag::None,
            TextAlignment::Center,
            &format!("{}: {}/{}", name, value, maximum),
        );
    }

    pub fn print_messages(&mut self, msgs: &Messages) {
        let mut y = MSG_HEIGHT as i32;

        for &(ref msg, color) in msgs.iter().rev() {
            let msg_height = self.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
            y -= msg_height;
            if y < 0 {
                break;
            }
            self.panel.set_default_foreground(color);
            self.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        }
    }
}

use easyray::prelude::*;
use rand::Rng;

pub const COLS: i32 = 32;
pub const ROWS: i32 = 24;

#[derive(Default, PartialEq)]
pub enum Direc {
    #[default]
    Left,
    Right,
    Down,
    Up,
}

impl Direc {
    pub fn next(&self, mut last: (i32, i32)) -> (i32, i32) {
        match *self {
            Self::Left => last.0 -= 1,
            Self::Right => last.0 += 1,
            Self::Up => last.1 -= 1,
            Self::Down => last.1 += 1,
        }
        (check_limit(last.0, COLS), check_limit(last.1, ROWS))
    }
}

fn check_limit(coord: i32, limit: i32) -> i32 {
    if coord < 0 {
        limit - 1
    } else if coord > limit {
        0
    } else {
        coord
    }
}

#[derive(Default)]
pub enum Page {
    #[default]
    Menu,
    Play,
}

#[derive(Default)]
pub struct World {
    direc: Direc,
    apple: (i32, i32),
    body: Vec<(i32, i32)>,
    page: Page,
    timer: f32,
}
impl World {
    fn eat(&mut self, next: (i32, i32)) {
        self.body.insert(0, next);
        while self.body.contains(&self.apple) {
            self.apple = (
                rand::rng().random_range(0..COLS),
                rand::rng().random_range(0..ROWS),
            )
        }
    }
}
pub enum Msg {
    Exit,
    Page(Page),
    Down,
    Left,
    Right,
    Up,
}
impl Console for World {
    type Event = Msg;
    fn load(&mut self, _path: &str) {
        self.eat(self.apple);
    }
    fn update(&mut self, dt: f32) -> bool {
        if let Page::Play = self.page {
            self.timer += dt;
            if self.timer > 1.0 / (3.0 + self.body.len() as f32 / 5.0) {
                self.timer = 0.0;
                let next = self.direc.next(self.body[0]);
                if self.body.contains(&next) {
                    self.body.clear();
                } else {
                    if next != self.apple {
                        self.body.pop();
                    }
                    self.eat(next);
                }
            }
        }
        !self.body.is_empty()
    }
    fn handle(&mut self, event: Self::Event) {
        match event {
            Msg::Exit => self.body.clear(),
            Msg::Page(value) => self.page = value,
            Msg::Down => {
                if [Direc::Left, Direc::Right].contains(&self.direc) {
                    self.direc = Direc::Down;
                }
            }
            Msg::Left => {
                if [Direc::Down, Direc::Up].contains(&self.direc) {
                    self.direc = Direc::Left;
                }
            }
            Msg::Right => {
                if [Direc::Down, Direc::Up].contains(&self.direc) {
                    self.direc = Direc::Right;
                }
            }
            Msg::Up => {
                if [Direc::Left, Direc::Right].contains(&self.direc) {
                    self.direc = Direc::Up;
                }
            }
        }
    }
    fn draw(
        &self,
        canvas: &mut RaylibDrawHandle,
        width: i32,
        height: i32,
        key: Option<KeyboardKey>,
    ) -> Option<Self::Event> {
        let cell = (width / COLS, height / ROWS);
        canvas.clear_background(solarized::BASE2);
        if let Page::Play = self.page {
            for x in 0..COLS {
                for y in 0..ROWS {
                    if (x + y) % 2 == 0 {
                        canvas.draw_rectangle(
                            x * cell.0,
                            y * cell.1,
                            cell.0,
                            cell.1,
                            solarized::BASE3,
                        );
                    }
                }
            }
            for (x, y) in &self.body {
                canvas.draw_rectangle(x * cell.0, y * cell.1, cell.0, cell.1, solarized::CYAN);
            }
            canvas.draw_rectangle(
                self.body[0].0 * cell.0,
                self.body[0].1 * cell.1,
                cell.0,
                cell.1,
                solarized::GREEN,
            );
            canvas.draw_rectangle(
                self.apple.0 * cell.0,
                self.apple.1 * cell.1,
                cell.0,
                cell.1,
                solarized::RED,
            );
            if let Some(KeyboardKey::KEY_ESCAPE) = key {
                return Some(Msg::Page(Page::Menu));
            }
            if let Some(KeyboardKey::KEY_DOWN) = key {
                return Some(Msg::Down);
            }
            if let Some(KeyboardKey::KEY_UP) = key {
                return Some(Msg::Up);
            }
            if let Some(KeyboardKey::KEY_LEFT) = key {
                return Some(Msg::Left);
            }
            if let Some(KeyboardKey::KEY_RIGHT) = key {
                return Some(Msg::Right);
            }
        } else {
            let text = "MENU";
            let size = 44;
            canvas.draw_text(
                text,
                width / 2 - canvas.measure_text(text, size) / 2,
                height / 2,
                size,
                solarized::GREEN,
            );
            if let Some(KeyboardKey::KEY_ENTER) = key {
                return Some(Msg::Page(Page::Play));
            }
            if let Some(KeyboardKey::KEY_ESCAPE) = key {
                return Some(Msg::Exit);
            }
        }
        None
    }
    fn exit(&self, _path: &str) {}
}

use easyray::prelude::*;
use rand::Rng;

#[derive(Default, Clone)]
pub enum Direc {
    #[default]
    Left,
    Right,
    Down,
    Up,
}

impl Direc {
    fn way(&self) -> (i32, i32) {
        match *self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Up => (0, -1),
            Self::Down => (0, 1),
        }
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
    predi: Option<Direc>,
    apple: (i32, i32),
    body: Vec<(i32, i32)>,
    field: (i32, i32),
    timer: f32,
    page: Page,
}

impl World {
    fn eat(&mut self, next: (i32, i32)) {
        self.body.insert(0, next);
        while self.body.contains(&self.apple) {
            self.apple = (
                rand::rng().random_range(0..self.field.0),
                rand::rng().random_range(0..self.field.1),
            )
        }
    }
    fn turn(&mut self, value: Direc) {
        if self.direc.way().0 != -value.way().0 && self.direc.way().1 != -value.way().1 {
            self.predi = Some(value);
        }
    }
    fn step(&mut self) -> f32 {
        if let Some(value) = self.predi.clone() {
            self.direc = value;
            self.predi = None;
        }
        let next = (
            (self.field.0 + self.body[0].0 + self.direc.way().0) % self.field.0,
            (self.field.1 + self.body[0].1 + self.direc.way().1) % self.field.1,
        );
        if self.body.contains(&next) {
            self.body.clear();
            self.eat(next);
            self.page = Page::Menu;
        } else {
            if next != self.apple {
                self.body.pop();
            }
            self.eat(next);
        };
        0.0
    }
}

impl Console for World {
    fn load(&mut self, _path: &str) {
        self.field = (32, 24);
        self.eat(self.apple);
    }
    fn update(&mut self, dt: f32) {
        if let Page::Play = self.page {
            self.timer = match self.timer > 1.0 / (3.0 + self.body.len() as f32 / 5.0) {
                true => self.step(),
                false => self.timer + dt,
            }
        }
    }
    fn handle(&mut self, key: KeyboardKey) -> bool {
        match (&self.page, key) {
            (&Page::Menu, KeyboardKey::KEY_ESCAPE) => return false,
            (&Page::Menu, KeyboardKey::KEY_ENTER) => self.page = Page::Play,
            (&Page::Play, KeyboardKey::KEY_ESCAPE) => self.page = Page::Menu,
            (&Page::Play, KeyboardKey::KEY_DOWN) => self.turn(Direc::Down),
            (&Page::Play, KeyboardKey::KEY_UP) => self.turn(Direc::Up),
            (&Page::Play, KeyboardKey::KEY_LEFT) => self.turn(Direc::Left),
            (&Page::Play, KeyboardKey::KEY_RIGHT) => self.turn(Direc::Right),
            _ => {}
        }
        true
    }
    fn draw(&self, canvas: &mut RaylibDrawHandle, width: i32, height: i32) {
        let cell = (width / self.field.0, height / self.field.1);
        canvas.clear_background(solarized::BASE2);
        if let Page::Play = self.page {
            for y in 0..self.field.1 {
                for x in 0..self.field.0 {
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
            canvas.draw_circle(
                self.apple.0 * cell.0 + cell.0 / 2,
                self.apple.1 * cell.1 + cell.1 / 2,
                cell.0 as f32 / 2.0,
                solarized::RED,
            );
        } else {
            let text = "MENU";
            let size = 44;
            canvas.draw_text(
                text,
                width / 2 - canvas.measure_text(text, size) / 2,
                height / 2,
                size,
                solarized::BLUE,
            );
        }
    }
    fn exit(&self, _path: &str) {}
}

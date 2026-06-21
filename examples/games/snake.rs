    use easyray::prelude::*;
    use rand::Rng;

    #[derive(Default,PartialEq)]
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
        Welcome,
        Playing,
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
    impl Console for World {
        fn load(&mut self, _path: &str) {
            self.eat(self.apple);
        }
        fn update(&mut self, dt: f32) -> bool {
            if let Page::Playing = self.page {
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
            ! self.body.is_empty()
        }
        fn handle(&mut self, key: KeyboardKey) {
            match (&self.page, key) {
                (&Page::Welcome, KeyboardKey::KEY_ESCAPE) => self.body.clear(),
                (&Page::Welcome, KeyboardKey::KEY_ENTER) => self.page = Page::Playing,
                (&Page::Playing, KeyboardKey::KEY_ESCAPE) => self.page = Page::Welcome,
                (&Page::Playing, KeyboardKey::KEY_DOWN) => {
                    if [Direc::Left, Direc::Right].contains(&self.direc) {
                        self.direc = Direc::Down;
                    }
                }
                (&Page::Playing, KeyboardKey::KEY_UP) => {
                    if [Direc::Left, Direc::Right].contains(&self.direc) {
                        self.direc = Direc::Up;
                    }
                }
                (&Page::Playing, KeyboardKey::KEY_LEFT) => {
                    if [Direc::Down, Direc::Up].contains(&self.direc) {
                        self.direc = Direc::Left;
                    }
                }
                (&Page::Playing, KeyboardKey::KEY_RIGHT) => {
                    if [Direc::Down, Direc::Up].contains(&self.direc) {
                        self.direc = Direc::Right;
                    }
                }
                _ => {},
            }
        }
        fn draw(&self, canvas: &mut RaylibDrawHandle) {
            canvas.clear_background(solarized::BASE2);
            if let Page::Playing = self.page {
                for x in 0..COLS {
                    for y in 0..ROWS {
                        if (x + y) % 2 == 0 {
                            canvas.draw_rectangle(
                                x * CELL,
                                y * CELL,
                                CELL,
                                CELL,
                                solarized::BASE3,
                            );
                        }
                    }
                }
                for (x, y) in &self.body {
                    canvas.draw_rectangle(x * CELL, y * CELL, CELL, CELL, solarized::BLUE);
                }
                canvas.draw_rectangle(
                    self.body[0].0 * CELL,
                    self.body[0].1 * CELL,
                    CELL,
                    CELL,
                    solarized::GREEN,
                );
                canvas.draw_rectangle(
                    self.apple.0 * CELL,
                    self.apple.1 * CELL,
                    CELL,
                    CELL,
                    solarized::RED,
                );
            } else {
                let text = "MENU";
                let size = 44;
                canvas.draw_text(text, SCREEN.0 / 2 - canvas.measure_text(text, size) / 2, SCREEN.1 / 2, size, solarized::GREEN);
            }
        }
        fn exit(&self, _path: &str) {}
    }


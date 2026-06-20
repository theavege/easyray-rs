mod models {
    use rand::Rng;

    #[derive(Default)]
    pub enum Difficulty {
        #[default]
        Normal = 0,
        Hard,
        Easy,
    }

    impl Difficulty {
        pub fn limit(&self) -> (f32, f32) {
            const CACTUS_MIN_GAP: f32 = 320.0;
            const CACTUS_MAX_GAP: f32 = 520.0;
            (
                CACTUS_MIN_GAP * self.gap_scale(),
                CACTUS_MAX_GAP * self.gap_scale(),
            )
        }
        pub fn scaled_gap(&self, first: bool) -> f32 {
            let (min, max) = self.limit();
            match first {
                true => (min + max) * 0.5,
                false => rand::rng().random_range(min..max),
            }
        }
        pub fn label(&self) -> &'static str {
            match self {
                Self::Easy => "Easy",
                Self::Normal => "Normal",
                Self::Hard => "Hard",
            }
        }
        pub fn speed_mul(&self) -> f32 {
            match self {
                Self::Easy => 0.9,
                Self::Normal => 1.0,
                Self::Hard => 1.12,
            }
        }
        pub fn gap_scale(&self) -> f32 {
            match self {
                Self::Easy => 1.15,
                Self::Normal => 1.0,
                Self::Hard => 0.85,
            }
        }
        pub fn ptero_enabled(&self) -> bool {
            matches!(self, Self::Normal | Self::Hard)
        }
    }

    pub enum ObstacleKind {
        Cactus(bool),      // 30x46
        Pterodactyl(bool), // flies at given y
    }

    pub struct Obstacle {
        pub rect: (f32, f32, f32, f32),
        pub kind: ObstacleKind,
    }

    #[derive(Default)]
    pub struct Dino {
        pub ducking: bool,
        pub jumping: bool,
        pub leg_state: bool,
        pub leg_timer: f32,
        pub y: f32,
        pub vy: f32,
    }

    impl Dino {
        pub fn x(&self) -> f32 {
            120.0
        }
        pub fn w(&self) -> f32 {
            44.0
        }
        pub fn height(&self) -> f32 {
            if self.ducking { 30.0 } else { 48.0 }
        }
    }
}

mod games {
    use super::models::*;
    use easyray::*;
    use rand::Rng;
    const PADDING: i32 = 24;

    #[derive(Default)]
    pub enum Page {
        #[default]
        Welcome = 0,
        Playing,
        Paused,
        GameOver,
    }

    #[derive(Default)]
    pub struct World {
        page: Page,
        avatar: Dino,
        npc: Vec<Obstacle>,

        // Counter
        next_gap: f32,
        run_speed: f32,
        distance_x: f32,
        ground_scroll: f32,
        difficulty: Difficulty,
        score: u32,
        high_score: u32,
    }

    impl World {
        fn ground_y(&self) -> f32 {
            (SCREEN.1 - PADDING) as f32 - 64.0
        }
        fn draw_game(&self, canvas: &mut RaylibDrawHandle) {
            // Dino
            let dino_rect = Rectangle::new(
                self.avatar.x(),
                self.avatar.y,
                self.avatar.w(),
                self.avatar.height(),
            );
            canvas.draw_rectangle_rec(
                dino_rect,
                match self.avatar.jumping {
                    true => solarized::VIOLET,
                    false => solarized::GREEN,
                },
            );
            canvas.draw_rectangle_lines_ex(dino_rect, 2.0, solarized::BASE01);

            // Feet animation (simple legs)
            if !self.avatar.jumping {
                let foot_w = 10;
                let foot_h = 4;
                let step = 6.0;
                canvas.draw_rectangle(
                    match self.avatar.leg_state {
                        true => (self.avatar.x() + step) as i32,
                        false => (self.avatar.x() + self.avatar.w() - step - foot_w as f32) as i32,
                    },
                    (self.avatar.y + self.avatar.height() - foot_h as f32) as i32,
                    foot_w,
                    foot_h,
                    solarized::CYAN,
                );
            }

            for o in &self.npc {
                let rect = Rectangle::new(o.rect.0, o.rect.1, o.rect.2, o.rect.3);
                canvas.draw_rectangle_rec(
                    rect,
                    match o.kind {
                        ObstacleKind::Cactus(..) => solarized::ORANGE,
                        ObstacleKind::Pterodactyl(..) => solarized::BLUE,
                    },
                );
                canvas.draw_rectangle_lines_ex(rect, 2.0, solarized::BASE01);
            }
        }
        fn draw_menu(&self, canvas: &mut RaylibDrawHandle) {
            let cx = SCREEN.0 / 2;
            let mut y = (SCREEN.1 / 2) - 90;

            draw_centered_text(canvas, "T-Rex Runner", cx, y, 48, solarized::BASE00);
            y += 56;
            draw_centered_text(
                canvas,
                "Solarized Light edition",
                cx,
                y,
                24,
                solarized::BASE1,
            );
            y += 56;

            draw_centered_text(
                canvas,
                "Press <1> - Easy, <2> - Normal, <3> - Hard",
                cx,
                y,
                22,
                solarized::BASE00,
            );
            y += 30;
            draw_centered_text(
                canvas,
                &format!("Selected: {}", self.difficulty.label()),
                cx,
                y,
                24,
                solarized::BLUE,
            );
            y += 44;

            draw_centered_text(canvas, "Enter to Start", cx, y, 28, solarized::GREEN);
            y += 24;
            draw_centered_text(
                canvas,
                "Controls: Space/Up (jump), Down (duck), P (pause), Esc (menu)",
                cx,
                y,
                18,
                solarized::BASE00,
            );
        }
    }

    impl Console for World {
        fn load(&mut self, _path: &str) {}
        fn exit(&self, _path: &str) {}
        fn handle(&mut self, rl: &mut RaylibHandle) -> Option<bool> {
            if let Some(key) = rl.get_key_pressed() {
                match (&self.page, key) {
                    (&Page::Welcome, KeyboardKey::KEY_ONE) => self.difficulty = Difficulty::Easy,
                    (&Page::Welcome, KeyboardKey::KEY_TWO) => self.difficulty = Difficulty::Normal,
                    (&Page::Welcome, KeyboardKey::KEY_THREE) => self.difficulty = Difficulty::Hard,
                    (&Page::Welcome, KeyboardKey::KEY_ESCAPE) => return None,
                    (&Page::Welcome, KeyboardKey::KEY_ENTER) => {
                        self.avatar = Dino::default();
                        self.avatar.y = self.ground_y() - self.avatar.height();
                        self.distance_x = 0.0;
                        self.ground_scroll = 0.0;
                        self.score = 0;
                        const BASE_RUN_SPEED: f32 = 360.0;
                        self.run_speed = BASE_RUN_SPEED * self.difficulty.speed_mul();
                        self.npc.clear();
                        self.next_gap = self.difficulty.scaled_gap(true);
                        self.page = Page::Playing;
                    }
                    (&Page::Paused, KeyboardKey::KEY_ENTER) => self.page = Page::Playing,
                    (&Page::GameOver, KeyboardKey::KEY_ENTER) => self.page = Page::Welcome,
                    (&Page::Playing, KeyboardKey::KEY_SPACE) => {
                        if !self.avatar.jumping {
                            self.avatar.vy = -900.0;
                            self.avatar.jumping = true;
                        }
                    }
                    (&Page::Playing, KeyboardKey::KEY_DOWN) => {
                        if !self.avatar.jumping {
                            self.avatar.ducking = true;
                        }
                    }
                    (&Page::Playing, KeyboardKey::KEY_ENTER) => self.page = Page::Paused,
                    (&Page::Playing, KeyboardKey::KEY_ESCAPE) => self.page = Page::Welcome,
                    _ => return Some(false),
                }
                Some(true)
            } else {
                Some(false)
            }
        }
        fn update(&mut self, dt: f32) -> bool {
            if let Page::Playing = self.page {
                let ground_top = self.ground_y() - self.avatar.height();
                if self.avatar.jumping {
                    self.avatar.vy += 2400.0 * dt;
                    self.avatar.y += self.avatar.vy * dt;
                    self.avatar.jumping = self.avatar.y < ground_top;
                } else {
                    self.avatar.y = ground_top;
                    self.avatar.vy = 0.0;
                    self.avatar.leg_timer += dt;
                    if self.avatar.leg_timer >= 0.12 {
                        self.avatar.leg_timer = 0.0;
                        self.avatar.leg_state = !self.avatar.leg_state;
                    }
                }
                self.run_speed *= (1.0 + (0.06 * dt)).min(1.02);
                let dx = self.run_speed * dt;
                self.distance_x += dx;
                self.ground_scroll = (self.ground_scroll + dx) % 32.0;
                self.score = (self.score as f32 + 120.0 * dt) as u32;
                self.next_gap -= dx;
                if self.next_gap <= 0.0 {
                    let kind = match self.difficulty.ptero_enabled()
                        && self.score > 300
                        && rand::rng().random_bool(0.35)
                    {
                        true => ObstacleKind::Pterodactyl(rand::rng().random_bool(0.5)),
                        false => ObstacleKind::Cactus(rand::rng().random_bool(0.4)),
                    };
                    let (y, w, h) = match kind {
                        ObstacleKind::Cactus(tall) => match tall {
                            true => (self.ground_y() - 46.0, 24.0, 46.0),
                            false => (self.ground_y() - 60.0, 36.0, 60.0),
                        },
                        ObstacleKind::Pterodactyl(high) => match high {
                            true => (self.ground_y() - 48.0 - 22.0 - 24.0, 46.0, 24.0),
                            false => (self.ground_y() - 30.0 - 10.0 - 24.0, 46.0, 24.0),
                        },
                    };
                    let range = if let ObstacleKind::Pterodactyl(..) = kind {
                        const PTERO_MIN_GAP: f32 = 520.0;
                        const PTERO_MAX_GAP: f32 = 760.0;
                        PTERO_MIN_GAP * self.difficulty.gap_scale()
                            ..PTERO_MAX_GAP * self.difficulty.gap_scale()
                    } else {
                        let (min, max) = self.difficulty.limit();
                        min..max
                    };
                    self.next_gap = rand::rng().random_range(range);
                    self.npc.push(Obstacle {
                        rect: (SCREEN.0 as f32 + 40.0, y, w, h),
                        kind,
                    });
                }
                for ob in &mut self.npc {
                    ob.rect.0 -= dx;
                }
                self.npc.retain(|o| o.rect.0 + o.rect.2 >= -60.0);
                if self.npc.iter().any(|obstacle| {
                    rect_intersect(
                        Rectangle::new(
                            self.avatar.x(),
                            self.avatar.y,
                            self.avatar.w(),
                            self.avatar.height(),
                        ),
                        Rectangle::new(
                            obstacle.rect.0,
                            obstacle.rect.1,
                            obstacle.rect.2,
                            obstacle.rect.3,
                        ),
                    )
                }) {
                    self.high_score = self.high_score.max(self.score);
                    self.page = Page::GameOver;
                }
                return true;
            }
            false
        }
        fn draw(&self, canvas: &mut RaylibDrawHandle) {
            canvas.clear_background(solarized::BACKGROUND);
            canvas.draw_rectangle_lines(
                PADDING,
                PADDING,
                SCREEN.0 - PADDING * 2,
                SCREEN.0 - PADDING * 2,
                solarized::FOREGROUND,
            );
            canvas.draw_rectangle(
                PADDING,
                self.ground_y() as i32,
                SCREEN.0 - PADDING * 2,
                4,
                solarized::FOREGROUND,
            );
            canvas.draw_text(
                &format!("Score: {:05}   High: {:05}", self.score, self.high_score),
                PADDING + 8,
                PADDING - 18,
                20,
                solarized::FOREGROUND2,
            );

            const STEP: i32 = 32;
            let offset = (self.ground_scroll as i32) % STEP;
            for x in (PADDING - offset..SCREEN.0 - PADDING).step_by(STEP as usize) {
                if (PADDING..SCREEN.0 - PADDING).contains(&x) {
                    canvas.draw_line(
                        x,
                        self.ground_y() as i32,
                        x + 8,
                        self.ground_y() as i32,
                        solarized::FOREGROUND2,
                    );
                }
            }
            match self.page {
                Page::Welcome => self.draw_menu(canvas),
                Page::Playing => self.draw_game(canvas),
                Page::Paused => {
                    self.draw_game(canvas);
                    draw_overlay(
                        canvas,
                        "Paused",
                        "Press P or Enter to resume",
                        solarized::BASE00,
                        solarized::BASE1,
                    );
                }
                Page::GameOver => {
                    self.draw_game(canvas);
                    draw_overlay(
                        canvas,
                        "Game Over",
                        "<ENTER> or <ESC>: Menu",
                        solarized::FOREGROUND2,
                        solarized::FOREGROUND,
                    );
                }
            }
        }
    }

    fn rect_intersect(a: Rectangle, b: Rectangle) -> bool {
        a.x < (b.x + b.width)
            && (a.x + a.width) > b.x
            && a.y < (b.y + b.height)
            && (a.y + a.height) > b.y
    }

    fn draw_centered_text(
        d: &mut RaylibDrawHandle,
        text: &str,
        cx: i32,
        y: i32,
        size: i32,
        color: Color,
    ) {
        let w = d.measure_text(text, size);
        d.draw_text(text, cx - w / 2, y, size, color);
    }

    fn draw_overlay(
        d: &mut RaylibDrawHandle,
        title: &str,
        subtitle: &str,
        fg: Color,
        accent: Color,
    ) {
        d.draw_rectangle(0, 0, SCREEN.0, SCREEN.0, Color::new(0, 0, 0, 32));
        let cx = SCREEN.0 / 2;
        let cy = SCREEN.1 / 2;
        draw_centered_text(d, title, cx, cy - 16, 44, fg);
        draw_centered_text(d, subtitle, cx, cy + 26, 20, accent);
    }
}

fn main() {
    use easyray::*;
    games::World::run("Runner");
}

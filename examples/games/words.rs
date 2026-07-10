mod models {
    use rand::prelude::SliceRandom;

    #[derive(Default)]
    pub struct Score {
        pub typed: (u32, u32),
        pub highscore: (u32, u32),
        pub secs: f32,
        pub timer: f32,
        pub level: Level,
        pub lang: Lang,
    }

    impl Score {
        pub fn go(&mut self, dt: f32) -> bool {
            self.secs += dt;
            self.timer += dt;
            if self.timer >= (2.0 / self.level.to_u16() as f32) {
                self.timer = 0.0;
                true
            } else {
                false
            }
        }
        pub fn highscore(&mut self) {
            self.highscore = (
                self.highscore.0.max(self.typed.0),
                self.highscore.1.max(self.typed.1),
            );
        }
        pub fn wordlist(&self) -> Vec<String> {
            let mut result: Vec<String> = match (&self.lang, &self.level) {
                (&Lang::En, &Level::Easy) => include_str!("../../assets/eng/A1.txt"),
                (&Lang::En, &Level::Normal) => include_str!("../../assets/eng/A2.txt"),
                (&Lang::En, &Level::Hard) => include_str!("../../assets/eng/B1.txt"),
                (&Lang::Su, &Level::Easy) => include_str!("../../assets/suo/A1.txt"),
                (&Lang::Su, &Level::Normal) => include_str!("../../assets/suo/A2.txt"),
                (&Lang::Su, &Level::Hard) => include_str!("../../assets/suo/B1.txt"),
            }
            .lines()
            .map(str::to_string)
            .collect();
            result.shuffle(&mut rand::rng());
            result
        }

        pub fn draw(&self) -> String {
            format!(
                "Chars: {}; Words: {}; CPM: {}, WPM: {}",
                self.typed.0,
                self.typed.1,
                ((self.typed.0 as f32 / self.secs) * 60.0) as u32,
                ((self.typed.1 as f32 / self.secs) * 60.0) as u32,
            )
        }
    }

    #[derive(Default)]
    pub enum Level {
        #[default]
        Easy = 0,
        Normal,
        Hard,
    }
    impl Level {
        pub fn to_str(&self) -> &str {
            match self {
                Self::Easy => "Easy",
                Self::Normal => "Normal",
                Self::Hard => "Hard",
            }
        }
        pub fn to_u16(&self) -> u16 {
            match self {
                Self::Easy => 2,
                Self::Normal => 3,
                Self::Hard => 4,
            }
        }
        pub fn switch(&self) -> Self {
            match self {
                Self::Easy => Self::Normal,
                Self::Normal => Self::Hard,
                Self::Hard => Self::Easy,
            }
        }
    }

    #[derive(Default)]
    pub enum Lang {
        #[default]
        En = 0,
        Su,
    }
    impl Lang {
        pub fn to_str(&self) -> &str {
            match self {
                Self::En => "English",
                Self::Su => "Finnish",
            }
        }
        pub fn switch(&self) -> Self {
            match self {
                Self::En => Self::Su,
                Self::Su => Self::En,
            }
        }
    }
}

use easyray::prelude::*;
use rand::Rng;

#[derive(Default)]
pub enum Page {
    #[default]
    Menu = 0,
    Play,
}

#[derive(Default)]
pub struct World {
    page: Page,
    size: (u16, u16),
    store: models::Score,
    avatar: Vec<(String, String, u16, u16)>,
    npc: Vec<String>,
}

impl World {
    const U8: u32 = 255;
    fn append(&mut self) -> bool {
        if !self.npc.is_empty() {
            let line = self.npc.pop().unwrap();
            let mut split = line.split(" - ");
            let first = split.next().unwrap().to_string();
            let x = rand::rng().random_range(2..self.size.0 - 2 - first.len() as u16);
            self.avatar
                .push((first, split.next().unwrap_or("PLAY").to_string(), x, 1));
            return true;
        };
        false
    }
    fn letter(&mut self, value: char) -> bool {
        if value == self.avatar[0].0.chars().next().unwrap() {
            self.store.typed.0 += 1;
            self.avatar[0].0.remove(0);
            self.avatar[0].2 += 1;
            if self.avatar[0].0.is_empty() {
                self.store.typed.1 += 1;
                self.avatar.remove(0);
                if self.avatar.is_empty() && !self.append() {
                    self.store.highscore();
                    return false;
                }
            };
        };
        true
    }
}

impl Console for World {
    fn load(&mut self, path: &str) {
        if let Ok(value) = std::fs::read(path) {
            self.store.highscore = (
                value[0] as u32 * Self::U8 + value[1] as u32,
                value[2] as u32 * Self::U8 + value[3] as u32,
            )
        };
        self.npc = self.store.wordlist();
    }
    fn update(&mut self, dt: f32) -> bool {
        if let Page::Play = self.page
            && self.store.go(dt)
        {
            for word in &mut self.avatar {
                word.3 += 1;
            }
            if self.avatar[0].3 == self.size.1 / self.store.level.to_u16() {
                self.append();
            }
            if self.avatar[0].3 > (self.size.1 - 2) {
                self.store.highscore();
                self.page = Page::Menu;
            }
            return true;
        };
        false
    }
    fn handle(&mut self, event: Event) -> Option<bool> {
        match event {
            Event::FocusLost => {
                self.page = Page::Menu;
            }
            Event::Resize(w, h) => {
                self.size = (w, h);
                self.page = Page::Menu;
            }
            Event::Key(value) => match (&self.page, value.code, value.kind) {
                (&Page::Menu, KeyCode::Esc, KeyEventKind::Press) => return None,
                (&Page::Menu, KeyCode::Tab, KeyEventKind::Press) => {
                    self.store.lang = self.store.lang.switch();
                }
                (&Page::Menu, KeyCode::F(10), KeyEventKind::Press) => {
                    self.store.level = self.store.level.switch();
                }
                (&Page::Menu, KeyCode::Enter, KeyEventKind::Press) => {
                    self.page = Page::Play;
                    self.avatar.clear();
                    self.append();
                }
                (&Page::Play, KeyCode::Char(value), KeyEventKind::Press) => {
                    if !self.letter(value) {
                        self.page = Page::Menu;
                    }
                }
                (&Page::Play, KeyCode::Esc, KeyEventKind::Press) => {
                    self.page = Page::Menu;
                }
                _ => return Some(false),
            },
            _ => return Some(false),
        }
        Some(true)
    }
    fn draw(&self, frame: &mut Frame) {
        match self.page {
            Page::Menu => {
                let outer = Block::bordered()
                    .yellow()
                    .border_type(BorderType::Rounded)
                    .title(
                        Line::from(vec![
                            "[".bold().yellow(),
                            "WELCOME".bold().reset(),
                            "]".bold().yellow(),
                        ])
                        .centered(),
                    )
                    .title_bottom(
                        Line::from(vec![
                            "[".bold().yellow(),
                            format!(
                                "Highest (Chars: {}, Words: {})",
                                self.store.highscore.0, self.store.highscore.1
                            )
                            .reset(),
                            "]".bold().yellow(),
                        ])
                        .centered(),
                    );
                let [title, body] =
                    Layout::vertical([Constraint::Ratio(1, 3); 2]).areas(outer.inner(frame.area()));
                let [_left, center, _right] = Layout::horizontal([
                    Constraint::Min(0),
                    Constraint::Length(45),
                    Constraint::Min(0),
                ])
                .areas(body);
                frame.render_widget(outer, frame.area());
                frame.render_widget(
                    Paragraph::new(
                        figleter::FIGfont::standard()
                            .unwrap()
                            .convert("TypeMaster")
                            .unwrap()
                            .to_string(),
                    )
                    .green()
                    .centered(),
                    title,
                );
                frame.render_widget(
                    Paragraph::new(format!(
                        r#"
Press <ENTER> to start
      <F10>   to toggle difficulty: {}
      <TAB>   to toggle language:   {}
      <ESC>   to quit
"#,
                        self.store.level.to_str(),
                        self.store.lang.to_str(),
                    ))
                    .green()
                    .block(Block::default().borders(Borders::NONE)),
                    center,
                );
            }
            Page::Play => {
                let outer = Block::bordered()
                    .yellow()
                    .border_type(BorderType::Rounded)
                    .title(
                        Line::from(vec![
                            "[".bold().yellow(),
                            self.avatar[0].1.to_string().bold().green(),
                            "]".bold().yellow(),
                        ])
                        .centered(),
                    )
                    .title_bottom(
                        Line::from(vec![
                            "[".bold().yellow(),
                            self.store.draw().reset(),
                            "]".bold().yellow(),
                        ])
                        .centered(),
                    );
                frame.render_widget(outer, frame.area());
                for (word, _, x, y) in &self.avatar {
                    let mut chars = word.chars();
                    frame.render_widget(
                        Paragraph::new(Line::from(vec![
                            "(".reset().bold(),
                            chars.next().unwrap().to_string().bold().yellow(),
                            chars.collect::<String>().reset().green(),
                            ")".reset().bold(),
                        ])),
                        Rect::new(*x, *y, word.len() as u16 + 2, 1),
                    );
                }
            }
        };
    }
    fn exit(&self, path: &str) {
        let (typed_chars, cpm) = self.store.highscore;
        std::fs::write(
            path,
            [
                (typed_chars / Self::U8) as u8,
                (typed_chars % Self::U8) as u8,
                (cpm / Self::U8) as u8,
                (cpm % Self::U8) as u8,
            ],
        )
        .unwrap();
    }
}

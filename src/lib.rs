#[cfg(feature = "ray")]
pub mod prelude {
    pub use raylib::prelude::*;

    pub mod solarized {
        use raylib::prelude::Color;
        pub const BASE03: Color = Color::new(0x00, 0x2B, 0x36, 0xFF);
        pub const BASE02: Color = Color::new(0x07, 0x36, 0x42, 0xFF);
        pub const BASE01: Color = Color::new(0x58, 0x6E, 0x75, 0xFF);
        pub const BASE00: Color = Color::new(0x65, 0x7B, 0x83, 0xFF);
        pub const BASE0: Color = Color::new(0x83, 0x94, 0x96, 0xFF);
        pub const BASE1: Color = Color::new(0x93, 0xA1, 0xA1, 0xFF);
        pub const BASE2: Color = Color::new(0xEE, 0xE8, 0xD5, 0xFF);
        pub const BASE3: Color = Color::new(0xFD, 0xF6, 0xE3, 0xFF);

        pub const YELLOW: Color = Color::new(181, 137, 0, 255);
        pub const ORANGE: Color = Color::new(0xCB, 0x4B, 0x16, 0xFF);
        pub const RED: Color = Color::new(220, 50, 47, 255);
        pub const CYAN: Color = Color::new(42, 161, 152, 255);
        pub const MAGENTA: Color = Color::new(211, 54, 130, 255);
        pub const GREEN: Color = Color::new(133, 153, 0, 255);
        pub const BLUE: Color = Color::new(0x26, 0x8B, 0xD2, 0xFF);
        pub const VIOLET: Color = Color::new(0x6C, 0x71, 0xC4, 0xFF);
        pub const FOREGROUND: Color = BASE01;
        pub const FOREGROUND2: Color = BASE00;
        pub const BACKGROUND: Color = BASE03;
        pub const BACKGROUND2: Color = BASE02;
    }

    pub trait Console: Default {
        type Event: 'static;
        fn load(&mut self, path: &str);
        fn update(&mut self, dt: f32) -> bool;
        fn handle(&mut self, event: Self::Event);
        fn draw(
            &self,
            canvas: &mut RaylibDrawHandle,
            width: i32,
            height: i32,
            key: Option<KeyboardKey>,
        ) -> Option<Self::Event>;
        fn exit(&self, path: &str);
        fn run(title: &str) {
            let path = format!("{}/.config/{title}", std::env::var("HOMEPATH").unwrap());
            let mut model = Self::default();
            model.load(&path);
            let (mut rl, thread) = raylib::init().title(title).build();
            rl.set_target_fps(48);
            rl.toggle_fullscreen();
            while model.update(rl.get_frame_time()) {
                let key = rl.get_key_pressed();
                let (width, height) = (rl.get_screen_width(), rl.get_screen_height());
                let mut canvas = rl.begin_drawing(&thread);
                if let Some(msg) = model.draw(&mut canvas, width, height, key) {
                    model.handle(msg);
                };
            }
            model.exit(&path);
        }
    }
}

#[cfg(feature = "tui")]
pub mod prelude {
    pub use ratatui::{
        Frame,
        crossterm::{
            ExecutableCommand,
            event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        },
        layout::{Constraint, Layout},
        prelude::*,
        symbols::Marker,
        text::Line,
        widgets::{
            Block, BorderType, Borders, Padding, Paragraph,
            canvas::{Canvas, Circle, Rectangle},
        },
    };

    pub trait Console: Default {
        fn load(self, path: &str) -> Self;
        fn update(&mut self, dt: f32) -> bool;
        fn handle(&mut self, event: Event);
        fn draw(&self, frame: &mut Frame);
        fn exit(&self, path: &str);
        fn run(title: &str) {
            ratatui::run(|terminal| {
                let path = format!(
                    "{}/.{title}",
                    std::env::var(match cfg!(target_os = "windows") {
                        true => "HOMEPATH",
                        false => "HOME",
                    })
                    .unwrap()
                );
                let mut model = Self::default().load(&path);
                if let Ok(size) = terminal.size() {
                    model.handle(Event::Resize(size.width, size.height));
                };
                let mut time = std::time::Instant::now();
                while model.update(time.elapsed().as_secs_f32()) {
                    time = std::time::Instant::now();
                    if event::poll(std::time::Duration::from_millis(20)).unwrap() {
                        model.handle(event::read().unwrap());
                    };
                    terminal.draw(|frm| model.draw(frm)).unwrap();
                }
                model.exit(&path);
            });
        }
    }
}

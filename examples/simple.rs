mod games;

#[cfg(feature = "ray")]
fn main() {
    use easyray::prelude::*;
    games::snake::World::run("Snake");
}

#[cfg(feature = "tui")]
fn main() {
    use easyray::prelude::*;
    games::words::World::run("Words");
}

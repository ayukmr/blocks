use blocks::run::run;

fn main() {
    // run sync
    pollster::block_on(run());
}

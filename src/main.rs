// Hide the console window on Windows Release builds
#![windows_subsystem = "windows"]

fn main() -> anyhow::Result<()> {
    env_logger::init();
    portkiller::run()
}

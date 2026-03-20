mod app;
mod cli;
mod config;
mod export;
mod google;
mod model;
mod oauth2_helper;
mod path_utils;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("\n✗ エラーが発生しました:");
        eprintln!("{error}");
        std::process::exit(1);
    }
}

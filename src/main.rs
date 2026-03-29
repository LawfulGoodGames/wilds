pub mod app;
pub mod character;
pub mod combat;
pub mod db;
pub mod event;
pub mod inventory;
pub mod settings;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let pool = db::init().await?;
    let settings = settings::UserSettings::load(&pool).await?;
    let terminal = ratatui::init();
    let result = app::App::new(pool, settings).run(terminal).await;
    ratatui::restore();
    result
}

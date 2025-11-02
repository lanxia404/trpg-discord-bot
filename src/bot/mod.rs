pub mod commands;
pub mod data;

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, data::BotData, Error>;

pub fn commands() -> Vec<poise::Command<data::BotData, Error>> {
    vec![
        commands::dice::roll(),
        commands::dice::coc(),
        commands::logs::log_stream(),
        commands::logs::log_stream_mode(),
        commands::logs::crit(),
        commands::skills::skill(),
        commands::admin::admin(),
        commands::help::help(),
        commands::import::import_data(),
    ]
}

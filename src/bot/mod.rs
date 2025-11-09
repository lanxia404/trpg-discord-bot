pub mod commands;
pub mod data;

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, data::BotData, Error>;

pub fn commands() -> Vec<poise::Command<data::BotData, Error>> {
    vec![
        commands::base_settings_search::base_settings_search(), // 使用了 name = "bs-search" 屬性
        commands::chat::chat(),
        commands::dice::roll(),
        commands::dice::coc(),
        commands::effect::effect(),
        commands::logs::log_stream(),
        commands::logs::log_stream_mode(),
        commands::logs::crit(),
        commands::skills::skill(),
        commands::admin::admin(),
        commands::admin_api_clear::clear_api(),
        commands::help::help(),
        commands::import::import_data(),
    ]
}

use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Поддерживаются следующие команды:"
)]
#[derive(Debug)]
pub enum Command {
    #[command(description = "Показать это сообщение.")]
    Help,
    #[command(description = "Показать, сколько осталось до нефорской пятницы.")]
    Friday,
    #[command(description = "Остановить уведомления.")]
    Stop,
    #[command(description = "Показать, какая модель сгенерировала сообщние (из последних 10)")]
    Model,
}

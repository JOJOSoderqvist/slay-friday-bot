use crate::errors::ApiError;
use crate::errors::ApiError::CommandConversionError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::EnumIter;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Поддерживаются следующие команды:"
)]
#[derive(Debug, EnumIter, PartialEq)]
pub enum Command {
    #[command(description = "Показать это сообщение.")]
    Help,
    #[command(
        description = "Метакоманда, предоставляющая интерфейс взаимодействия с другими командами"
    )]
    Slay,
    #[command(description = "Показать, сколько осталось до нефорской пятницы.")]
    Friday,
    #[command(description = "Показать, какая модель сгенерировала сообщние (из последних 20)")]
    Model,
    #[command(description = "Отправить стикер или gif с определенным названием.\nНапример, /get xdd",
    aliases = ["get"])]
    GetMedia(String),
    #[command(rename = "list_media", description = "Показать доступные стикеры или gif", aliases = ["list"])]
    ListMedia,

    #[command(rename="add_media", description = "Добавляет новый стикер или gif.",
    aliases = ["add"])]
    AddMedia,

    #[command(rename="rename_media", description = "Переименовывает существующий стикер или gif.",
    aliases = ["rename"])]
    RenameMedia,

    #[command(rename="delete_media", description = "Удаляет существующий стикер или gif.",
    aliases = ["delete", "remove"])]
    DeleteMedia,

    #[command(description = "Отмена операции в рамках диалога")]
    Cancel,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Command::Help => "/help",
            Command::Slay => "/slay",
            Command::Friday => "/friday",
            Command::Model => "/model",
            Command::GetMedia(_) => "/get",
            Command::ListMedia => "/list",
            Command::AddMedia => "/add",
            Command::RenameMedia => "/rename",
            Command::DeleteMedia => "/delete",
            Command::Cancel => "/cancel",
        })
    }
}

impl FromStr for Command {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/help" => Ok(Command::Help),
            "/slay" => Ok(Command::Slay),
            "/friday" => Ok(Command::Friday),
            "/model" => Ok(Command::Model),
            "/get" => Ok(Command::GetMedia(String::default())),
            "/delete" => Ok(Command::DeleteMedia),
            "/add" => Ok(Command::AddMedia),
            "/list" => Ok(Command::ListMedia),
            "/rename" => Ok(Command::RenameMedia),
            "/cancel" => Ok(Command::Cancel),
            cmd => Err(CommandConversionError(format!("Unknown command: {}", cmd))),
        }
    }
}

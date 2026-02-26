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
    #[command(description = "Отправить стикер с определенным названием.\nНапример, /sticker xdd или /get xdd",
    aliases = ["get"])]
    Sticker(String),
    #[command(rename = "list_stickers", description = "Показать доступные стикеры", aliases = ["list"])]
    ListStickers,

    #[command(rename="add_sticker", description = "Добавляет новый стикер.",
    aliases = ["add"])]
    AddSticker,

    #[command(rename="rename_sticker", description = "Переименовывает существующий стикер.",
    aliases = ["rename"])]
    RenameSticker,

    #[command(rename="delete_sticker", description = "Удаляет существующий стикер.",
    aliases = ["delete", "remove_sticker", "remove"])]
    DeleteSticker,

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
            Command::Sticker(_) => "/get",
            Command::ListStickers => "/list",
            Command::AddSticker => "/add",
            Command::RenameSticker => "/rename",
            Command::DeleteSticker => "/delete",
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
            "/get" => Ok(Command::Sticker(String::default())), // TODO: Additional alloc?
            "/add" => Ok(Command::AddSticker),
            "/list" => Ok(Command::ListStickers),
            "/rename" => Ok(Command::RenameSticker),
            "/cancel" => Ok(Command::Cancel),
            cmd => Err(CommandConversionError(format!("Unknown command: {}", cmd))),
        }
    }
}

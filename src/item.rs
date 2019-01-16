use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub enum ItemType {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Item {
    pub path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub subject: String,
    pub body: String,
    pub type_: ItemType,
}

pub fn display_items<I>(iter: I, color_choice: ColorChoice) -> std::io::Result<()>
where
    I: IntoIterator<Item = Item>,
{
    let writer = BufferWriter::stdout(color_choice);
    let mut buffer = writer.buffer();
    for item in iter {
        buffer.set_color(ColorSpec::new().set_bold(true))?;
        write!(&mut buffer, "{}:{}:", item.path, item.line)?;
        if item.column.is_some() {
            write!(&mut buffer, "{}:", item.column.unwrap())?;
        }
        match item.type_ {
            ItemType::Error => {
                buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                write!(&mut buffer, " error: ")?;
            }
            ItemType::Warning => {
                buffer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(&mut buffer, " warning: ")?;
            }
        }
        buffer.set_color(&ColorSpec::new())?;
        writeln!(&mut buffer, "{}", item.subject)?;
        writeln!(&mut buffer, "{}", item.body)?;
    }
    writer.print(&buffer)?;
    Ok(())
}

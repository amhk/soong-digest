use std::io::Write;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum ItemType {
    Error,
    Warning,
}

#[derive(Debug, PartialOrd, Ord)]
pub struct Item {
    pub path: String,
    pub line: usize,
    pub column: Option<usize>,
    pub subject: String,
    pub body: String,
    pub type_: ItemType,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.path == other.path
            && self.line == other.line
            && self.column == other.column
            && self.subject == other.subject
            /* ignore body */
            && self.type_ == other.type_
    }
}

impl Eq for Item {}

pub fn display_items<I>(iter: I, color_choice: ColorChoice) -> std::io::Result<()>
where
    I: Iterator<Item = Item>,
{
    let writer = BufferWriter::stdout(color_choice);
    let mut buffer = writer.buffer();
    fill_buffer(&mut buffer, iter)?;
    writer.print(&buffer)?;
    Ok(())
}

fn fill_buffer<I>(mut buffer: &mut Buffer, iter: I) -> std::io::Result<()>
where
    I: Iterator<Item = Item>,
{
    let mut v = iter.collect::<Vec<_>>();
    v.sort();
    v.dedup();

    for item in v {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error;
    use termcolor::{BufferWriter, ColorChoice};

    #[test]
    fn test_group_identical_items() {
        let haystack = include_str!("../tests/data/idmap-identical-errors/error.log");
        let items = error::parse(&haystack);
        let writer = BufferWriter::stdout(ColorChoice::Never);
        let mut buffer = writer.buffer();
        super::fill_buffer(&mut buffer, items).unwrap();
        let v = buffer.into_inner();
        let s = String::from_utf8_lossy(&v);
        assert_eq!(s.matches("frameworks/base/libs/androidfw/misc.cpp:40:9: error: no matching function for call to 'stat'").count(), 1);
    }
}

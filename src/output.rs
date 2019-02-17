use crate::item::{Item, ItemType};
use std::io::Write;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    Full,
    Cfile,
}

pub fn display_items<I>(
    iter: I,
    output_format: OutputFormat,
    color_choice: ColorChoice,
) -> std::io::Result<usize>
where
    I: Iterator<Item = Item>,
{
    let mut func = match output_format {
        OutputFormat::Full => fill_buffer_full,
        OutputFormat::Cfile => fill_buffer_cfile,
    };
    let writer = BufferWriter::stdout(color_choice);
    let mut buffer = writer.buffer();
    let n = fill_buffer(&mut func, &mut buffer, iter)?;
    writer.print(&buffer)?;
    Ok(n)
}

fn fill_buffer<I, F>(func: &mut F, mut buffer: &mut Buffer, iter: I) -> std::io::Result<usize>
where
    I: Iterator<Item = Item>,
    F: FnMut(&mut Buffer, &Item) -> std::io::Result<()>,
{
    let mut v = iter.collect::<Vec<_>>();
    v.sort();
    v.dedup();
    let total = v.len();

    for item in &v {
        func(&mut buffer, &item)?;
    }

    Ok(total)
}

fn fill_buffer_full(mut buffer: &mut Buffer, item: &Item) -> std::io::Result<()> {
    buffer.set_color(ColorSpec::new().set_bold(true))?;
    write!(&mut buffer, "{}:", item.path)?;
    if item.line.is_some() {
        write!(&mut buffer, "{}:", item.line.unwrap())?;
    }
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
    if item.body.is_some() {
        writeln!(&mut buffer, "{}", &item.body.as_ref().unwrap())?;
    }
    Ok(())
}

fn fill_buffer_cfile(mut buffer: &mut Buffer, item: &Item) -> std::io::Result<()> {
    buffer.set_color(ColorSpec::new().set_bold(true))?;
    write!(&mut buffer, "{}:", item.path)?;
    if item.line.is_some() {
        write!(&mut buffer, "{}:", item.line.unwrap())?;
    }
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error;
    use termcolor::{BufferWriter, ColorChoice};

    #[test]
    fn test_group_identical_items() {
        let haystack = include_str!("../tests/data/idmap-identical-errors/error.log");
        let items = error::parse(&haystack).unwrap();
        let writer = BufferWriter::stdout(ColorChoice::Never);
        let mut buffer = writer.buffer();
        super::fill_buffer(&mut super::fill_buffer_full, &mut buffer, items).unwrap();
        let v = buffer.into_inner();
        let s = String::from_utf8_lossy(&v);
        assert_eq!(s.matches("frameworks/base/libs/androidfw/misc.cpp:40:9: error: no matching function for call to 'stat'").count(), 1);
    }
}

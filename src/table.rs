use prettytable::format::{FormatBuilder, LinePosition, LineSeparator};
use prettytable::Table;

pub fn mk_table() -> Table {
    let mut table = Table::new();

    table.set_format(
        FormatBuilder::new()
            .column_separator('│')
            .borders('│')
            .separators(&[LinePosition::Top], LineSeparator::new('─', '┬', '┌', '┐'))
            .separators(
                &[LinePosition::Title],
                LineSeparator::new('─', '┼', '├', '┤'),
            )
            .separators(
                &[LinePosition::Bottom],
                LineSeparator::new('─', '┴', '└', '┘'),
            )
            .padding(1, 1)
            .build(),
    );
    table
}

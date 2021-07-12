use chrono::{DateTime, TimeZone, Utc};
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

pub fn format_date(dt: i64) -> String {
    let secs = dt / 1000;
    let nsec: u32 = ((dt % 1000) * 1000) as u32;
    let dt: DateTime<Utc> = Utc.timestamp(secs, nsec);
    dt.format("%Y-%m-%d").to_string()
}

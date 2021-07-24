//! Defines human readable table output for various types.

use crate::cli::sink::{Error as SinkError, Sink};
use crate::http::payload::*;
use chrono::{DateTime, TimeZone, Utc};
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator};
use prettytable::{cell, row, Table};

/// A trait to format a data structure into a [`prettytable::Table`].
pub trait AsTable {
    fn to_table(&self) -> Table;
}

/// Creates a new table with some default settings.
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

/// Formats a date given as a unix timestap or returns the empty
/// string.
pub fn format_date_opt(dtopt: &Option<i64>) -> String {
    match dtopt {
        Some(dt) => format_date(*dt),
        None => "".into(),
    }
}

/// Formats the date given as unix timestamp into "year-month-day".
pub fn format_date(dt: i64) -> String {
    let secs = dt / 1000;
    let nsec: u32 = ((dt % 1000) * 1000) as u32;
    let dt: DateTime<Utc> = Utc.timestamp(secs, nsec);
    dt.format("%Y-%m-%d").to_string()
}

/// Combines two [`IdName`] objects by their name via a separator.
/// Returns only one value if the other is empty or the empty string
/// if both are not present.
fn combine(opta: &Option<IdName>, optb: &Option<IdName>, sep: &str) -> String {
    match (opta, optb) {
        (Some(a), Some(b)) => format!("{}{}{}", a.name, sep, b.name),
        (Some(a), None) => a.name.clone(),
        (None, Some(b)) => b.name.clone(),
        (None, None) => "".into(),
    }
}

/// Returns the value in the option or the empty string.
pub fn str_or_empty(opt: Option<&String>) -> &str {
    opt.as_ref().map(|s| s.as_str()).unwrap_or("")
}

// --- impls for payloads

impl AsTable for ItemDetail {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "Property", "Value"]);
        table.add_row(row!["Id", self.id]);
        table.add_row(row!["Name", self.name]);
        table.add_row(row!["Source", self.source]);
        table.add_row(row!["State", self.state]);
        table.add_row(row!["Date", format_date_opt(&self.item_date)]);
        table.add_row(row!["Due", format_date_opt(&self.due_date)]);
        table.add_row(row!["Created", format_date(self.created)]);
        table.add_row(row![
            "Correspondent",
            combine(&self.corr_org, &self.corr_person, "/")
        ]);
        table.add_row(row![
            "Concerning",
            combine(&self.conc_person, &self.conc_equip, "/")
        ]);
        table.add_row(row![
            "Folder",
            str_or_empty(self.folder.as_ref().map(|f| &f.name))
        ]);
        let tag_list: Vec<String> = self.tags.iter().map(|t| t.name.clone()).collect();
        table.add_row(row!["Tags", tag_list.join(", ")]);
        let field_list: Vec<String> = self
            .customfields
            .iter()
            .map(|f| format!("{} {}", f.name_or_label(), f.value))
            .collect();
        table.add_row(row!["Fields", field_list.join(" ")]);
        table.add_row(row!["Attachments", format!("{}", self.attachments.len())]);
        table.add_row(row![
            "Attachments Original",
            format!("{}", self.sources.len())
        ]);
        table.add_row(row![
            "Attachments Archives",
            format!("{}", self.archives.len())
        ]);
        table
    }
}
impl Sink for ItemDetail {}

impl AsTable for ResetPasswordResp {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "success", "new password", "message"]);
        table.add_row(row![self.success, self.new_password, self.message,]);
        table
    }
}
impl Sink for ResetPasswordResp {}

impl AsTable for AuthResp {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(
            row![bFg => "success", "collective", "user", "message", "token", "valid (ms)"],
        );
        table.add_row(row![
            self.success,
            self.collective,
            self.user,
            self.message,
            str_or_empty(self.token.as_ref()),
            self.valid_ms
        ]);
        table
    }
}
impl Sink for AuthResp {}

impl AsTable for InviteResult {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "success", "key", "message"]);
        table.add_row(row![
            self.success,
            str_or_empty(self.key.as_ref()),
            self.message
        ]);
        table
    }
}
impl Sink for InviteResult {}

impl AsTable for Vec<SourceAndTags> {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(
            row![bFg => "id", "name", "enabled", "prio", "folder", "file filter", "language"],
        );
        for item in self {
            table.add_row(row![
                item.source.id[0..8],
                item.source.abbrev,
                item.source.enabled,
                item.source.priority,
                str_or_empty(item.source.folder.as_ref()),
                str_or_empty(item.source.file_filter.as_ref()),
                str_or_empty(item.source.language.as_ref())
            ]);
        }
        table
    }
}
impl Sink for Vec<SourceAndTags> {}

impl AsTable for Vec<CheckFileResult> {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "exists", "items", "file"]);
        for el in self {
            let item_list: Vec<String> = el.items.iter().map(|i| i.id[0..8].into()).collect();
            table.add_row(row![
                el.exists,
                item_list.join(", "),
                str_or_empty(el.file.as_ref())
            ]);
        }
        table
    }
}
impl Sink for Vec<CheckFileResult> {}

impl AsTable for BasicResult {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg =>
            "success",
            "message",
        ]);
        table.add_row(row![self.success, self.message,]);
        table
    }
}
impl Sink for BasicResult {}

impl AsTable for VersionInfo {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.add_row(row!["version", self.version]);
        table.add_row(row!["built at", self.built_at_string]);
        table.add_row(row!["commit", self.git_commit]);
        table
    }
}
impl Sink for VersionInfo {}

impl AsTable for BuildInfo {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.add_row(row!["build date", self.build_date]);
        table.add_row(row!["version", self.build_version]);
        table.add_row(row!["commit", self.git_commit]);
        table.add_row(row!["rustc host", self.rustc_host_triple]);
        table.add_row(row!["llvm", self.rustc_llvm_version]);
        table.add_row(row!["rust version", self.rustc_version]);
        table
    }
}
impl Sink for BuildInfo {}

impl AsTable for Vec<&TagCount> {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "id", "name", "category", "count"]);
        for item in self {
            table.add_row(row![
                item.tag.id[0..8],
                item.tag.name,
                str_or_empty(item.tag.category.as_ref()),
                item.count,
            ]);
        }
        table
    }
}
impl Sink for Vec<&TagCount> {}

impl AsTable for Vec<&CatCount> {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "name", "count"]);
        for item in self {
            table.add_row(row![
                item.name.clone().unwrap_or_else(|| "(no category)".into()),
                item.count,
            ]);
        }
        table
    }
}
impl Sink for Vec<&CatCount> {}

impl AsTable for Vec<FieldStats> {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(
            row![bFg => "id", "name/label", "type", "count", "sum", "avg", "max", "min"],
        );
        for item in self {
            table.add_row(row![
                item.id[0..8],
                item.label.as_ref().unwrap_or(&item.name),
                item.ftype,
                item.count,
                item.sum,
                item.avg,
                item.max,
                item.min
            ]);
        }
        table
    }
}
impl Sink for Vec<FieldStats> {}

impl AsTable for Summary {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg => "items"]);
        table.add_row(row![self.count]);
        table
    }
}
impl Sink for Summary {
    fn write_tabular(value: &Self) -> Result<(), SinkError> {
        println!("All");
        value.to_table().printstd();

        println!("\nTags");
        Sink::write_tabular(&value.tag_cloud.without_empty())?;

        println!("\nCategories");
        Sink::write_tabular(&value.tag_category_cloud.without_empty())?;

        println!("\nCustom Fields");
        Sink::write_tabular(&value.field_stats)?;

        Ok(())
    }

    fn write_csv(value: &Self) -> Result<(), SinkError> {
        value.to_table().to_csv(std::io::stdout())?;
        println!();
        Sink::write_csv(&value.tag_cloud.without_empty())?;
        println!();
        Sink::write_csv(&value.tag_category_cloud.without_empty())?;
        println!();
        Sink::write_csv(&value.field_stats)?;
        Ok(())
    }
}

impl AsTable for SearchResult {
    fn to_table(&self) -> Table {
        let mut table = mk_table();
        table.set_titles(row![bFg =>
            "id",
            "name",
            "state",
            "date",
            "due",
            "correspondent",
            "concerning",
            "folder",
            "tags",
            "fields",
            "files"
        ]);
        for group in &self.groups {
            for item in &group.items {
                let tag_list: Vec<String> = item.tags.iter().map(|t| t.name.clone()).collect();
                let field_list: Vec<String> = item
                    .customfields
                    .iter()
                    .map(|f| format!("{} {}", f.name_or_label(), f.value))
                    .collect();
                table.add_row(row![
                    item.id[0..8],
                    item.name,
                    item.state,
                    format_date(item.date),
                    item.due_date.map(format_date).unwrap_or_else(|| "".into()),
                    combine(&item.corr_org, &item.corr_person, "/"),
                    combine(&item.conc_person, &item.conc_equip, "/"),
                    item.folder
                        .as_ref()
                        .map(|a| a.name.as_str())
                        .unwrap_or_else(|| ""),
                    tag_list.join(", "),
                    field_list.join(", "),
                    item.attachments.len(),
                ]);
            }
        }

        table
    }
}
impl Sink for SearchResult {}

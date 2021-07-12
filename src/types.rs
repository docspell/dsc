use crate::sink::{str_or_empty, AsTable, Error as SinkError, Sink};
use crate::table;
use prettytable::{cell, row, Table};
use serde::{Deserialize, Serialize};
pub const DOCSPELL_AUTH: &'static str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &'static str = "Docspell-Admin-Secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordResp {
    pub success: bool,
    pub message: String,
    #[serde(alias = "newPassword")]
    pub new_password: String,
}
impl AsTable for ResetPasswordResp {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "success", "new password", "message"]);
        table.add_row(row![self.success, self.new_password, self.message,]);
        table
    }
}
impl Sink for ResetPasswordResp {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResp {
    pub collective: String,
    pub user: String,
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    #[serde(alias = "validMs")]
    pub valid_ms: u64,
}
impl AsTable for AuthResp {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(
            row![bFg => "success", "collective", "user", "message", "token", "valid (ms)"],
        );
        table.add_row(row![
            self.success,
            self.collective,
            self.user,
            self.message,
            str_or_empty(&self.token),
            self.valid_ms
        ]);
        table
    }
}
impl Sink for AuthResp {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadMeta {
    pub multiple: bool,
    pub direction: Option<String>,
    pub folder: Option<String>,

    #[serde(alias = "skipDuplicates", rename(serialize = "skipDuplicates"))]
    pub skip_duplicates: bool,

    pub tags: StringList,

    #[serde(alias = "fileFilter", rename(serialize = "fileFilter"))]
    pub file_filter: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StringList {
    pub items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registration {
    #[serde(alias = "collectiveName", rename(serialize = "collectiveName"))]
    pub collective_name: String,
    pub login: String,
    pub password: String,
    pub invite: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteResult {
    pub success: bool,
    pub message: String,
    pub key: Option<String>,
}
impl AsTable for InviteResult {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "success", "key", "message"]);
        table.add_row(row![self.success, str_or_empty(&self.key), self.message]);
        table
    }
}
impl Sink for InviteResult {}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenInvite {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceList {
    pub items: Vec<SourceAndTags>,
}
impl AsTable for Vec<SourceAndTags> {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(
            row![bFg => "id", "name", "enabled", "prio", "folder", "file filter", "language"],
        );
        for item in self {
            table.add_row(row![
                item.source.id[0..8],
                item.source.abbrev,
                item.source.enabled,
                item.source.priority,
                str_or_empty(&item.source.folder),
                str_or_empty(&item.source.file_filter),
                str_or_empty(&item.source.language)
            ]);
        }
        table
    }
}
impl Sink for Vec<SourceAndTags> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceAndTags {
    pub source: Source,
    pub tags: TagList,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagList {
    pub count: u32,
    pub items: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub abbrev: String,
    pub description: Option<String>,
    pub counter: u32,
    pub enabled: bool,
    pub priority: String,
    pub folder: Option<String>,
    pub file_filter: Option<String>,
    pub language: Option<String>,
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckFileResult {
    pub exists: bool,
    pub items: Vec<ItemShort>,
    pub file: Option<String>,
}
impl AsTable for Vec<CheckFileResult> {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "exists", "items", "file"]);
        for el in self {
            let item_list: Vec<String> = el.items.iter().map(|i| i.id[0..8].into()).collect();
            table.add_row(row![
                el.exists,
                item_list.join(", "),
                str_or_empty(&el.file)
            ]);
        }
        table
    }
}
impl Sink for Vec<CheckFileResult> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemShort {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub state: String,
    pub created: i64,
    #[serde(alias = "itemDate")]
    pub item_date: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResult {
    pub success: bool,
    pub message: String,
}
impl AsTable for BasicResult {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg =>
            "success",
            "message",
        ]);
        table.add_row(row![self.success, self.message,]);
        table
    }
}
impl Sink for BasicResult {}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    #[serde(alias = "builtAtMillis")]
    pub built_at_millis: i64,
    #[serde(alias = "builtAtString")]
    pub built_at_string: String,
    #[serde(alias = "gitCommit")]
    pub git_commit: String,
    #[serde(alias = "gitVersion")]
    pub git_version: String,
}
impl AsTable for VersionInfo {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg =>
            "version",
            "built at",
            "commit",
        ]);
        table.add_row(row![self.version, self.built_at_string, self.git_commit,]);
        table
    }
}
impl Sink for VersionInfo {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdName {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagCount {
    pub tag: Tag,
    pub count: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CatCount {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagCloud {
    pub items: Vec<TagCount>,
}
impl AsTable for Vec<TagCount> {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "id", "name", "category", "count"]);
        for item in self {
            table.add_row(row![
                item.tag.id[0..8],
                item.tag.name,
                str_or_empty(&item.tag.category),
                item.count,
            ]);
        }
        table
    }
}
impl Sink for Vec<TagCount> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatCloud {
    pub items: Vec<CatCount>,
}
impl AsTable for Vec<CatCount> {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "name", "count"]);
        for item in self {
            table.add_row(row![item.name, item.count,]);
        }
        table
    }
}
impl Sink for Vec<CatCount> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldStats {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
    pub ftype: String,
    pub count: u32,
    pub sum: f64,
    pub avg: f64,
    pub max: f64,
    pub min: f64,
}
impl AsTable for Vec<FieldStats> {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "id", "name/label", "type", "count", "sum", "avg", "max", "min"]);
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderStats {
    pub id: String,
    pub name: String,
    pub owner: IdName,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub count: u32,
    #[serde(alias = "tagCloud")]
    pub tag_cloud: Box<TagCloud>,
    #[serde(alias = "tagCategoryCloud")]
    pub tag_category_cloud: Box<CatCloud>,
    #[serde(alias = "fieldStats")]
    pub field_stats: Vec<FieldStats>,
    #[serde(alias = "folderStats")]
    pub folder_stats: Vec<FolderStats>,
}
impl AsTable for Summary {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg => "items"]);
        table.add_row(row![self.count]);
        table
    }
}
impl Sink for Summary {
    fn write_tabular(value: &Self) -> Result<(), SinkError> {
        println!("All");
        value.to_table().printstd();

        println!("\nTags");
        Sink::write_tabular(&value.tag_cloud.as_ref().items)?;

        println!("\nCategories");
        Sink::write_tabular(&value.tag_category_cloud.as_ref().items)?;

        println!("\nCustom Fields");
        Sink::write_tabular(&value.field_stats)?;

        Ok(())
    }

    fn write_csv(value: &Self) -> Result<(), SinkError> {
        value.to_table().to_csv(std::io::stdout())?;
        println!("");
        Sink::write_csv(&value.tag_cloud.as_ref().items)?;
        println!("");
        Sink::write_csv(&value.tag_category_cloud.as_ref().items)?;
        println!("");
        Sink::write_csv(&value.field_stats)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attach {
    pub id: String,
    pub position: u32,
    pub name: Option<String>,
    #[serde(alias = "pageCount")]
    pub page_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
    pub ftype: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub name: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub state: String,
    pub date: i64,
    #[serde(alias = "dueDate")]
    pub due_date: Option<u64>,
    pub source: String,
    pub direction: Option<String>,
    #[serde(alias = "corrOrg")]
    pub corr_org: Option<IdName>,
    #[serde(alias = "corrPerson")]
    pub corr_person: Option<IdName>,
    #[serde(alias = "concPerson")]
    pub conc_person: Option<IdName>,
    #[serde(alias = "concEquipment")]
    pub conc_equip: Option<IdName>,
    pub folder: Option<IdName>,
    pub attachments: Vec<Attach>,
    pub tags: Vec<Tag>,
    pub customfields: Vec<CustomField>,
    pub notes: Option<String>,
    pub highlighting: Vec<Highlight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub groups: Vec<Group>,
}

impl AsTable for SearchResult {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.add_row(row![bFg =>
            "id",
            "name",
            "state",
            "date",
            "correspondent",
            "concerning",
            "folder",
            "tags"
        ]);
        for group in &self.groups {
            for item in &group.items {
                let tag_list: Vec<String> = item.tags.iter().map(|t| t.name.clone()).collect();
                table.add_row(row![
                    item.id[0..8],
                    item.name,
                    item.state,
                    table::format_date(item.date),
                    combine(&item.corr_org, &item.corr_person, "/"),
                    combine(&item.conc_person, &item.conc_equip, "/"),
                    item.folder.as_ref().map(|a| a.name.as_str()).unwrap_or(""),
                    tag_list.join(", ")
                ]);
            }
        }

        table
    }
}
impl Sink for SearchResult {}

fn combine(opta: &Option<IdName>, optb: &Option<IdName>, sep: &str) -> String {
    match (opta, optb) {
        (Some(a), Some(b)) => format!("{}{}{}", a.name, sep, b.name),
        (Some(a), None) => a.name.clone(),
        (None, Some(b)) => b.name.clone(),
        (None, None) => "".into(),
    }
}

use prettytable::{cell, row, Table};
use serde::{Deserialize, Serialize};

use crate::sink::{SerError, Sink};

pub const DOCSPELL_AUTH: &'static str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &'static str = "Docspell-Admin-Secret";

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

#[derive(Debug, Serialize, Deserialize)]
pub struct GenInvite {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceList {
    pub items: Vec<SourceAndTags>,
}
impl Sink for Vec<SourceAndTags> {
    fn write_tabular(value: &Vec<SourceAndTags>) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(
            row![b => "id", "name", "enabled", "prio", "folder", "file filter", "language"],
        );
        for item in value {
            table.add_row(row![
                item.source.id,
                item.source.abbrev,
                item.source.enabled,
                item.source.priority,
                Self::str_or_empty(&item.source.folder),
                Self::str_or_empty(&item.source.file_filter),
                Self::str_or_empty(&item.source.language)
            ]);
        }
        table.printstd();
        Ok(())
    }
    fn write_csv(value: &Vec<SourceAndTags>) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for item in value {
            wtr.serialize(&item.source)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

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
    pub created: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckFileResult {
    pub exists: bool,
    pub items: Vec<ItemShort>,
    pub file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemShort {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub state: String,
    pub created: u64,
    #[serde(alias = "itemDate")]
    pub item_date: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResult {
    pub success: bool,
    pub message: String,
}
impl Sink for BasicResult {
    fn write_tabular(value: &BasicResult) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b =>
            "success",
            "message",
        ]);
        table.add_row(row![value.success, value.message,]);
        table.printstd();
        Ok(())
    }
    fn write_csv(value: &BasicResult) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        wtr.serialize(value)?;
        wtr.flush()?;
        Ok(())
    }
}

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
impl Sink for VersionInfo {
    fn write_tabular(value: &VersionInfo) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b =>
            "version",
            "built at millis",
            "built at",
            "commit",
        ]);
        table.add_row(row![
            value.version,
            value.built_at_millis,
            value.built_at_string,
            value.git_commit,
        ]);
        table.printstd();
        Ok(())
    }
    fn write_csv(value: &VersionInfo) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        wtr.serialize(value)?;
        wtr.flush()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub created: u64,
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
impl Sink for Vec<TagCount> {
    fn write_tabular(value: &Vec<TagCount>) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b => "id", "name", "category", "count"]);
        for item in value {
            table.add_row(row![
                item.tag.id,
                item.tag.name,
                Self::str_or_empty(&item.tag.category),
                item.count,
            ]);
        }
        table.printstd();
        Ok(())
    }
    fn write_csv(value: &Vec<TagCount>) -> Result<(), SerError> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::io::stdout());
        wtr.write_record(&["id", "name", "category", "created", "count"])?;
        for item in value {
            wtr.serialize(&item)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatCloud {
    pub items: Vec<CatCount>,
}
impl Sink for Vec<CatCount> {
    fn write_tabular(value: &Vec<CatCount>) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b => "name", "count"]);
        for item in value {
            table.add_row(row![item.name, item.count,]);
        }
        table.printstd();
        Ok(())
    }
    fn write_csv(value: &Vec<CatCount>) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for item in value {
            wtr.serialize(&item)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

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
impl Sink for Vec<FieldStats> {
    fn write_tabular(value: &Self) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b => "id", "name/label", "type", "count", "sum", "avg", "max", "min"]);
        for item in value {
            table.add_row(row![
                item.id,
                item.label.as_ref().unwrap_or(&item.name),
                item.ftype,
                item.count,
                item.sum,
                item.avg,
                item.max,
                item.min
            ]);
        }
        table.printstd();
        Ok(())
    }

    fn write_csv(value: &Self) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for item in value {
            wtr.serialize(&item)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

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
impl Sink for Summary {
    fn write_tabular(value: &Summary) -> Result<(), SerError> {
        let mut table = Table::new();
        table.add_row(row![b => "items"]);
        table.add_row(row![value.count]);
        table.printstd();

        println!("\nTags");
        Sink::write_tabular(&value.tag_cloud.as_ref().items)?;

        println!("\nCategories");
        Sink::write_tabular(&value.tag_category_cloud.as_ref().items)?;

        println!("\nCustom Fields");
        Sink::write_tabular(&value.field_stats)?;

        Ok(())
    }
    fn write_csv(value: &Summary) -> Result<(), SerError> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        wtr.write_record(&["count"])?;
        wtr.write_record(&[format!("{}", value.count)])?;
        wtr.flush()?;
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
    pub name: String,
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
    pub date: u64,
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

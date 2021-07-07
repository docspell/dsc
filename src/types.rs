use serde::{Deserialize, Serialize};

pub const DOCSPELL_AUTH: &'static str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &'static str = "Docspell-Admin-Secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckFileResult {
    pub exists: bool,
    pub items: Box<[ItemShort]>,
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
    pub items: Box<[TagCount]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatCloud {
    pub items: Box<[CatCount]>,
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
    pub field_stats: Box<[FieldStats]>,
    #[serde(alias = "folderStats")]
    pub folder_stats: Box<[FolderStats]>,
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
    pub lines: Box<[String]>,
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
    pub attachments: Box<[Attach]>,
    pub tags: Box<[Tag]>,
    pub customfields: Box<[CustomField]>,
    pub notes: Option<String>,
    pub highlighting: Box<[Highlight]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub items: Box<[Item]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub groups: Box<[Group]>,
}

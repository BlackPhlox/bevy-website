use cratesio_dbdump_csvtab::rusqlite::Connection;
use cratesio_dbdump_lookup::{get_versions, CrateDependency, CrateLookup};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::{fs, io, path::PathBuf, str::FromStr};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub name: String,
    pub link: String,
    pub description: String,
    pub order: Option<usize>,
    pub image: Option<String>,
    pub color: Option<String>,
    pub emoji: Option<String>,

    // this field is not read from the toml file
    #[serde(skip)]
    pub original_path: Option<PathBuf>,
    #[serde(skip)]
    pub tags: Vec<String>,
    #[serde(skip)]
    pub dependencies: Vec<CrateDependency>,
    #[serde(skip)]
    pub downloads: u32,
    #[serde(skip)]
    pub repo_url: Option<String>,
    #[serde(skip)]
    pub homepage_url: Option<String>,
    #[serde(skip)]
    pub last_update: i64,
    #[serde(skip)]
    pub latest_version: String,
    #[serde(skip)]
    pub license: String,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub content: Vec<AssetNode>,
    pub template: Option<String>,
    pub header: Option<String>,
    pub order: Option<usize>,
    pub sort_order_reversed: bool,
}

#[derive(Debug, Clone)]
pub enum AssetNode {
    Section(Section),
    Asset(Asset),
}
impl AssetNode {
    pub fn name(&self) -> String {
        match self {
            AssetNode::Section(content) => content.name.clone(),
            AssetNode::Asset(content) => content.name.clone(),
        }
    }
    pub fn order(&self) -> usize {
        match self {
            AssetNode::Section(content) => content.order.unwrap_or(99999),
            AssetNode::Asset(content) => content.order.unwrap_or(99999),
        }
    }
}

fn visit_dirs(dir: PathBuf, section: &mut Section, db: &Connection) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.file_name().unwrap() == ".git" || path.file_name().unwrap() == ".github" {
                continue;
            }
            if path.is_dir() {
                let folder = path.file_name().unwrap();
                let (order, sort_order_reversed) = if path.join("_category.toml").exists() {
                    let from_file: toml::Value = toml::de::from_str(
                        &fs::read_to_string(path.join("_category.toml")).unwrap(),
                    )
                    .unwrap();
                    (
                        from_file
                            .get("order")
                            .and_then(|v| v.as_integer())
                            .map(|v| v as usize),
                        from_file
                            .get("sort_order_reversed")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    )
                } else {
                    (None, false)
                };
                let mut new_section = Section {
                    name: folder.to_str().unwrap().to_string(),
                    content: vec![],
                    template: None,
                    header: None,
                    order,
                    sort_order_reversed,
                };
                visit_dirs(path.clone(), &mut new_section, db)?;
                section.content.push(AssetNode::Section(new_section));
            } else {
                if path.file_name().unwrap() == "_category.toml"
                    || path.extension().unwrap() != "toml"
                {
                    continue;
                }
                let mut asset: Asset =
                    toml::de::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
                asset.original_path = Some(path);

                populate_with_crate_io_data(db, &mut asset);

                section.content.push(AssetNode::Asset(asset));
            }
        }
    }
    Ok(())
}

fn populate_with_crate_io_data(db: &Connection, asset: &mut Asset) {
    if asset.image.is_none() && asset.emoji.is_none() {
        let emoji_code: u32 = thread_rng().gen_range(0x1F600..0x1F64F);
        let emoji = char::from_u32(emoji_code).unwrap_or('💔');
        asset.emoji = Some(emoji.to_string());
    }

    let co = db.get_crate(&asset.name);
    if let Ok(Some(c)) = co {
        let latest_version = &get_versions(&db, asset.name.to_string(), true).unwrap()[0];
        asset.latest_version = latest_version.1.clone();
        let license = &latest_version.2;
        asset.license = license.to_string();
        if asset.description.is_empty() {
            asset.description = c.description;
        }
        asset.homepage_url = c.homepage_url;
        let dt =
            chrono::NaiveDateTime::parse_from_str(c.last_update.as_str(), "%Y-%m-%d %H:%M:%S%.6f");
        if let Ok(dtime) = dt {
            asset.last_update = dtime.format("%s").to_string().parse().unwrap();
        } else {
            println!("{:?}", dt.unwrap_err());
        }
        asset.downloads = c.downloads;
        asset.tags = c
            .keywords
            .into_iter()
            .filter(|s| !(s.eq("bevy") || s.eq("bevyengine") || s.eq("gamedev") || s.eq("game")))
            .collect();
        asset.repo_url = c.repo_url;
        let mut crate_dependencies = c.dependencies;

        //Removes features version duplicates and Crate dependency kinds (Dev & Normal)
        crate_dependencies.dedup_by_key(|cd| format!("{}{}", cd.crate_id, cd.version));

        asset.dependencies = crate_dependencies
            .into_iter()
            .map(|f| {
                let is_bevy = (f.crate_id.eq("bevy") || f.crate_id.eq("bevy_app"))
                    && f.version.ends_with(".0");
                let v = if is_bevy {
                    f.version[..f.version.len() - 2].to_string()
                } else {
                    f.version
                }
                .replace("^", "");
                CrateDependency {
                    crate_id: f.crate_id,
                    version: v,
                    kind: f.kind,
                }
            })
            .collect()
    }
}

pub fn parse_assets(asset_dir: &str, db: &Connection) -> io::Result<Section> {
    let mut asset_root_section = Section {
        name: "Assets".to_string(),
        content: vec![],
        template: Some("assets.html".to_string()),
        header: Some("Assets".to_string()),
        order: None,
        sort_order_reversed: false,
    };
    visit_dirs(
        PathBuf::from_str(asset_dir).unwrap(),
        &mut asset_root_section,
        db,
    )?;
    Ok(asset_root_section)
}

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Latest {
    name: String,
    pub full_name: String,
    description: String,
    icon: String,
    /// major.minor.rev:
    version_number: String,
    pub dependencies: Vec<String>,
    pub download_url: String,
    downloads: u64,
    date_created: String,
    website_url: String,
    is_active: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub full_name: String,
    pub owner: String,
    package_url: String,
    date_created: String,
    date_updated: String,
    rating_score: u8,
    is_pinned: bool,
    is_deprecated: bool,
    total_downloads: u64,
    pub latest: Latest,
}

#[derive(Deserialize, Debug)]
pub struct Resp {
    count: u16,
    next: Option<String>,
    previous: Option<String>,
    pub results: Vec<Package>,
}

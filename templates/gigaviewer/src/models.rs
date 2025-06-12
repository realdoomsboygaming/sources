use aidoku::alloc::{String, Vec};
use serde::Deserialize;

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct GigaEpisode {
	pub readable_product: GigaReadableProduct,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct GigaReadableProduct {
	pub page_structure: GigaPageStructure,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct GigaPageStructure {
	pub pages: Vec<GigaPage>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct GigaPage {
	pub src: Option<String>,
	pub r#type: Option<String>,
	pub width: Option<i32>,
	pub height: Option<i32>,
}

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct GigaReadMoreResponse {
	pub html: String,
	pub next_url: String,
}

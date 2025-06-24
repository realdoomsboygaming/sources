use aidoku::{
	alloc::{vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, ContentRating, FilterValue, Listing, Manga, MangaPageResult, MangaStatus, Page,
	PageContent, Result, Viewer,
};
use serde::Deserialize;

use crate::BASE_API_URL;

#[derive(Deserialize)]
struct ApiResponse {
	data: Vec<MangaData>,
	#[serde(default)]
	total: Option<i32>,
}

#[derive(Deserialize)]
struct MangaData {
	title: String,
	thumbnail: String,
	series_slug: String,
}

#[derive(Deserialize)]
struct SeriesResponse {
	id: i32,
	title: String,
	thumbnail: String,
	description: String,
	author: String,
	studio: String,
	series_slug: String,
	status: String,
	tags: Vec<TagData>,
}

#[derive(Deserialize)]
struct TagData {
	name: String,
}

#[derive(Deserialize)]
struct ChapterResponse {
	data: Vec<ChapterData>,
	meta: MetaData,
}

#[derive(Deserialize)]
struct ChapterData {
	chapter_slug: String,
	price: i32,
	created_at: String,
}

#[derive(Deserialize)]
struct MetaData {
	first_page: i32,
	last_page: i32,
}

pub fn parse_manga_list(
	base_url: String,
	query: Option<String>,
	filters: Vec<FilterValue>,
	page: i32,
) -> Result<MangaPageResult> {
	let search_query = query.unwrap_or_default();
	let mut genres = String::new();

	for filter in filters {
		match filter {
			FilterValue::Select { id, value, .. } => {
				if id == "genre" && value != "All" {
					// Map genre name to ID for the API
					let genre_id = match value.as_str() {
						"Romance" => "1",
						"Drama" => "2",
						"Fantasy" => "3",
						"Hardcore" => "4",
						"SM" => "5",
						"Harem" => "8",
						"Hypnosis" => "9",
						"Novel Adaptation" => "10",
						"Netori" => "11",
						"Netorare" => "12",
						"Isekai" => "13",
						"Yuri" => "14",
						"MILF" => "16",
						"Office" => "17",
						"Short Story" => "18",
						"Comedy" => "19",
						"Campus" => "20",
						"Crime" => "21",
						"Revenge" => "22",
						"Supernatural" => "23",
						"Action" => "24",
						"Military" => "25",
						"Cohabitation" => "27",
						"Training" => "28",
						"Ability" => "29",
						_ => continue,
					};
					genres.push_str(genre_id);
					genres.push(',');
				}
			}
			_ => continue,
		}
	}

	if !genres.is_empty() {
		genres.pop();
	}

	let url = format!("{}/query?query_string={}&order=desc&orderBy=total_views&series_type=Comic&page={}&perPage=10&tags_ids=[{}]&adult=true", BASE_API_URL, search_query, page, genres);
	let response = Request::get(&url)?.send()?;
	let manga = parse_manga(&base_url, response)?;
	let has_next_page = !manga.is_empty();

	Ok(MangaPageResult {
		entries: manga,
		has_next_page,
	})
}

pub fn parse_manga_listing(
	base_url: String,
	listing: Listing,
	page: i32,
) -> Result<MangaPageResult> {
	let list_query = match listing.id.as_str() {
		"latest" => "latest",
		"popular" => "total_views",
		"newest" => "created_at",
		"alphabetical" => "title",
		_ => "latest",
	};
	let url = format!("{}/query?query_string=&order=desc&orderBy={}&series_type=Comic&page={}&perPage=10&tags_ids=[]&adult=true", BASE_API_URL, list_query, page);

	let response = Request::get(&url)?.send()?;
	let manga = parse_manga(&base_url, response)?;
	let has_next_page = !manga.is_empty();

	Ok(MangaPageResult {
		entries: manga,
		has_next_page,
	})
}

pub fn parse_manga_details(base_url: &String, manga_id: String) -> Result<Manga> {
	let url = format!("{}/series/{}", BASE_API_URL, manga_id);
	let data = Request::get(&url)?.send()?.get_json::<SeriesResponse>()?;

	let cover = Some(data.thumbnail);
	let title = data.title;
	let description = Some(data.description);
	let authors = vec![data.author];
	let artists = vec![data.studio];
	let key = data.series_slug;
	let url = Some(format!("{}/series/{}", base_url, key));
	let status_str = data.status;

	let status = match status_str.as_str() {
		"New" => MangaStatus::Unknown,
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Cancelled" => MangaStatus::Cancelled,
		"Dropped" => MangaStatus::Cancelled,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};

	let tags = data.tags.into_iter().map(|tag| tag.name).collect();
	let chapters = parse_chapter_list_internal(base_url.clone(), key.clone(), data.id)?;

	Ok(Manga {
		key,
		cover,
		title,
		authors: Some(authors),
		artists: Some(artists),
		description,
		url,
		tags: Some(tags),
		status,
		content_rating: ContentRating::NSFW,
		viewer: Viewer::Webtoon,
		chapters: Some(chapters),
		..Default::default()
	})
}

fn parse_chapter_list_internal(base_url: String, manga_id: String, series_id: i32) -> Result<Vec<Chapter>> {
	let url = format!(
		"{}/chapter/query?page=1&perPage=30&series_id={}",
		BASE_API_URL, series_id
	);
	let data = Request::get(&url)?.send()?.get_json::<ChapterResponse>()?;
	let mut page = data.meta.first_page;
	let last_page = data.meta.last_page;

	let mut all_chapters: Vec<Chapter> = Vec::new();

	while page <= last_page {
		let url = format!(
			"{}/chapter/query?page={}&perPage=30&series_id={}",
			BASE_API_URL, page, series_id
		);
		let data = Request::get(&url)?.send()?.get_json::<ChapterResponse>()?;

		for chapter in data.data {
			// Only get free chapters
			if chapter.price != 0 {
				continue;
			}

			let key = chapter.chapter_slug;

			let index = key.split('-').collect::<Vec<&str>>();
			let chapter_number = if index.len() > 1 {
				Some(String::from(index[1]).parse::<f32>().unwrap_or(-1.0))
			} else {
				None
			};

			let url = Some(format!("{}/series/{}/{}", base_url, manga_id, key));

			// Parse the date - this might fail, that's ok
			let date_uploaded = chrono::DateTime::parse_from_rfc3339(&chapter.created_at)
				.map(|dt| dt.timestamp())
				.ok();

			all_chapters.push(Chapter {
				key,
				title: None,
				chapter_number,
				volume_number: None,
				date_uploaded,
				url,
				..Default::default()
			});
		}
		page += 1;
	}

	Ok(all_chapters)
}

pub fn parse_page_list(
	base_url: String,
	_manga_key: String,
	chapter_key: String,
) -> Result<Vec<Page>> {
	// Extract manga ID from chapter key format
	let parts: Vec<&str> = chapter_key.split('-').collect();
	let manga_id = if parts.len() > 1 {
		parts[0]
	} else {
		return Ok(Vec::new());
	};
	
	let url = format!("{}/series/{}/{}", base_url, manga_id, chapter_key);
	let obj = Request::get(&url)?.html()?;

	let pages = obj
		.select("img")
		.map(|els| {
			els.filter_map(|el| {
				let mut url = el.attr("data-src").unwrap_or_default();

				if url.is_empty() {
					url = el.attr("src").unwrap_or_default();
				}

				if !url.is_empty() && !url.contains("icon.png") && !url.contains("banner") {
					Some(Page {
						content: PageContent::url(url),
						..Default::default()
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
		})
		.unwrap_or_default();

	Ok(pages)
}

fn parse_manga(base_url: &String, response: aidoku::imports::net::Response) -> Result<Vec<Manga>> {
	let data = response.get_json::<ApiResponse>()?;
	let mut mangas: Vec<Manga> = Vec::new();

	for manga in data.data {
		let url = format!("{}/series/{}", base_url, manga.series_slug);

		mangas.push(Manga {
			key: manga.series_slug,
			cover: Some(manga.thumbnail),
			title: manga.title,
			url: Some(url),
			..Default::default()
		});
	}

	Ok(mangas)
}

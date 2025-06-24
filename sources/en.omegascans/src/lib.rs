#![no_std]
use aidoku::{
	alloc::{String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, ImageRequestProvider, Listing,
	ListingProvider, Manga, MangaPageResult, Page, PageContext, Result, Source,
};

mod parser;

const BASE_URL: &str = "https://omegascans.org";
const BASE_API_URL: &str = "https://api.omegascans.org";

struct OmegaScans;

impl Source for OmegaScans {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		parser::parse_manga_list(String::from(BASE_URL), query, filters, page)
	}

	fn get_manga_update(
		&self,
		manga: Manga,
		_needs_details: bool,
		_needs_chapters: bool,
	) -> Result<Manga> {
		parser::parse_manga_details(&String::from(BASE_URL), manga.key)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		parser::parse_page_list(String::from(BASE_URL), manga.key, chapter.key)
	}
}

impl ListingProvider for OmegaScans {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		parser::parse_manga_listing(String::from(BASE_URL), listing, page)
	}
}

impl ImageRequestProvider for OmegaScans {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(&url)?.header("Referer", BASE_URL))
	}
}

impl DeepLinkHandler for OmegaScans {
	fn handle_deep_link(&self, _url: String) -> Result<Option<DeepLinkResult>> {
		Ok(None)
	}
}

register_source!(OmegaScans, ListingProvider, ImageRequestProvider, DeepLinkHandler);

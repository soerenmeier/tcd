
use crate::Data;

use std::{io, fmt};
use std::path::Path;

use fire::{FireBuilder, Error};
use fire::util::PinnedFuture;
use fire::fs::{Range, Caching};
use fire::routes::{Route, check_static};
use fire::request::Request;
use fire::response::Response;
use fire::header::{RequestHeader, Mime, StatusCode, Method};
use fire::into::IntoResponse;


#[derive(Debug)]
pub struct RangeIncorrect(Range);

impl fmt::Display for RangeIncorrect {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for RangeIncorrect {}

macro_rules! static_file {
	($name:ident, $uri_path:expr, $file_path:expr) => (
		struct $name {
			caching: Caching,
			mime: Mime
		}

		impl $name {
			fn new() -> Self {
				let ext = Path::new($file_path)
					.extension()
					.and_then(|f| f.to_str());

				let mime = match ext {
					Some(e) => Mime::from_ext(e),
					None => Mime::Binary
				};

				Self {
					caching: Caching::default(),
					mime
				}
			}
		}

		impl Route<Data> for $name {
			fn check(&self, header: &RequestHeader) -> bool {
				header.method() == &Method::Get &&
				check_static(header.uri().path(), $uri_path)
			}

			fn call<'a>(
				&'a self,
				req: &'a mut Request,
				_: &'a Data
			) -> PinnedFuture<'a, fire::Result<Response>> {
				let file = include_bytes!($file_path);

				let caching = self.caching.clone();

				PinnedFuture::new(async move {

					if caching.if_none_match(req.header()) {
						return Ok(caching.into_response())
					}

					let range = Range::parse(req.header());

					let mut resp = match range {
						Some(range) => {
							// partial file
							let size = file.len();
							let start = range.start;
							let end = range.end.unwrap_or(size - 1);

							if end >= size || start >= end {
								return Err(Error::from_client_io(io::Error::new(
									io::ErrorKind::Other,
									RangeIncorrect(range)
								)));
							}

							let len = (end + 1) - start;

							Response::builder()
								.status_code(StatusCode::PartialContent)
								.content_type(self.mime)
								.header("accept-ranges", "bytes")
								.header("content-length", len)
								.header(
									"content-range",
									format!("bytes {}-{}/{}", start, end, size)
								)
								.body(&file[start..=end])
								.build()
						},
						None => {
							Response::builder()
								.content_type(self.mime)
								.header("content-length", file.len())
								.body(file.as_slice())
								.build()
						}
					};

					caching.complete_header(&mut resp.header);

					Ok(resp)
				})
			}
		}
	)
}

static_file!(Index, "/", "../../../tcd-ui/public/index.html");
static_file!(GlobalCss, "/global.css", "../../../tcd-ui/public/global.css");
static_file!(ManifestJson,
	"/manifest.json",
	"../../../tcd-ui/public/manifest.json"
);
static_file!(BundleJs,
	"/build/bundle.js",
	"../../../tcd-ui/public/build/bundle.js"
);
static_file!(BundleCss,
	"/build/bundle.css",
	"../../../tcd-ui/public/build/bundle.css"
);

pub(crate) fn handle(fire: &mut FireBuilder<Data>) {
	fire.add_route(Index::new());
	fire.add_route(GlobalCss::new());
	fire.add_route(ManifestJson::new());
	fire.add_route(BundleJs::new());
	fire.add_route(BundleCss::new());
}
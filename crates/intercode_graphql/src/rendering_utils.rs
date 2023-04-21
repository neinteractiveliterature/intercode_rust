use http::Uri;

pub fn url_with_possible_host(path: &str, host: Option<String>) -> String {
  if let Some(host) = host {
    Uri::builder()
      .scheme("https")
      .authority(host)
      .path_and_query(path)
      .build()
      .map(|uri| uri.to_string())
      .unwrap_or_else(|_| path.to_string())
  } else {
    path.to_string()
  }
}

use crate::build_active_storage_blob_url;
use intercode_entities::active_storage_blobs;
use linkify::{LinkFinder, LinkKind};
use pulldown_cmark::{html, Event, LinkType, Options, Tag};
use std::collections::HashMap;

fn linkify_event<'a>(event: Event<'a>) -> Box<dyn Iterator<Item = Event<'a>> + 'a> {
  match event {
    Event::Text(ref text) => {
      #[allow(clippy::needless_collect)]
      let events: Vec<_> = LinkFinder::new()
        .url_must_have_scheme(false)
        .spans(text)
        .flat_map(|span| match span.kind() {
          Some(LinkKind::Email) | Some(LinkKind::Url) => vec![
            Event::Start(Tag::Link(
              LinkType::Inline,
              span.as_str().to_owned().into(),
              "".to_string().into(),
            )),
            Event::Text(span.as_str().to_owned().into()),
            Event::End(Tag::Link(
              LinkType::Inline,
              span.as_str().to_owned().into(),
              "".to_string().into(),
            )),
          ]
          .into_iter(),
          _ => vec![event.clone()].into_iter(),
        })
        .collect();

      Box::new(events.into_iter())
    }
    _ => Box::new(std::iter::once(event)),
  }
}

pub fn render_markdown(
  markdown: &str,
  image_attachments: &HashMap<String, active_storage_blobs::Model>,
) -> String {
  // no_intra_emphasis is implicitly part of commonmark so we don't need to/can't specify it here
  let mut options = Options::empty();
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_FOOTNOTES);
  options.insert(Options::ENABLE_SMART_PUNCTUATION);
  options.insert(Options::ENABLE_TABLES);

  let parser = pulldown_cmark::Parser::new_ext(markdown, options)
    .flat_map(linkify_event)
    .map(|event| -> Event {
      match event {
        Event::Start(Tag::Image(link_type, destination, title)) => {
          let blob = image_attachments.get(&destination.to_string());
          if let Some(blob) = blob {
            Event::Start(Tag::Image(
              link_type,
              build_active_storage_blob_url(blob).into(),
              title,
            ))
          } else {
            Event::Start(Tag::Image(link_type, destination, title))
          }
        }
        _ => event,
      }
    });

  // TODO figure out how to get target="_blank" rel="noreferrer noopener" onto all links
  // TODO figure out how to get class="img-fluid" onto all images
  // TODO strip_single_p
  let mut html_output = String::new();
  html::push_html(&mut html_output, parser);
  html_output
}

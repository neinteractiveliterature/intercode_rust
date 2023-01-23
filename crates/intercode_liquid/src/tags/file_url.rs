use std::{io::Write, sync::Arc};

use intercode_entities::cms_parent::{CmsParent, CmsParentTrait};
use intercode_entities::{active_storage_attachments, active_storage_blobs, cms_files};
use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use seawater::ConnectionWrapper;
use tokio::runtime::Handle;

use crate::build_active_storage_blob_url;

#[derive(Clone, Debug)]
pub struct FileUrlTag {
  cms_parent: Arc<CmsParent>,
  db: ConnectionWrapper,
}

impl FileUrlTag {
  pub fn new(cms_parent: Arc<CmsParent>, db: ConnectionWrapper) -> Self {
    FileUrlTag { cms_parent, db }
  }
}

impl TagReflection for FileUrlTag {
  fn tag(&self) -> &'static str {
    "file_url"
  }

  fn description(&self) -> &'static str {
    "Given a filename of an uploaded file, returns the URL to use for displaying or serving that file."
  }
}

impl ParseTag for FileUrlTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let filename = arguments.expect_next("Identifier or literal expected.")?;
    let filename = filename.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(FileUrl {
      cms_parent: self.cms_parent.clone(),
      filename,
      db: self.db.clone(),
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct FileUrl {
  filename: Expression,
  cms_parent: Arc<CmsParent>,
  db: ConnectionWrapper,
}

impl Renderable for FileUrl {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let filename = self.filename.evaluate(runtime)?;
    if !filename.is_scalar() {
      return Error::with_msg("filename must be a string")
        .context("file_url", format!("{}", filename.source()))
        .into_err();
    }
    let filename = filename.to_kstr().into_owned();
    let cms_parent = self.cms_parent.clone();
    let db = self.db.clone();

    let attachment_handle = tokio::spawn(async move {
      active_storage_attachments::Entity::find()
        .filter(active_storage_attachments::Column::RecordType.eq("CmsFile"))
        .filter(active_storage_attachments::Column::Name.eq("file"))
        .filter(
          active_storage_attachments::Column::RecordId.in_subquery(
            QuerySelect::query(
              &mut cms_parent
                .as_ref()
                .cms_files()
                .select_only()
                .column(cms_files::Column::Id),
            )
            .to_owned(),
          ),
        )
        .find_also_related(active_storage_blobs::Entity)
        .one(db.as_ref())
        .await
    });
    let attachment = Handle::current().block_on(attachment_handle).unwrap();

    let attachment = match attachment {
      Ok(att) => Ok(att),
      Err(error) => Err(liquid_core::Error::with_msg(error.to_string())),
    }?;

    let (attachment, blob) = match attachment {
      Some(att) => Ok(att),
      None => Err(liquid_core::Error::with_msg(format!(
        "File not found: {}",
        filename
      ))),
    }?;

    let blob = match blob {
      Some(b) => Ok(b),
      None => Err(liquid_core::Error::with_msg(format!(
        "Attachment {} is missing blob record {} in the database",
        attachment.id, attachment.blob_id
      ))),
    }?;

    let url = build_active_storage_blob_url(&blob);

    if let Err(error) = writer.write(url.as_bytes()) {
      Err(Error::with_msg(error.to_string()))
    } else {
      Ok(())
    }
  }
}

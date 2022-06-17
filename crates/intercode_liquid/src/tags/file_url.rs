use std::{io::Write, sync::Arc};

use intercode_entities::cms_parent::{CmsParent, CmsParentTrait};
use intercode_entities::{active_storage_attachments, active_storage_blobs, cms_files};
use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QuerySelect};
use tokio::runtime::Handle;

#[derive(Clone, Debug)]
pub struct FileUrlTag {
  cms_parent: Arc<Option<CmsParent>>,
  db: Arc<DatabaseConnection>,
}

impl FileUrlTag {
  pub fn new(cms_parent: Arc<Option<CmsParent>>, db: Arc<DatabaseConnection>) -> Self {
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
  cms_parent: Arc<Option<CmsParent>>,
  db: Arc<DatabaseConnection>,
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

    let handle = Handle::current();
    let attachment = handle.block_on(
      active_storage_attachments::Entity::find()
        .filter(active_storage_attachments::Column::RecordType.eq("CmsFile"))
        .filter(active_storage_attachments::Column::Name.eq("file"))
        .filter(
          active_storage_attachments::Column::RecordId.in_subquery(
            QuerySelect::query(
              &mut self
                .cms_parent
                .as_ref()
                .as_ref()
                .ok_or_else(|| {
                  liquid_core::Error::with_msg(
                    "file_url can only be used inside a CMS parent (root site or convention)",
                  )
                })?
                .cms_files()
                .column(cms_files::Column::Id),
            )
            .to_owned(),
          ),
        )
        .one(self.db.as_ref()),
    );

    let attachment = match attachment {
      Ok(att) => Ok(att),
      Err(error) => Err(liquid_core::Error::with_msg(error.to_string())),
    }?;

    let attachment = match attachment {
      Some(att) => Ok(att),
      None => Err(liquid_core::Error::with_msg(format!(
        "File not found: {}",
        filename
      ))),
    }?;

    let blob = handle.block_on(
      attachment
        .find_related(active_storage_blobs::Entity)
        .one(self.db.as_ref()),
    );

    let blob = match blob {
      Ok(b) => Ok(b),
      Err(error) => Err(liquid_core::Error::with_msg(error.to_string())),
    }?;

    let blob = match blob {
      Some(b) => Ok(b),
      None => Err(liquid_core::Error::with_msg(format!(
        "Attachment {} is missing blob record {} in the database",
        attachment.id, attachment.blob_id
      ))),
    }?;

    // TODO do something actually real here
    let url = format!("https://assets.neilhosting.net/{}", blob.key);

    if let Err(error) = writer.write(url.as_bytes()) {
      Err(Error::with_msg(error.to_string()))
    } else {
      Ok(())
    }
  }
}

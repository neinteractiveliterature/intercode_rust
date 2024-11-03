use std::marker::PhantomData;

use async_graphql::{Context, Error, Object, OutputType, Result};
use intercode_entities::user_con_profiles;
use intercode_graphql_core::{query_data::QueryData, ModelBackedType};
use sea_orm::{ActiveModelTrait, Set};

pub struct AcceptClickwrapAgreement;

pub struct AcceptClickwrapAgreementPayload<
  UserConProfileType: ModelBackedType<Model = user_con_profiles::Model>,
> {
  pub my_profile: user_con_profiles::Model,
  _phantom: PhantomData<UserConProfileType>,
}

#[Object]
impl<UserConProfileType> AcceptClickwrapAgreementPayload<UserConProfileType>
where
  UserConProfileType: ModelBackedType<Model = user_con_profiles::Model> + Send + Sync + OutputType,
{
  async fn my_profile(&self) -> UserConProfileType {
    UserConProfileType::new(self.my_profile.clone())
  }
}

impl AcceptClickwrapAgreement {
  pub async fn accept_clickwrap_agreement<
    UserConProfileType: ModelBackedType<Model = user_con_profiles::Model>,
  >(
    ctx: &Context<'_>,
  ) -> Result<AcceptClickwrapAgreementPayload<UserConProfileType>> {
    let query_data = ctx.data::<QueryData>()?;
    let Some(user_con_profile) = query_data.user_con_profile() else {
      return Err(Error::new("Must be logged in"));
    };

    let mut user_con_profile = user_con_profiles::ActiveModel::from(user_con_profile.clone());
    user_con_profile.accepted_clickwrap_agreement = Set(true);
    let user_con_profile = user_con_profile.update(query_data.db()).await?;

    Ok(AcceptClickwrapAgreementPayload {
      my_profile: user_con_profile,
      _phantom: PhantomData {},
    })
  }
}

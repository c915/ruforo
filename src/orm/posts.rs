//! SeaORM Entity. Generated by sea-orm-codegen 0.4.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub thread_id: i32,
    pub position: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::threads::Entity",
        from = "Column::ThreadId",
        to = "super::threads::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Threads,
    #[sea_orm(
        belongs_to = "super::ugc::Entity",
        from = "Column::UgcId",
        to = "super::ugc::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Ugc,
    #[sea_orm(
        belongs_to = "super::ugc_deletions::Entity",
        from = "Column::UgcId",
        to = "super::ugc_deletions::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    UgcDeletions,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::user_names::Entity",
        from = "Column::UserId",
        to = "super::user_names::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    UserName,
}

impl Related<super::threads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Threads.def()
    }
}

impl Related<super::ugc::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ugc.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::user_names::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserName.def()
    }
}

impl Related<super::ugc_deletions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UgcDeletions.def()
    }
}

impl Related<super::ugc_revisions::Entity> for Entity {
    fn to() -> RelationDef {
        super::ugc::Relation::UgcRevisions.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::ugc::Relation::Posts.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct PostToUgcRevision;

impl Linked for PostToUgcRevision {
    type FromEntity = super::posts::Entity;

    type ToEntity = super::ugc_revisions::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::posts::Relation::Ugc.def(),
            super::ugc::Relation::UgcRevisions.def(),
        ]
    }
}

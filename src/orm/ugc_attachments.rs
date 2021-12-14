//! SeaORM Entity. Generated by sea-orm-codegen 0.4.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ugc_attachments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub attachment_id: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub ip_id: Option<i32>,
    pub created_at: DateTime,
    #[sea_orm(column_type = "Text")]
    pub filename: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::attachments::Entity",
        from = "Column::AttachmentId",
        to = "super::attachments::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Attachments,
    #[sea_orm(
        belongs_to = "super::ip::Entity",
        from = "Column::IpId",
        to = "super::ip::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Ip,
    #[sea_orm(
        belongs_to = "super::ugc::Entity",
        from = "Column::UgcId",
        to = "super::ugc::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Ugc,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::attachments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachments.def()
    }
}

impl Related<super::ip::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ip.def()
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

impl ActiveModelBehavior for ActiveModel {}

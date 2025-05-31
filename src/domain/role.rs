use serde::Serialize;

#[derive(Clone, Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Teacher,
    Student,
}

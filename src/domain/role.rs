#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    Teacher,
    Student,
}

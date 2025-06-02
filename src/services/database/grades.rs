use crate::domain::{
    grades::{DbGradeRecord, StudentGrade, Subject},
    subject_id::SubjectId,
    user_id::DbUserId,
};

use super::{sql_error, Database};

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_subjects(db: &Database) -> crate::Result<Vec<Subject>> {
    sqlx::query_as(
        "
        select id, title
        from subjects
        ",
    )
    .fetch_all(&db.pool)
    .await
    .map_err(sql_error)
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_db_grades(
    db: &Database,
    subject_id: Option<SubjectId>,
) -> crate::Result<Vec<DbGradeRecord>> {
    let base_query = "
        select
            users.id as student_id,
            users.name as student_name,
            grades.value as grade,
            subjects.id as subject_id,
            subjects.title as subject_title
        from users
        join grades on users.id = grades.user_id
        join subjects on grades.subject_id = subjects.id
        ";
    match subject_id {
        Some(subject_id) => {
            sqlx::query_as(&format!("{base_query} where subjects.id = $1"))
                .bind(&subject_id)
                .fetch_all(&db.pool)
                .await
        }
        None => {
            sqlx::query_as(base_query)
                .bind(&subject_id)
                .fetch_all(&db.pool)
                .await
        }
    }
    .map_err(sql_error)
}

#[tracing::instrument(skip(db), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_db_student_grades(
    db: &Database,
    student_id: DbUserId,
) -> crate::Result<Vec<StudentGrade>> {
    sqlx::query_as(
        "
        select
            grades.value as grade,
            subjects.id as subject_id,
            subjects.title as subject_title
        from grades
        join subjects on grades.subject_id = subjects.id
        where grades.user_id = $1
        ",
    )
    .bind(&student_id)
    .fetch_all(&db.pool)
    .await
    .map_err(sql_error)
}

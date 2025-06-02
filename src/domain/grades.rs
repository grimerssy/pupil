use serde::Serialize;

use super::{
    email::Email,
    grade::Grade,
    name::Name,
    subject_id::SubjectId,
    subject_title::SubjectTitle,
    user_id::{DbUserId, UserId},
};

pub trait GetSubjects {
    async fn get_subjects(&self) -> crate::Result<Vec<Subject>>;
}

pub trait GetStudentGrades {
    async fn get_student_grades(&self, student_id: UserId) -> crate::Result<Vec<StudentGrade>>;
}

pub trait GetGrades {
    async fn get_grades(&self, subject: Option<SubjectId>) -> crate::Result<Vec<GradeRecord>>;
}

pub trait GetDbStudentGrades {
    async fn get_db_student_grades(&self, student_id: DbUserId)
        -> crate::Result<Vec<StudentGrade>>;
}

pub trait GetDbGrades {
    async fn get_db_grades(&self, subject: Option<SubjectId>) -> crate::Result<Vec<DbGradeRecord>>;
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Subject {
    pub id: SubjectId,
    pub title: SubjectTitle,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct StudentGrade {
    pub grade: Grade,
    pub subject_id: SubjectId,
    pub subject_title: SubjectTitle,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeRecord {
    pub student_id: UserId,
    pub student_name: Name,
    pub grade: Grade,
    pub subject_id: SubjectId,
    pub subject_title: SubjectTitle,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbGradeRecord {
    pub student_id: DbUserId,
    pub student_name: Name,
    pub grade: Grade,
    pub subject_id: SubjectId,
    pub subject_title: SubjectTitle,
}

#[derive(Debug, Clone, Serialize)]
pub struct Student {
    pub id: UserId,
    pub email: Email,
    pub name: Name,
}

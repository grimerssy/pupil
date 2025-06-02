use crate::{
    domain::{
        auth::{DecodeUserId, EncodeUserId},
        grades::*,
        subject_id::SubjectId,
        user_id::{DbUserId, UserId},
    },
    error::ErrorKind,
    services::database::grades::{get_db_grades, get_db_student_grades, get_subjects},
};

use super::AppContext;

impl GetSubjects for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_subjects(&self) -> crate::Result<Vec<Subject>> {
        get_subjects(&self.database).await
    }
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_grades(
    ctx: &AppContext,
    subject: Option<String>,
) -> crate::Result<Vec<GradeRecord>> {
    let subject = subject.and_then(|subject| SubjectId::new(subject).ok());
    ctx.get_grades(subject).await
}

impl GetGrades for AppContext {
    async fn get_grades(&self, subject_id: Option<SubjectId>) -> crate::Result<Vec<GradeRecord>> {
        get_grades_with(self, self, subject_id).await
    }
}

impl GetStudentGrades for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_student_grades(&self, student_id: UserId) -> crate::Result<Vec<StudentGrade>> {
        get_student_grades_with(self, self, student_id).await
    }
}

async fn get_grades_with(
    storage: &impl GetDbGrades,
    encoder: &impl EncodeUserId,
    subject_id: Option<SubjectId>,
) -> crate::Result<Vec<GradeRecord>> {
    let db_grades = storage.get_db_grades(subject_id).await?;
    let mut grades = Vec::with_capacity(db_grades.len());
    for grade in db_grades {
        let DbGradeRecord {
            student_id,
            student_name,
            grade,
            subject_id,
            subject_title,
        } = grade;
        let student_id = encoder.encode_user_id(student_id)?;
        let grade = GradeRecord {
            student_id,
            student_name,
            grade,
            subject_id,
            subject_title,
        };
        grades.push(grade);
    }
    Ok(grades)
}

async fn get_student_grades_with(
    decoder: &impl DecodeUserId,
    storage: &impl GetDbStudentGrades,
    student_id: UserId,
) -> crate::Result<Vec<StudentGrade>> {
    let student_id = match decoder.decode_user_id(student_id) {
        Ok(id) => id,
        Err(error) => match error.kind {
            ErrorKind::Expected(_) => return Ok(Vec::new()),
            ErrorKind::Internal(error) => return Err(crate::Error::internal(error)),
        },
    };
    storage.get_db_student_grades(student_id).await
}

impl GetDbGrades for AppContext {
    async fn get_db_grades(
        &self,
        subject_id: Option<SubjectId>,
    ) -> crate::Result<Vec<DbGradeRecord>> {
        get_db_grades(&self.database, subject_id).await
    }
}

impl GetDbStudentGrades for AppContext {
    async fn get_db_student_grades(
        &self,
        student_id: DbUserId,
    ) -> crate::Result<Vec<StudentGrade>> {
        get_db_student_grades(&self.database, student_id).await
    }
}

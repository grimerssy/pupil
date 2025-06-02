use crate::{
    domain::{
        auth::{DecodeUserId, EncodeUserId},
        grades::{
            DbGradeRecord, GetDbGrades, GetDbStudentGrades, GetGrades, GetStudentGrades,
            GradeRecord, StudentGrade,
        },
        user_id::{DbUserId, UserId},
    },
    error::ErrorKind,
    services::database::grades::{get_db_grades, get_db_student_grades},
};

use super::AppContext;

impl GetGrades for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_grades(&self) -> crate::Result<Vec<GradeRecord>> {
        get_grades_with(self, self).await
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
) -> crate::Result<Vec<GradeRecord>> {
    let db_grades = storage.get_db_grades().await?;
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
    async fn get_db_grades(&self) -> crate::Result<Vec<DbGradeRecord>> {
        get_db_grades(&self.database).await
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

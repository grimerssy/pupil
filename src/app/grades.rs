use std::str::FromStr;

use squint::Id;

use crate::{
    domain::{
        auth::{DecodeUserId, EncodeUserId},
        grade::Grade,
        grades::*,
        subject_id::SubjectId,
        user_id::{DbUserId, UserId},
    },
    error::ErrorKind,
    services::database::grades::{
        get_db_grade, get_db_grades, get_db_student_grades, get_subjects, update_db_grade,
    },
};

use super::AppContext;

impl GetSubjects for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_subjects(&self) -> crate::Result<Vec<Subject>> {
        get_subjects(&self.database).await
    }
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_grade(
    ctx: &AppContext,
    subject_id: String,
    student_id: String,
) -> crate::Result<GradeRecord, GetGradeError> {
    let subject_id =
        SubjectId::new(subject_id).map_err(|_| crate::Error::expected(GetGradeError::NotFound))?;
    let student_id = Id::from_str(&student_id)
        .map(UserId::new)
        .map_err(|_| crate::Error::expected(GetGradeError::NotFound))?;
    ctx.get_grade(subject_id, student_id).await
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_grades(
    ctx: &AppContext,
    subject: Option<String>,
) -> crate::Result<Vec<GradeRecord>> {
    let subject = subject.and_then(|subject| SubjectId::new(subject).ok());
    ctx.get_grades(subject).await
}

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn update_grade(
    ctx: &AppContext,
    subject: String,
    student: String,
    grade: String,
) -> crate::Result<GradeRecord> {
    let subject = SubjectId::new(subject).unwrap();
    let student = Id::from_str(&student).map(UserId::new).unwrap();
    let grade = Grade::new(grade).unwrap();
    ctx.update_grade(subject, student, grade).await
}

impl GetGrade for AppContext {
    async fn get_grade(
        &self,
        subject_id: SubjectId,
        student_id: UserId,
    ) -> crate::Result<GradeRecord, GetGradeError> {
        get_grade_with(self, self, self, subject_id, student_id).await
    }
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

impl UpdateGrade for AppContext {
    async fn update_grade(
        &self,
        subject: SubjectId,
        student: UserId,
        grade: Grade,
    ) -> crate::Result<GradeRecord> {
        update_grade_with(self, self, self, subject, student, grade).await
    }
}

async fn get_grade_with(
    decoder: &impl DecodeUserId,
    storage: &impl GetDbGrade,
    encoder: &impl EncodeUserId,
    subject_id: SubjectId,
    student_id: UserId,
) -> crate::Result<GradeRecord, GetGradeError> {
    let student_id = decoder
        .decode_user_id(student_id)
        .map_err(|_| crate::Error::expected(GetGradeError::NotFound))?;
    let grade = storage.get_db_grade(subject_id, student_id).await?;
    let DbGradeRecord {
        student_id,
        student_name,
        grade,
        subject_id,
        subject_title,
    } = grade;
    let student_id = encoder
        .encode_user_id(student_id)
        .map_err(crate::Error::from_internal)?;
    let grade = GradeRecord {
        student_id,
        student_name,
        grade,
        subject_id,
        subject_title,
    };
    Ok(grade)
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

async fn update_grade_with(
    decoder: &impl DecodeUserId,
    setter: &impl UpdateDbGrade,
    getter: &impl GetGrades,
    subject: SubjectId,
    student: UserId,
    grade: Grade,
) -> crate::Result<GradeRecord> {
    let student_id = decoder.decode_user_id(student.clone()).unwrap();
    setter
        .update_db_grade(subject.clone(), student_id, grade)
        .await?;
    let grade = getter
        .get_grades(Some(subject))
        .await?
        .into_iter()
        .find(|grade| grade.student_id == student)
        .unwrap();
    Ok(grade)
}

impl GetDbGrade for AppContext {
    async fn get_db_grade(
        &self,
        subject_id: SubjectId,
        student_id: DbUserId,
    ) -> crate::Result<DbGradeRecord, GetGradeError> {
        get_db_grade(&self.database, subject_id, student_id).await
    }
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

impl UpdateDbGrade for AppContext {
    async fn update_db_grade(
        &self,
        subject: SubjectId,
        student: DbUserId,
        grade: crate::domain::grade::Grade,
    ) -> crate::Result<()> {
        update_db_grade(&self.database, subject, student, grade).await
    }
}

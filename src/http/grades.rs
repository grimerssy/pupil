use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{grades::get_grades, AppContext},
    domain::{
        auth::User,
        grades::{GetStudentGrades, GetSubjects, GradeRecord, Subject},
        role::Role,
    },
    error::Error,
};

use super::middleware::template::{Template, TemplateName};

const TEACHER_GRADES: &str = "teacher-grades.html";

const STUDENT_GRADES: &str = "student-grades.html";

pub fn grade_routes() -> Router<AppContext> {
    Router::new().route("/", get(grades_page))
}

#[derive(Clone, Debug, Deserialize)]
pub struct GradesQuery {
    pub subject: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TeacherGrades {
    pub grades: Vec<GradeRecord>,
    pub subjects: Vec<Subject>,
}

pub async fn grades_page(
    user: User,
    State(ctx): State<AppContext>,
    Query(query): Query<GradesQuery>,
) -> Result<Response, Template<Error>> {
    match user.role {
        Role::Teacher => {
            let grades = get_grades(&ctx, query.subject).await;
            let subjects = ctx.get_subjects().await;
            grades
                .and_then(|grades| subjects.map(|subjects| TeacherGrades { grades, subjects }))
                .map(|grades| Template::new(TEACHER_GRADES, grades).into_response())
        }
        Role::Student => ctx
            .get_student_grades(user.id)
            .await
            .map(|grades| Template::new(STUDENT_GRADES, grades).into_response()),
    }
    .map_err(|error| Template::new(TemplateName::error(), error))
}

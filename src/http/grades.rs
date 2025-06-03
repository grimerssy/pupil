use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        grades::{get_grade, get_grades},
        AppContext,
    },
    domain::{
        auth::User,
        grades::{GetGradeError, GetStudentGrades, GetSubjects, GradeRecord, Subject},
        role::Role,
    },
    error::Error,
};

use super::{
    error::HttpError,
    middleware::template::{Template, TemplateName},
};

const GRADE: &str = "components/grade.html";

const GRADE_EDIT: &str = "components/grade-edit.html";

const TEACHER_GRADES: &str = "teacher-grades.html";

const STUDENT_GRADES: &str = "student-grades.html";

pub fn grades_routes() -> Router<AppContext> {
    let grade_routes = Router::new()
        .route("/", get(grade))
        .route("/edit", get(grade_edit))
        .route("/", put(grade));
    Router::new()
        .route("/", get(grades_page))
        .nest("/{subject_id}/{student_id}", grade_routes)
}

#[derive(Clone, Debug, Deserialize)]
struct GradesQuery {
    subject: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct TeacherGrades {
    grades: Vec<GradeRecord>,
    subjects: Vec<Subject>,
}

#[derive(Clone, Debug, Deserialize)]
struct GradePath {
    subject_id: String,
    student_id: String,
}

async fn grade(
    State(ctx): State<AppContext>,
    Path(path): Path<GradePath>,
) -> Result<Template<GradeRecord>, Template<Error<GetGradeError>>> {
    get_grade(&ctx, path.subject_id, path.student_id)
        .await
        .map(|grade| Template::new(GRADE, grade))
        .map_err(|error| Template::new(TemplateName::error(), error))
}

async fn grade_edit(
    State(ctx): State<AppContext>,
    Path(path): Path<GradePath>,
) -> Result<Template<GradeRecord>, Template<Error<GetGradeError>>> {
    get_grade(&ctx, path.subject_id, path.student_id)
        .await
        .map(|grade| Template::new(GRADE_EDIT, grade))
        .map_err(|error| Template::new(TemplateName::error(), error))
}

async fn grades_page(
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

impl HttpError for GetGradeError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetGradeError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

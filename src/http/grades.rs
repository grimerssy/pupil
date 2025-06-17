use std::{collections::HashSet, convert::Infallible};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
    Form, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        grades::{get_grade, get_grades, update_grade, UpdateGradeRequest},
        validation::{try_convert, ValidationErrors},
        AppContext, AppError,
    },
    domain::{
        auth::User,
        grades::{GetGradeError, GetGrades, GetStudentGrades, GetSubjects, GradeRecord, Subject},
        name::Name,
        role::Role,
        user_id::UserId,
    },
    error::Error,
};

use super::{
    error::HttpError,
    middleware::{
        template::{Template, TemplateName},
        view::View,
    },
};

const GRADE: &str = "components/grade.html";

const GRADE_EDIT: &str = "components/grade-edit.html";

const GRADE_ADD: &str = "grade-add.html";

const TEACHER_GRADES: &str = "teacher-grades.html";

const STUDENT_GRADES: &str = "student-grades.html";

pub fn grades_routes() -> Router<AppContext> {
    let grade_routes = Router::new()
        .route("/", get(grade))
        .route("/edit", get(grade_edit))
        .route("/", put(edit_grade));
    Router::new()
        .route("/", get(grades_page))
        .route("/add", get(grade_add))
        .route("/add", post(add_grade))
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
struct Student {
    id: UserId,
    name: Name,
}

#[derive(Clone, Debug, Serialize)]
struct GradeAddOptions {
    students: HashSet<Student>,
    subjects: HashSet<Subject>,
}

async fn grade_add(
    State(ctx): State<AppContext>,
) -> Result<Template<GradeAddOptions>, Template<Error>> {
    let grades = ctx
        .get_grades(None)
        .await
        .map_err(|error| Template::new(TemplateName::error(), error))?;
    let students = grades
        .iter()
        .cloned()
        .map(|record| Student {
            id: record.student_id,
            name: record.student_name,
        })
        .collect();
    let subjects = grades
        .iter()
        .cloned()
        .map(|record| Subject {
            id: record.subject_id,
            title: record.subject_title,
        })
        .collect();
    let options = GradeAddOptions { students, subjects };
    Ok(Template::new(GRADE_ADD, options))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct NewGrade {
    subject_id: String,
    student_id: String,
    grade: String,
}

async fn add_grade(
    State(ctx): State<AppContext>,
    Form(form): Form<NewGrade>,
) -> Result<Html<&'static str>, View<Error<AppError<Infallible>>>> {
    let req = GradeThing {
        subject: form.subject_id,
        student: form.student_id,
        grade: form.grade,
    };
    update_grade(&ctx, req)
        .await
        .map(|_| Html("<script>window.location = \"/\"</script>"))
        .map_err(|error| View::new(TemplateName::error(), error))
}

#[derive(Clone, Debug, Deserialize)]
struct GradeForm {
    grade: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GradeThing {
    pub subject: String,
    pub student: String,
    pub grade: String,
}

async fn edit_grade(
    State(ctx): State<AppContext>,
    Path(path): Path<GradePath>,
    Form(form): Form<GradeForm>,
) -> Result<View<GradeRecord>, View<Error<AppError<Infallible>, GradeThing>>> {
    let req = GradeThing {
        subject: path.subject_id,
        student: path.student_id,
        grade: form.grade,
    };
    update_grade(&ctx, req.clone())
        .await
        .map(|grade| View::new(GRADE, grade))
        .map_err(|error| View::new(GRADE_EDIT, error.with_input(req)))
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
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl TryFrom<GradeThing> for UpdateGradeRequest {
    type Error = ValidationErrors;

    fn try_from(value: GradeThing) -> Result<Self, Self::Error> {
        try_convert!(GradeThing value => UpdateGradeRequest {
            student,
            subject,
            grade
        })
    }
}

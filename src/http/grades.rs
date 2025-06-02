use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::{
    app::AppContext,
    domain::{
        auth::User,
        grades::{GetGrades, GetStudentGrades},
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

pub async fn grades_page(
    user: User,
    State(ctx): State<AppContext>,
) -> Result<Response, Template<Error>> {
    match user.role {
        Role::Teacher => ctx
            .get_grades()
            .await
            .map(|grades| Template::new(TEACHER_GRADES, grades).into_response()),

        Role::Student => ctx
            .get_student_grades(user.id)
            .await
            .map(|grades| Template::new(STUDENT_GRADES, grades).into_response()),
    }
    .map_err(|error| Template::new(TemplateName::error(), error))
}

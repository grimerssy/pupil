use std::collections::HashMap;

use rust_decimal::{Decimal, MathematicalOps};

use crate::{
    domain::{
        grades::GetDbGrades, key::Key, name::Name, percentile::Percentile, performance::*,
        signature::Signature, user_id::DbUserId,
    },
    services::{database::performance::lookup_key, signer::sign_evaluation},
};

use super::AppContext;

#[tracing::instrument(skip(ctx), ret(level = "debug") err(Debug, level = "debug"))]
pub async fn get_signature(
    ctx: &AppContext,
    key: String,
) -> crate::Result<SignedEvaluation, KeyLookupError> {
    let key = Key::try_from(key).map_err(|_| crate::Error::expected(KeyLookupError::UnknownKey))?;
    ctx.get_signature(key).await
}

async fn get_signature_with(
    evaluator: &impl GetPerformanceEvaluation,
    signer: &impl SignEvaluation,
    key: Key,
) -> crate::Result<SignedEvaluation, KeyLookupError> {
    let claim = evaluator.get_performance_evaluation(key).await?;
    let signature = signer
        .sign_evaluation(&claim)
        .map_err(crate::Error::from_internal)?;
    Ok(SignedEvaluation { claim, signature })
}

async fn get_performance_evaluation_with(
    key_storage: &impl LookupKey,
    grade_storage: &impl GetDbGrades,
    key: Key,
) -> crate::Result<PerformanceEvaluation, KeyLookupError> {
    let (student_id, student_name) = key_storage.lookup_key(key).await?;
    let grades = grade_storage
        .get_db_grades(None)
        .await
        .map_err(crate::Error::from_internal)?;

    let subject_grades = grades
        .iter()
        .fold(HashMap::<_, Vec<_>>::new(), |mut map, record| {
            map.entry(record.subject_id.clone())
                .or_default()
                .push(Decimal::from(record.grade));
            map
        });
    let means = subject_grades
        .iter()
        .map(|(subject, grades)| {
            let sum = grades.iter().copied().sum::<Decimal>();
            let n = Decimal::new(grades.len() as i64, 0);
            let mean = sum / n;
            (subject, mean)
        })
        .collect::<HashMap<_, _>>();
    let std_deviations = subject_grades
        .iter()
        .map(|(subject, grades)| {
            let deviation_sum = grades
                .iter()
                .map(|grade| (grade - means.get(subject).unwrap()).powu(2))
                .sum::<Decimal>();
            let variance = deviation_sum / Decimal::new((grades.len() - 1) as i64, 0);
            let std_deviation = variance.sqrt().unwrap();
            (subject, std_deviation)
        })
        .collect::<HashMap<_, _>>();
    let z_scores = grades
        .iter()
        .map(|record| {
            let student = record.student_id;
            let subject = record.subject_id.clone();
            let grade = Decimal::from(record.grade);
            let mean = means.get(&subject).unwrap();
            let std_deviation = std_deviations.get(&subject).unwrap();
            let z_score = (grade - mean)
                .checked_div(*std_deviation)
                .unwrap_or(Decimal::ZERO);
            (student, z_score)
        })
        .collect::<Vec<_>>();
    let avg_z_scores = z_scores
        .into_iter()
        .fold(HashMap::<_, Vec<_>>::new(), |mut map, (student, score)| {
            map.entry(student).or_default().push(score);
            map
        })
        .into_iter()
        .map(|(student, z_scores)| {
            let n = Decimal::new(z_scores.len() as i64, 0);
            let sum = z_scores.into_iter().sum::<Decimal>();
            let avg = sum / n;
            (student, avg)
        })
        .collect::<HashMap<_, _>>();
    let target_z_score = avg_z_scores
        .get(&student_id)
        .copied()
        .unwrap_or(Decimal::ZERO);
    let below_target = avg_z_scores
        .iter()
        .filter(|&(_, z_score)| *z_score <= target_z_score)
        .count();
    let total = avg_z_scores.len();
    let percentile = Decimal::new(below_target as i64, 0) / Decimal::new(total as i64, 0);
    let percentile = Percentile::new(percentile)?;
    let evaluation = PerformanceEvaluation {
        student: student_name,
        percentile,
    };
    Ok(evaluation)
}

impl GetSignature for AppContext {
    async fn get_signature(&self, key: Key) -> crate::Result<SignedEvaluation, KeyLookupError> {
        get_signature_with(self, self, key).await
    }
}

impl GetPerformanceEvaluation for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    async fn get_performance_evaluation(
        &self,
        key: Key,
    ) -> crate::Result<PerformanceEvaluation, KeyLookupError> {
        get_performance_evaluation_with(self, self, key).await
    }
}

impl SignEvaluation for AppContext {
    #[tracing::instrument(skip(self), ret(level = "debug") err(Debug, level = "debug"))]
    fn sign_evaluation(&self, claim: &PerformanceEvaluation) -> crate::Result<Signature> {
        sign_evaluation(&self.signer, claim)
    }
}

impl LookupKey for AppContext {
    async fn lookup_key(&self, key: Key) -> crate::Result<(DbUserId, Name), KeyLookupError> {
        lookup_key(&self.database, key).await
    }
}

//! Derived-state maintenance shared by the API and the worker.

use sqlx::{PgPool, Row};

use crate::level;

/// Recomputes one user's state for one problem from their submissions and
/// keeps the problem's solver count in step. Returns true when the solved
/// state changed (so the rating must be recomputed).
pub async fn refresh_user_problem(
    db: &PgPool,
    user_id: i64,
    problem_id: i32,
) -> anyhow::Result<bool> {
    let mut tx = db.begin().await?;
    let row = sqlx::query(
        "SELECT count(*) AS n,
                bool_or(status = 'AC') AS solved,
                min(created_at) FILTER (WHERE status = 'AC') AS first_ac
         FROM submissions WHERE user_id = $1 AND problem_id = $2",
    )
    .bind(user_id)
    .bind(problem_id)
    .fetch_one(&mut *tx)
    .await?;
    let n: i64 = row.get("n");
    let solved: bool = row.get::<Option<bool>, _>("solved").unwrap_or(false);
    let first_ac: Option<chrono::DateTime<chrono::Utc>> = row.get("first_ac");

    let prev: Option<String> = sqlx::query_scalar(
        "SELECT state FROM user_problem WHERE user_id = $1 AND problem_id = $2 FOR UPDATE",
    )
    .bind(user_id)
    .bind(problem_id)
    .fetch_optional(&mut *tx)
    .await?;
    let was_solved = prev.as_deref() == Some("SOLVED");

    if n == 0 {
        sqlx::query("DELETE FROM user_problem WHERE user_id = $1 AND problem_id = $2")
            .bind(user_id)
            .bind(problem_id)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query(
            "INSERT INTO user_problem (user_id, problem_id, state, first_ac_at) VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, problem_id) DO UPDATE SET state = EXCLUDED.state, first_ac_at = EXCLUDED.first_ac_at",
        )
        .bind(user_id)
        .bind(problem_id)
        .bind(if solved { "SOLVED" } else { "TRIED" })
        .bind(first_ac)
        .execute(&mut *tx)
        .await?;
    }

    let changed = solved != was_solved;
    if changed {
        sqlx::query(
            "UPDATE problems SET solved_count =
               (SELECT count(*) FROM user_problem WHERE problem_id = $1 AND state = 'SOLVED')
             WHERE id = $1",
        )
        .bind(problem_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(changed)
}

/// Recomputes rating and tier from the levels of solved public problems.
pub async fn recompute_rating(db: &PgPool, user_id: i64) -> anyhow::Result<(i64, u8)> {
    let levels: Vec<i16> = sqlx::query_scalar(
        "SELECT p.level FROM user_problem up JOIN problems p ON p.id = up.problem_id
         WHERE up.user_id = $1 AND up.state = 'SOLVED' AND p.status = 'PUBLIC'",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let rating = level::rating(levels.into_iter().map(|l| l.clamp(0, 30) as u8));
    let tier = level::tier(rating);
    sqlx::query("UPDATE users SET rating = $2, tier = $3 WHERE id = $1")
        .bind(user_id)
        .bind(rating as i32)
        .bind(i16::from(tier))
        .execute(db)
        .await?;
    Ok((rating, tier))
}

/// Recomputes everyone who solved a problem, e.g. after its level changed.
pub async fn recompute_solvers(db: &PgPool, problem_id: i32) -> anyhow::Result<usize> {
    let users: Vec<i64> = sqlx::query_scalar(
        "SELECT user_id FROM user_problem WHERE problem_id = $1 AND state = 'SOLVED'",
    )
    .bind(problem_id)
    .fetch_all(db)
    .await?;
    for &u in &users {
        recompute_rating(db, u).await?;
    }
    Ok(users.len())
}

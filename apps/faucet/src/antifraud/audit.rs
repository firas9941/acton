use anyhow::Context;
use sqlx::{SqlitePool, query};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Clone)]
pub struct AntifraudAuditStore {
    pool: SqlitePool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AuditSubject {
    kind: &'static str,
    value: String,
}

#[derive(Debug)]
pub struct PayoutAudit {
    request_id: String,
    amount_nano: i64,
    client_kind: String,
    subjects: Vec<AuditSubject>,
}

impl AuditSubject {
    pub fn wallet(value: impl Into<String>) -> anyhow::Result<Self> {
        Self::new("wallet", value)
    }

    pub fn ip(value: IpAddr) -> Self {
        let value = match value {
            IpAddr::V6(value) => value
                .to_ipv4_mapped()
                .map(IpAddr::V4)
                .unwrap_or(IpAddr::V6(value)),
            value => value,
        };

        Self {
            kind: "ip",
            value: value.to_string(),
        }
    }

    pub fn device_uid(value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into().replace('-', "").to_ascii_lowercase();
        Self::new("device_uid", value)
    }

    pub fn github_user_id(value: u64) -> Self {
        Self {
            kind: "github_user_id",
            value: value.to_string(),
        }
    }

    fn new(kind: &'static str, value: impl Into<String>) -> anyhow::Result<Self> {
        let value = value.into();
        anyhow::ensure!(
            !value.trim().is_empty(),
            "audit subject value must not be empty"
        );
        Ok(Self { kind, value })
    }
}

impl PayoutAudit {
    pub fn new(
        request_id: impl Into<String>,
        amount_nano: u64,
        client_kind: impl Into<String>,
        subjects: Vec<AuditSubject>,
    ) -> anyhow::Result<Self> {
        let request_id = request_id.into();
        Uuid::parse_str(&request_id).context("audit request ID must be a UUID")?;

        let amount_nano = i64::try_from(amount_nano)
            .context("audit payout amount exceeds SQLite INTEGER range")?;
        anyhow::ensure!(amount_nano > 0, "audit payout amount must be positive");

        let client_kind = client_kind.into();
        anyhow::ensure!(
            !client_kind.trim().is_empty(),
            "audit client kind must not be empty"
        );
        anyhow::ensure!(!subjects.is_empty(), "audit payout must have subjects");

        Ok(Self {
            request_id,
            amount_nano,
            client_kind,
            subjects,
        })
    }
}

impl AntifraudAuditStore {
    pub async fn setup(pool: SqlitePool) -> anyhow::Result<Self> {
        query(
            r#"
            CREATE TABLE IF NOT EXISTS antifraud_audit (
                id INTEGER PRIMARY KEY,
                request_id TEXT NOT NULL CHECK (length(trim(request_id)) > 0),
                paid_at INTEGER NOT NULL,
                amount_nano INTEGER NOT NULL CHECK (amount_nano > 0),
                subject_kind TEXT NOT NULL CHECK (length(trim(subject_kind)) > 0),
                subject_value TEXT NOT NULL CHECK (length(trim(subject_value)) > 0),
                client_kind TEXT NOT NULL CHECK (length(trim(client_kind)) > 0),
                UNIQUE (request_id, subject_kind, subject_value)
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create antifraud audit table")?;

        for statement in [
            r#"
            CREATE INDEX IF NOT EXISTS antifraud_audit_request_id_idx
            ON antifraud_audit (request_id)
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS antifraud_audit_subject_paid_at_idx
            ON antifraud_audit (subject_kind, subject_value, paid_at DESC)
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS antifraud_audit_paid_at_idx
            ON antifraud_audit (paid_at DESC)
            "#,
        ] {
            query(statement)
                .execute(&pool)
                .await
                .context("Failed to create antifraud audit index")?;
        }

        Ok(Self { pool })
    }

    pub async fn record_payout(&self, payout: &PayoutAudit, paid_at: i64) -> anyhow::Result<()> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Failed to start antifraud audit transaction")?;

        for subject in &payout.subjects {
            query(
                r#"
                INSERT INTO antifraud_audit (
                    request_id,
                    paid_at,
                    amount_nano,
                    subject_kind,
                    subject_value,
                    client_kind
                )
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT (request_id, subject_kind, subject_value) DO NOTHING
                "#,
            )
            .bind(&payout.request_id)
            .bind(paid_at)
            .bind(payout.amount_nano)
            .bind(subject.kind)
            .bind(&subject.value)
            .bind(&payout.client_kind)
            .execute(&mut *transaction)
            .await
            .context("Failed to insert antifraud audit subject")?;
        }

        transaction
            .commit()
            .await
            .context("Failed to commit antifraud audit transaction")
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Antifraud database health check failed")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Row, query, sqlite::SqlitePoolOptions};

    use super::{AntifraudAuditStore, AuditSubject, PayoutAudit};

    async fn store() -> AntifraudAuditStore {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        AntifraudAuditStore::setup(pool).await.unwrap()
    }

    fn payout() -> PayoutAudit {
        PayoutAudit::new(
            "00000000-0000-4000-8000-000000000001",
            2_000_000_000,
            "acton",
            vec![
                AuditSubject::wallet("0:abc").unwrap(),
                AuditSubject::ip("203.0.113.7".parse().unwrap()),
                AuditSubject::device_uid("00112233-4455-6677-8899-AABBCCDDEEFF").unwrap(),
                AuditSubject::github_user_id(42),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn records_all_payout_subjects_in_one_audit() {
        let store = store().await;
        store.record_payout(&payout(), 1_800_000_000).await.unwrap();

        let rows = query(
            r#"
            SELECT request_id, paid_at, amount_nano, subject_kind, subject_value, client_kind
            FROM antifraud_audit
            ORDER BY id
            "#,
        )
        .fetch_all(&store.pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|row| {
            row.get::<String, _>("request_id") == "00000000-0000-4000-8000-000000000001"
                && row.get::<i64, _>("paid_at") == 1_800_000_000
                && row.get::<i64, _>("amount_nano") == 2_000_000_000
                && row.get::<String, _>("client_kind") == "acton"
        }));
        assert_eq!(rows[0].get::<String, _>("subject_kind"), "wallet");
        assert_eq!(rows[0].get::<String, _>("subject_value"), "0:abc");
        assert_eq!(rows[2].get::<String, _>("subject_kind"), "device_uid");
        assert_eq!(
            rows[2].get::<String, _>("subject_value"),
            "00112233445566778899aabbccddeeff"
        );
    }

    #[tokio::test]
    async fn repeated_payout_recording_is_idempotent() {
        let store = store().await;
        let payout = payout();

        store.record_payout(&payout, 1_800_000_000).await.unwrap();
        store.record_payout(&payout, 1_800_000_001).await.unwrap();

        let count = query("SELECT COUNT(*) AS count FROM antifraud_audit")
            .fetch_one(&store.pool)
            .await
            .unwrap()
            .get::<i64, _>("count");
        assert_eq!(count, 4);
    }

    #[test]
    fn normalizes_ip_and_device_subjects() {
        assert_eq!(
            AuditSubject::ip("::ffff:192.0.2.44".parse().unwrap()),
            AuditSubject {
                kind: "ip",
                value: "192.0.2.44".to_string(),
            }
        );
        assert_eq!(
            AuditSubject::ip("2001:db8:1234:5678:abcd::1".parse().unwrap()),
            AuditSubject {
                kind: "ip",
                value: "2001:db8:1234:5678:abcd::1".to_string(),
            }
        );
        assert_eq!(
            AuditSubject::device_uid("00112233-4455-6677-8899-AABBCCDDEEFF").unwrap(),
            AuditSubject {
                kind: "device_uid",
                value: "00112233445566778899aabbccddeeff".to_string(),
            }
        );
    }
}

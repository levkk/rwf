use super::bootstrap::RwfDatabaseSchema;
pub fn migrations() -> Vec<RwfDatabaseSchema> {
    [
        RwfDatabaseSchema {
            id: 1i64,
            name: "migrations".to_string(),
            up: include_str!("bootstrap/1_migrations.up.sql").to_string(),
            down: include_str!("bootstrap/1_migrations.down.sql").to_string(),
        },
        RwfDatabaseSchema {
            id: 2i64,
            name: "jobs".to_string(),
            down: include_str!("bootstrap/2_jobs.down.sql").to_string(),
            up: include_str!("bootstrap/2_jobs.up.sql").to_string(),
        },
        RwfDatabaseSchema {
            id: 3i64,
            name: "requests_tracking".to_string(),
            down: include_str!("bootstrap/3_requests_tracking.down.sql").to_string(),
            up: include_str!("bootstrap/3_requests_tracking.up.sql").to_string(),
        },
        RwfDatabaseSchema {
            id: 4i64,
            name: "static_file_metadata".to_string(),
            down: include_str!("bootstrap/4_static_file_metadata.down.sql").to_string(),
            up: include_str!("bootstrap/4_static_files_metadata.up.sql").to_string(),
        },
    ]
    .to_vec()
}

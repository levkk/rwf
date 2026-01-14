use super::bootstrap::{parse_database_schema, RwfDatabaseSchema};
pub fn migrations() -> Vec<RwfDatabaseSchema> {
    [
        parse_database_schema(include_str!("bootstrap/1_migrations.yaml")),
        parse_database_schema(include_str!("bootstrap/2_jobs.yaml")),
        parse_database_schema(include_str!("bootstrap/3_requests_tracking.yaml")),
        parse_database_schema(include_str!("bootstrap/4_static_file_metadata.yaml")),
    ]
        .to_vec()
}

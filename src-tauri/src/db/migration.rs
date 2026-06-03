use refinery::embed_migrations;

embed_migrations!("./migrations");

pub fn run_migrations(conn: &mut rusqlite::Connection) -> Result<(), String> {
    migrations::runner().run(conn).map_err(|e| e.to_string())?;
    Ok(())
}

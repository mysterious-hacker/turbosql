use super::{MigrationsToml, Table};
use proc_macro_error::abort_call_site;
use quote::quote;
use rusqlite::params;
use serde::Serialize;
use std::fs;

#[cfg(not(feature = "test"))]
const MIGRATIONS_FILENAME: &str = "migrations.toml";
#[cfg(feature = "test")]
const MIGRATIONS_FILENAME: &str = "test.migrations.toml";

/// CREATE TABLE
pub(super) fn create(table: &Table) -> proc_macro2::TokenStream {
 // create the migrations

 let sql = makesql_create(&table);

 rusqlite::Connection::open_in_memory().unwrap().execute(&sql, params![]).unwrap_or_else(|e| {
  abort_call_site!("Error validating auto-generated CREATE TABLE statement: {} {:#?}", sql, e)
 });

 let target_migrations = make_migrations(&table);

 // read in the existing migrations from toml

 let lockfile = std::fs::File::create(std::env::temp_dir().join("migrations.toml.lock")).unwrap();
 fs2::FileExt::lock_exclusive(&lockfile).unwrap();

 let migrations_toml_path = std::env::current_dir().unwrap().join(MIGRATIONS_FILENAME);
 let migrations_toml_path_lossy = migrations_toml_path.to_string_lossy();

 let old_toml_str = if migrations_toml_path.exists() {
  fs::read_to_string(&migrations_toml_path)
   .unwrap_or_else(|e| abort_call_site!("Unable to read {}: {:?}", migrations_toml_path_lossy, e))
 } else {
  String::new()
 };

 let source_migrations_toml: MigrationsToml = toml::from_str(&old_toml_str).unwrap_or_else(|e| {
  abort_call_site!("Unable to decode toml in {}: {:?}", migrations_toml_path_lossy, e)
 });

 // add any migrations that aren't already present

 let mut output_migrations = source_migrations_toml.migrations_append_only.unwrap_or_default();

 target_migrations.iter().for_each(|target_m| {
  if output_migrations
   .iter()
   .find(|source_m| (source_m == &target_m) || (source_m == &&format!("--{}", target_m)))
   .is_none()
  {
   output_migrations.push(target_m.clone());
  }
 });

 // save to toml

 let mut new_toml_str = String::new();
 let mut serializer = toml::Serializer::pretty(&mut new_toml_str);
 serializer.pretty_array_indent(2);

 MigrationsToml {
  output_generated_schema_for_your_information_do_not_edit: Some(format!(
   "  {}\n",
   super::migrations_to_schema(&output_migrations)
    .unwrap()
    .replace("\n", "\n  ")
    .replace("(", "(\n    ")
    .replace(", ", ",\n    ")
    .replace(")", ",\n  )")
  )),
  migrations_append_only: Some(output_migrations),
 }
 .serialize(&mut serializer)
 .unwrap_or_else(|e| abort_call_site!("Unable to serialize migrations toml: {:?}", e));

 let new_toml_str = indoc::formatdoc! {"
  # This file is auto-generated by Turbosql.
  # It is used to create and apply automatic schema migrations.
  # It should be checked into source control.
  # Modifying it by hand may be dangerous; see the docs.

  {}", &new_toml_str};

 // Only write migrations.toml file if it has actually changed;
 // this keeps file mod date clean so cargo doesn't pathologically rebuild

 if old_toml_str != new_toml_str {
  fs::write(&migrations_toml_path, new_toml_str)
   .unwrap_or_else(|e| abort_call_site!("Unable to write {}: {:?}", migrations_toml_path_lossy, e));
 }

 quote!()
}

fn makesql_create(table: &Table) -> String {
 format!(
  "CREATE TABLE {} ({})",
  table.name,
  table.columns.iter().map(|c| format!("{} {}", c.name, c.sql_type)).collect::<Vec<_>>().join(",")
 )
}

fn make_migrations(table: &Table) -> Vec<String> {
 let mut vec = vec![format!("CREATE TABLE {} (rowid INTEGER PRIMARY KEY)", table.name)];

 let mut alters = table
  .columns
  .iter()
  .filter_map(|c| match (c.name.as_str(), c.sql_type) {
   ("rowid", "INTEGER PRIMARY KEY") => None,
   _ => Some(format!("ALTER TABLE {} ADD COLUMN {} {}", table.name, c.name, c.sql_type)),
  })
  .collect::<Vec<_>>();

 vec.append(&mut alters);

 vec
}

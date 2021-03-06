// cargo test --features test --manifest-path turbosql/Cargo.toml -- --nocapture

use i54_::i54;
use turbosql::{execute, select, Blob, Turbosql};

#[derive(Turbosql, Default, Debug, PartialEq, Clone)]
struct PersonIntegrationTest {
 rowid: Option<i64>,
 field_string: Option<String>,
 field_i64: Option<i64>,
 field_i54: Option<i54>,
 field_bool: Option<bool>,
 field_f64: Option<f64>,
 // field_f32: Option<f32>,
 field_u8: Option<u8>,
 field_i8: Option<i8>,
 field_u16: Option<u16>,
 field_i16: Option<i16>,
 field_u32: Option<u32>,
 field_i32: Option<i32>,
 field_blob: Option<Blob>,
}

#[derive(Turbosql, Default, Debug, Eq, PartialEq, Clone)]
#[allow(non_camel_case_types)]
struct PersonIntegrationTest_i54 {
 rowid: Option<i54>,
 name: Option<String>,
 age: Option<i64>,
 image_jpg: Option<Blob>,
}

#[test]
fn integration_test() {
 // check rowid returned from insert()
 let row = PersonIntegrationTest_i54 { ..Default::default() };
 assert!(row.insert().unwrap() == 1);
 assert!(row.insert().unwrap() == 2);

 let mut row = PersonIntegrationTest {
  rowid: None,
  field_string: Some("Bob".into()),
  field_u8: Some(42),
  field_i64: Some(85262398562),
  field_f64: Some(std::f64::consts::PI),
  // field_f32: Some(std::f64::consts::E),
  field_blob: None,
  ..Default::default()
 };

 row.insert().unwrap();
 row.rowid = Some(1);

 assert!(select!(i64 "1").unwrap() == 1);
 assert!(select!(i64 "SELECT 1").unwrap() == 1);
 assert!(
  execute!("")
   == Err(rusqlite::Error::SqliteFailure(
    rusqlite::ffi::Error { code: rusqlite::ErrorCode::APIMisuse, extended_code: 21 },
    Some("not an error".to_string()),
   ))
 );

 // assert!(select!(Vec<i64> "SELECT 1").unwrap() == Some(1));
 // assert!(select!(Option<i64> "SELECT 1").unwrap() == Some(1));

 assert!(select!(PersonIntegrationTest).unwrap() == row);
 assert!(
  select!(PersonIntegrationTest "rowid, field_string, field_i64, field_i54, field_bool, field_f64, field_u8, field_i8, field_u16, field_i16, field_u32, field_i32, field_blob FROM personintegrationtest")
   .unwrap()
   == row
 );

 // select! into struct without Turbosql derive

 #[derive(Debug, Eq, PartialEq, Clone)]
 struct NameAndAgeResult {
  name: Option<String>,
  age: Option<i64>,
 }

 assert!(
  select!(NameAndAgeResult r#""Martin Luther" AS name, field_u8 AS age FROM personintegrationtest"#)
   .unwrap() == NameAndAgeResult {
   name: Some("Martin Luther".into()),
   age: Some(row.field_u8.unwrap().into())
  }
 );

 assert!(select!(Vec<PersonIntegrationTest>).unwrap() == vec![row.clone()]);
 assert!(select!(Option<PersonIntegrationTest>).unwrap() == Some(row.clone()));

 assert!(select!(PersonIntegrationTest "WHERE field_u8 = ?", row.field_u8).unwrap() == row);
 assert!(
  select!(Vec<PersonIntegrationTest> "WHERE field_u8 = ?", row.field_u8).unwrap()
   == vec![row.clone()]
 );
 assert!(
  select!(Option<PersonIntegrationTest> "WHERE field_u8 = ?", row.field_u8).unwrap()
   == Some(row.clone())
 );

 // No rows returned

 assert!(select!(PersonIntegrationTest "WHERE field_u8 = 999").is_err());
 assert!(select!(Vec<PersonIntegrationTest> "WHERE field_u8 = 999").unwrap() == vec![]);
 assert!(select!(Option<PersonIntegrationTest> "WHERE field_u8 = 999").unwrap() == None);

 // assert!(
 //  select!(f32 "field_u8 FROM personintegrationtest").unwrap() == row.field_f64.unwrap() as f32
 // );
 assert!(select!(f64 "field_f64 FROM personintegrationtest").unwrap() == row.field_f64.unwrap());

 assert!(select!(i8 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as i8);
 assert!(select!(u8 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as u8);
 assert!(
  select!(i16 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as i16
 );
 assert!(
  select!(u16 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as u16
 );
 assert!(
  select!(i32 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as i32
 );
 assert!(
  select!(u32 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap() as u32
 );
 assert!(
  select!(i64 "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap().into()
 );

 assert!(
  select!(bool "field_string = ? FROM personintegrationtest", "Arthur Schopenhauer").unwrap()
   == false
 );
 let new_row = row.clone();
 assert!(
  select!(bool "field_string = ? FROM personintegrationtest", new_row.field_string.unwrap())
   .unwrap()
   == true
 );
 // this incorrectly consumes row:
 // assert!(select!(bool "field_string = ? FROM personintegrationtest", row.field_string.unwrap()).unwrap() == true);

 // select!(PersonIntegrationTest "WHERE field_string = ?", row.field_string.unwrap());

 assert!(
  select!(bool "field_string = ? FROM personintegrationtest", row.clone().field_string.unwrap())
   .unwrap()
   == true
 );

 assert!(
  select!(String "field_string FROM personintegrationtest").unwrap() == row.field_string.unwrap()
 );

 // assert!(select!(Option<i64> "field_u8 FROM personintegrationtest").unwrap() == Some(row.field_u8.unwrap()));
 assert!(select!(i64 "field_u8 FROM personintegrationtest WHERE FALSE").is_err());
 // assert!(select!(Option<i64> "field_u8 FROM personintegrationtest WHERE ?", false).unwrap() == None);

 // assert!(select!(Vec<i64> "field_u8 FROM personintegrationtest").unwrap() == row.field_u8.unwrap());
 // assert!(select!(Option<i64> "field_u8 FROM personintegrationtest").unwrap() == row.field_u8);
 // assert!(select!(String "field_string FROM personintegrationtest").unwrap() == row.field_string.unwrap());

 // DELETE

 assert!(execute!("DELETE FROM personintegrationtest").is_ok());
 assert!(select!(PersonIntegrationTest).is_err());
}

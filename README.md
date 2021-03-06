# Turbosql

[<img alt="github" src="https://img.shields.io/badge/github-trevyn/turbosql-663399?style=for-the-badge&labelColor=555555&logo=github" height="40">](https://github.com/trevyn/turbosql)
[<img alt="crates.io" src="https://img.shields.io/crates/v/turbosql.svg?style=for-the-badge&color=ffc833&logo=rust" height="40">](https://crates.io/crates/turbosql)
[<img alt="discord" src="https://img.shields.io/discord/761441128544600074?label=chat%20on%20Discord&style=for-the-badge&color=7289d9&logo=discord&logoColor=FFF" height="40">](https://discord.gg/RX3rTWUzUD)

Easy local data persistence layer, backed by SQLite.

- Schema auto-defined by your Rust `struct`s
- Automatic schema migrations
- Super-simple basic CRUD operations
- Use complex SQL if that's your jam
- Validates all SQL (including user-supplied) at compile time

## Status

In active development, updated February 2021! Contributions are very much welcome!

## Usage

```toml
[dependencies]
turbosql = "0.1"
```

```rust
use turbosql::{Turbosql, Blob, select, execute};

#[derive(Turbosql, Default)]
struct Person {
    rowid: Option<i64>, // rowid member required & enforced at compile time
    name: Option<String>,
    age: Option<i64>,
    image_jpg: Option<Blob>
}

// INSERT a row
let rowid = Person {
    rowid: None,
    name: Some("Joe".to_string()),
    age: Some(42),
    image_jpg: None
}.insert().unwrap();

// SELECT all rows
let people: Vec<Person> = select!(Vec<Person>).unwrap();

// SELECT multiple rows with a predicate
let people: Vec<Person> = select!(Vec<Person> "WHERE age > ?", 21).unwrap();

// SELECT a single row with a predicate
let person: Person = select!(Person "WHERE name = ?", "Joe").unwrap();

// UPDATE
execute!("UPDATE person SET age = ? WHERE name = ?", 18, "Joe").unwrap();

// DELETE
execute!("DELETE FROM person WHERE rowid = ?", 1).unwrap();
```

See [`integration_test.rs`](https://github.com/trevyn/turbosql/blob/main/turbosql/tests/integration_test.rs) for more usage examples!

## Under the Hood

Turbosql generates a SQLite schema and prepared queries for each struct:

```rust
use turbosql::{Turbosql, Blob};

#[derive(Turbosql, Default)]
struct Person {
    rowid: Option<i64>, // rowid member required & enforced
    name: Option<String>,
    age: Option<i64>,
    image_jpg: Option<Blob>
}
```

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↓&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;auto-generates and validates the schema

```sqlite3
CREATE TABLE person (
    rowid INTEGER PRIMARY KEY,
    name TEXT,
    age INTEGER,
    image_jpg BLOB,
)

INSERT INTO person (rowid, name, age, image_jpg) VALUES (?, ?, ?, ?)

SELECT rowid, name, age, image_jpg FROM person
```

Queries with SQL predicates are also assembled and validated at compile time. Note that SQL types vs Rust types for parameter bindings are not currently checked at compile time.

```rust-no-test
let people = select!(Vec<Person> "WHERE age > ?", 21);
```

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↓

```sqlite3
SELECT rowid, name, age, image_jpg FROM person WHERE age > ?
```

## Automatic Schema Migrations

At compile time, the `#[derive(Turbosql)]` macro runs and creates a `migrations.toml` file in your project root that describes the database schema.

Each time you change a `struct` declaration and the macro is re-run (e.g. by `cargo` or `rust-analyzer`), migration SQL statements are generated that update the database schema. These new statements are recorded in `migrations.toml`, and are automatically embedded in your binary.

```rust
#[derive(turbosql::Turbosql, Default)]
struct Person {
    rowid: Option<i64>,
    name: Option<String>
}
```

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;↓&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;auto-generates `migrations.toml`

```toml
migrations_append_only = [
  'CREATE TABLE person(rowid INTEGER PRIMARY KEY)',
  'ALTER TABLE person ADD COLUMN name TEXT',
]
output_generated_schema_for_your_information_do_not_edit = '''
  CREATE TABLE person (
    rowid INTEGER PRIMARY KEY,
    name TEXT
  )
'''
```

When your schema changes, any new version of your binary will automatically migrate any older database file to the current schema by applying the appropriate migrations in sequence.

This migration process is a one-way ratchet: Old versions of the binary run on a database file with a newer schema will detect a schema mismatch and will be blocked from operating on the futuristically-schema'd database file.

Unused or reverted migrations that are created during development can be manually removed from `migrations.toml` before being released, but any database files that have already applied these deleted migrations will error and must be rebuilt. Proceed with care. When in doubt, refrain from manually editing `migrations.toml`, and everything should work fine.

- Just declare and freely append fields to your `struct`s.
- Check out the `migrations.toml` file that is generated in your project root to see what's happening.
- If you run into any weird compiler errors, try just re-compiling first; depending on the order the proc macros run, sometimes it just needs a little push to get in sync after a schema change.
- Schema migrations are one-way, append-only. (SQLite doesn't even support `ALTER TABLE DROP {column}`, so we're not even going there for now.)
- On launch, versions of your binary built with a newer schema will automatically apply the appropriate migrations to an older database.
- If you're feeling adventurous, you can add your own schema migration entries to the bottom of the list. (For creating indexes, etc.)
- Questions? Ask on Discord (https://discord.gg/RX3rTWUzUD) or open a GitHub discussion! -> https://github.com/trevyn/turbosql/discussions/new

## Where's my data?

The SQLite database is created in the directory returned by [`directories_next`](https://crates.io/crates/directories-next)`::ProjectDirs::data_dir()` + your executable's filename stem, which resolves to something like:

<table><tr><td>Linux</td><td><br>

`$XDG_DATA_HOME`/`{exe_name}` or `$HOME`/.local/share/`{exe_name}` _/home/alice/.local/share/fooapp/fooapp.sqlite_

</td></tr><tr><td>macOS</td><td><br>

`$HOME`/Library/Application&nbsp;Support/`{exe_name}` _/Users/Alice/Library/Application&nbsp;Support/org.fooapp.fooapp/fooapp.sqlite_

</td></tr><tr><td>Windows</td><td><br>

`{FOLDERID_LocalAppData}`\\`{exe_name}`\\data _C:\Users\Alice\AppData\Local\fooapp\fooapp\data\fooapp.sqlite_

</td></tr></table>

## `-wal` and `-shm` files

SQLite is an extremely reliable database engine, but it helps to understand how it interfaces with the filesystem. The main `.sqlite` file contains the bulk of the database. During database writes, SQLite also creates `.sqlite-wal` and `.sqlite-shm` files. If the host process is terminated without flushing writes, you may end up with these three files when you expected to have a single file. This is always fine; on next launch, SQLite knows how to resolve any interrupted writes and make sense of the world. However, if the `-wal` and/or `-shm` files are present, they **must be considered essential to database integrity**. Deleting them may result in a corrupted database. See https://sqlite.org/tempfiles.html .

## ⚠️&nbsp;Example Query Forms

**⚠️&nbsp;NOTE: This table is somewhat speculative and not completely aligned with the code yet. Check [`integration_test.rs`](https://github.com/trevyn/turbosql/blob/main/turbosql/tests/integration_test.rs) for examples of what works today and is tested in CI.&nbsp;⚠️**

<table>

<tr><td><b>⚠️&nbsp;Primitive&nbsp;type</b></td><td><br>

```rust-no-test
let result = select!(String "SELECT name FROM person")?;
```

Returns one value cast to specified type, returns `TurboSql::Error::QueryReturnedNoRows` if no rows available.

```rust-no-test
let result = select!(String "name FROM person WHERE rowid = ?", rowid)?;
```

`SELECT` keyword is **always optional** when using `select!`; it's added automatically as needed.<br>Parameter binding is straightforward.

</td></tr>

<tr><td><b>⚠️&nbsp;Tuple</b></td><td><br>

```rust-no-test
let result = select!((String, i64) "name, age FROM person")?;
```

Use tuple types for multiple manually declared columns.

</td></tr>

<tr><td><b>⚠️&nbsp;Anonymous struct</b></td><td><br>

```rust-no-test
let result = select!("name_String, age_i64 FROM person")?;
println!("{}", result.name);
```

Types must be specified in column names to generate an anonymous struct.

</td></tr>

<tr><td>⚠️&nbsp;<b><code>Vec&lt;_&gt;</code></b></td><td><br>

```rust-no-test
let result = select!(Vec<String> "name FROM person")?;
```

Returns `Vec` of another type. If no rows, returns empty `Vec`. (Tuple types work inside, as well.)

```rust-no-test
let result = select!(Vec<_> "name_String, age_i64 FROM person")?;
```

Anonymous structs work, too.

</td></tr>

<tr><td>⚠️&nbsp;<b><code>Option&lt;_&gt;</code></b></td><td><br>

```rust-no-test
let result = select!(Option<String> "name FROM person")?;
```

Returns `Ok(None)` if no rows, `Error(Turbosql::Error)` on error.

</td></tr>

<tr><td><b>⚠️&nbsp;Your struct</b></td><td><br>

```rust-no-test
let result = select!(Person "WHERE name = ?", name)?;
```

Column list and table name are optional if type is a `#[derive(Turbosql)]` struct.

```rust-no-test
let result = select!(Vec<NameAndAdult> "name, age >= 18 AS adult FROM person")?;
```

You can use other struct types as well; column names must match the struct.<br>Implement `Default` to avoid specifying unused column names.<br>(And, of course, you can put it all in a `Vec` or `Option` as well.)

```rust-no-test
let result = select!(Vec<Person>)?;
```

Sometimes everything is optional; this example will retrieve all `Person` rows.

</td></tr>

<tr><td>⚠️&nbsp;<b>Transactions</b></td><td><br>

```rust-no-test
transaction! {
  if select!(Option<Person> "WHERE name = ?", name)?.is_none() {
    Person { ... }.insert!()?;
  }
}
```

- Haha just kidding, this doesn't exist yet.
- How might this work with threads and async?
- What if the transaction fails to commit?
- Nested transactions not supported?
- Calling other functions in a transaction? Async? This gets messy. Just say that any Turbosql calls outside of the literal text `transaction!{}` body will work fine, but _not_ be part of the transaction?

Inititally, this implementation might just open a new SQLite connection, and use it for all child calls.

</td></tr>

</table>
<br>

## "turbosql" or "Turbosql"?

Your choice, but you _definitely_ do not want to capitalize any of the _other_ letters in the name! ;)

### License: MIT OR Apache-2.0 OR CC0-1.0 (public domain)

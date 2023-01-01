// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    error::{DbErrorKind::MissingEntity, Result},
    prelude::AnkiError,
    storage::SqliteStorage,
};

impl SqliteStorage {
    /// Checks whether database table has specified column.
    pub(crate) fn table_has_column(&self, table_name: &str, column_name: &str) -> Result<bool> {
        let mut stmt = self
            .db
            .prepare(&format!("pragma table_info({})", table_name))?;
        let mut col_iter = stmt.query_map([], |row| {
            let found_col: String = row.get(1)?;
            Ok(found_col)
        })?;
        Ok(col_iter.any(|x| x.unwrap() == column_name))
    }

    /// Reads and returns extended version info from collection database.
    pub(crate) fn db_extended_version(&self) -> Result<u64> {
        let extended_version: Result<u64> = self
            .db
            .query_row("SELECT extended_version FROM col", [], |r| r.get(0))
            .map_err(Into::into);
        match extended_version {
            Ok(ev) => Ok(ev),
            Err(e) => {
                let msg: String = format!("extended version info not found in database: {:?}", e);
                Err(AnkiError::db_error(msg, MissingEntity))
            }
        }
    }

    /// Adds extended version information field to database.
    ///
    /// When run, the extended version info will initially be set to 202212011737, meaning "5:37
    /// P.M.  on 1 December, 2022".
    pub fn run_schema_extended_version_upgrade(&self) -> Result<()> {
        self.db
            .execute_batch(include_str!("schema_extended_version_upgrade.sql"))?;
        Ok(())
    }

    /// Upgrades database to extended version 202212011756.
    pub fn run_schema_202212011756_upgrade(&self) -> Result<()> {
        self.db
            .execute_batch(include_str!("schema_202212011756_upgrade.sql"))?;
        Ok(())
    }
}

/* Anki isn't really set up for unit tests. Tests below aren't isolated from the collection or
 * storage systems.
 */
#[cfg(test)]
mod test {
    #[test]
    fn table_has_column() {
        let col = crate::collection::open_test_collection();
        // Verify version columns present in db.
        assert!(col.storage.table_has_column("col", "ver").unwrap());
        assert!(col
            .storage
            .table_has_column("col", "extended_version")
            .unwrap());
        // Sanity check: should fail on nonexistent column.
        assert!(!col.storage.table_has_column("col", "fubar").unwrap());
    }

    #[test]
    fn db_extended_version() {
        let col = crate::collection::open_test_collection();
        // Verify latest extended database version.
        assert!(202212011756 <= col.storage.db_extended_version().unwrap());
        // Sanity check.
        assert!(999999999999 > col.storage.db_extended_version().unwrap());
    }
}

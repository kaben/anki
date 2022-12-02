// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::{
    backend::Backend,
    pb::revlog_service::Service as RevlogService,
    pb::{self as pb},
    prelude::*,
};

impl RevlogService for Backend {
    fn get_revlog_entry(&self, input: pb::RevlogId) -> Result<pb::RevlogEntry> {
        let rid = input.rid.into();
        self.with_col(|col| {
            col.storage
                .get_revlog_entry(rid)
                .and_then(|opt| opt.or_not_found(rid))
                .map(Into::into)
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn hello() {
        println!("hello from detailed_feedback::backend::revlog!");
    }
}

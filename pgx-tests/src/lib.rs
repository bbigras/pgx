// Copyright 2020 ZomboDB, LLC <zombodb@gmail.com>. All rights reserved. Use of this source code is
// governed by the MIT license that can be found in the LICENSE file.

mod framework;
#[cfg(any(test, feature = "pg_test"))]
mod tests;

pub use framework::*;

#[cfg(any(test, feature = "pg_test"))]
pgx::pg_sql_graph_magic!();

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // noop
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}

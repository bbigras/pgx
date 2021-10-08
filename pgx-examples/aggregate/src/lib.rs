use pgx::*;
use pgx::datum::sql_entity_graph::aggregate::{ParallelOption, FinalizeModify};
use std::{
    str::FromStr,
    ffi::CStr,
};
use serde::{Serialize, Deserialize};

pg_module_magic!();

#[derive(Copy, Clone, PostgresType, Serialize, Deserialize)]
#[pgvarlena_inoutfuncs]
pub struct IntegerAvgState {
    sum: i32,
    n: i32,
}

impl PgVarlenaInOutFuncs for IntegerAvgState {
    fn input(input: &CStr) -> PgVarlena<Self> {
        let mut result = PgVarlena::<Self>::new();

        let mut split = input.to_bytes().split(|b| *b == b',');
        let sum = split.next().map(|value| 
            i32::from_str(unsafe { std::str::from_utf8_unchecked(value) }).expect("invalid i32")
        ).expect("expected sum");
        let n = split.next().map(|value| 
            i32::from_str(unsafe { std::str::from_utf8_unchecked(value) }).expect("invalid i32")
        ).expect("expected n");

        result.sum = sum;
        result.n = n;

        result
    }
    fn output(&self, buffer: &mut StringInfo) {
        buffer.push_str(&format!("{},{}", self.sum, self.n));
    }
}

#[pg_aggregate]
impl Aggregate for IntegerAvgState {
    type Args = i32;
    const NAME: &'static str = "DEMOAVG";

    fn state(&self, _: i32) -> Self { todo!() }

    // You can skip all these:
    type Finalize = i32;
    type OrderBy = i32;
    type MovingState = i32;

    const PARALLEL: Option<ParallelOption> = Some(ParallelOption::Unsafe);
    const FINALIZE_MODIFY: Option<FinalizeModify> = Some(FinalizeModify::ReadWrite);
    const MOVING_FINALIZE_MODIFY: Option<FinalizeModify> = Some(FinalizeModify::ReadWrite);
    const INITIAL_CONDITION: Option<&'static str> = Some("0,0");
    const SORT_OPERATOR: Option<&'static str> = Some("sortop");
    const MOVING_INITIAL_CONDITION: Option<&'static str> = Some("1,1");
    const HYPOTHETICAL: bool = true;

    // You can skip all these:
    fn finalize(&self) -> Self::Finalize {
        unimplemented!("pgx stub, define in impls")
    }

    fn combine(&self, other: Self) -> Self {
        unimplemented!("pgx stub, define in impls")
    }
    
    fn serial(&self) -> Vec<u8> {
        unimplemented!("pgx stub, define in impls")
    }

    fn deserial(&self, buf: Vec<u8>, internal: PgBox<Self>) -> PgBox<Self> {
        unimplemented!("pgx stub, define in impls")
    }

    fn moving_state(mstate: Self::MovingState, v: Self::Args) -> Self::MovingState {
        unimplemented!("pgx stub, define in impls")
    }

    fn moving_finalize(mstate: Self::MovingState) -> Self::Finalize {
        unimplemented!("pgx stub, define in impls")
    } 
}

impl Default for IntegerAvgState {
    fn default() -> Self {
        Self { sum: 0, n: 0 }
    }
}

impl IntegerAvgState {
    fn acc(&self, v: i32) -> PgVarlena<Self> {
        let mut new = PgVarlena::<Self>::new();
        new.sum = self.sum + v;
        new.n = self.n + 1;
        new
    }
    fn finalize(&self) -> i32 {
        self.sum / self.n
    }
}

#[pg_extern]
fn integer_avg_state_func(
    internal_state: PgVarlena<IntegerAvgState>,
    next_data_value: i32,
) -> PgVarlena<IntegerAvgState> {
    internal_state.acc(next_data_value)
}

#[pg_extern]
fn integer_avg_final_func(internal_state: PgVarlena<IntegerAvgState>) -> i32 {
    internal_state.finalize()
}

extension_sql!(
    r#"
    CREATE AGGREGATE DEMOAVG (integer)
    (
        sfunc = integer_avg_state_func,
        stype = IntegerAvgState,
        finalfunc = integer_avg_final_func,
        initcond = '0,0'
    );
    "#,
    name = "create_demoavg_aggregate",
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::*;
    use crate::IntegerAvgState;

    #[pg_test]
    fn test_integer_avg_state() {
        assert_eq!(
            2,
            IntegerAvgState::default().acc(1).acc(2).acc(3).finalize()
        );
    }

    #[pg_test]
    fn test_integer_avg_state_sql() {
        Spi::run("CREATE TABLE demo_table (value INTEGER);");
        Spi::run("INSERT INTO demo_table (value) VALUES (1), (2), (3);");
        let retval =
            Spi::get_one::<i32>("SELECT DEMOAVG(value) FROM demo_table;")
                .expect("SQL select failed");
        assert_eq!(retval, 2);
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

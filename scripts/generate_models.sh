diesel_ext -d "Insertable, Queryable, Selectable, AsChangeset, Serialize, Deserialize" \
    -I "diesel::prelude::*" \
    -I "serde::{Serialize, Deserialize}" \
    -I "crate::db::schema::*" \
    -t \
    > src/db/model.rs

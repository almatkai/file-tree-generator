//! Contract root: all-target linting of the production adapter; byte-exact host dependency.
pub mod native_core {
    pub use beta_fixture::native_core::common;
    pub mod extensions {
        pub use beta_fixture::native_core::extensions::{v2_sdk, Sessions};
        pub mod v2_file_tree_generator;
    }
}

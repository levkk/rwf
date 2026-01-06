use rwf::prelude::*;
use rwf::controller::{IntoPkey, PkeyParamGenerator, ModelController, ModelListQuery, ModelPkeyParam};
use rwf::http::Handler;


#[rwf_macros::generate_full_model(i16, TestModelController, test_model, "/tmodel")]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TestModel {
    id: Option<i16>,
    ts: OffsetDateTime,
    name: String,
}






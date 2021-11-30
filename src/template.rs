use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    ScopedJson,
};

use fnv::FnvHasher;
use serde_json::value::Value as Json;
use std::format;
use std::hash::Hasher;

#[derive(Debug, Clone, Copy)]
pub struct LocaleHelper;

impl HelperDef for LocaleHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let collection_value = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"lookup\""))?;
        let index = h
            .param(1)
            .ok_or_else(|| RenderError::new("Insufficient params for helper \"lookup\""))?;

        let value = match *collection_value.value() {
            Json::Array(ref v) => index
                .value()
                .as_u64()
                .and_then(|u| v.get(u as usize))
                .unwrap_or(&Json::Null),
            Json::Object(ref m) => index
                .value()
                .as_str()
                .and_then(|k| m.get(k))
                .unwrap_or_else(|| m.get("en_US").unwrap_or(&Json::Null)),
            _ => &Json::Null,
        };
        if r.strict_mode() && value.is_null() {
            Err(RenderError::strict_error(None))
        } else {
            Ok(value.clone().into())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FNVHelper;

impl HelperDef for FNVHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"fnv\""))?;

        let mut fnv_hasher = FnvHasher::default();
        fnv_hasher.write(param.value().as_str().unwrap().as_bytes());

        let checksum = fnv_hasher.finish();

        out.write(format!("{:x}", checksum).as_str())?;
        Ok(())
    }
}

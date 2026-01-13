use abi_stable::std_types::{ROption, RResult, RStr, RString, RVec};
use lib_plugin_abi::{
    PluginContext, PluginInfo, PluginVTable, ServiceDescriptor, ServiceHandle, ServiceMethod,
    ServiceVTable, ServiceVersion,
};
use std::ffi::c_void;

const MESSAGES_FTL: &str = include_str!("../messages.ftl");

const METADATA_JSON: &str = r#"{
  "plugin_id": "adi.cli",
  "language": "fr-FR",
  "language_name": "Français",
  "namespace": "cli",
  "version": "1.0.0"
}"#;

const SERVICE_ID: &str = "adi.i18n.cli.fr-FR";

extern "C" fn plugin_info() -> PluginInfo {
    PluginInfo::new(
        "adi.cli.fr-FR",
        "ADI CLI - Français",
        "1.0.0",
        "translation",
    )
    .with_author("ADI Team")
    .with_description("French translations for ADI CLI")
    .with_min_host_version("0.8.0")
}

extern "C" fn plugin_init(ctx: *mut PluginContext) -> i32 {
    unsafe {
        let host = (*ctx).host();
        let descriptor =
            ServiceDescriptor::new(SERVICE_ID, ServiceVersion::new(1, 0, 0), "adi.cli.fr-FR")
                .with_description("French translations for ADI CLI");
        let handle = ServiceHandle::new(
            SERVICE_ID,
            ctx as *const c_void,
            &TRANSLATION_SERVICE_VTABLE as *const ServiceVTable,
        );
        if let Err(code) = host.register_svc(descriptor, handle) {
            host.error(&format!("Failed to register translation service: {}", code));
            return code;
        }
        host.info("ADI CLI French translation plugin initialized");
    }
    0
}

extern "C" fn plugin_cleanup(_ctx: *mut PluginContext) {}

extern "C" fn service_invoke(
    _handle: *const c_void,
    method: RStr<'_>,
    _args: RStr<'_>,
) -> RResult<RString, lib_plugin_abi::ServiceError> {
    match method.as_str() {
        "get_messages" => RResult::ROk(RString::from(MESSAGES_FTL)),
        "get_metadata" => RResult::ROk(RString::from(METADATA_JSON)),
        _ => RResult::RErr(lib_plugin_abi::ServiceError::method_not_found(
            method.as_str(),
        )),
    }
}

extern "C" fn service_list_methods(_handle: *const c_void) -> RVec<ServiceMethod> {
    vec![
        ServiceMethod::new("get_messages").with_description("Get Fluent .ftl file content"),
        ServiceMethod::new("get_metadata").with_description("Get translation metadata as JSON"),
    ]
    .into()
}

static TRANSLATION_SERVICE_VTABLE: ServiceVTable = ServiceVTable {
    invoke: service_invoke,
    list_methods: service_list_methods,
};

static PLUGIN_VTABLE: PluginVTable = PluginVTable {
    info: plugin_info,
    init: plugin_init,
    update: ROption::RNone,
    cleanup: plugin_cleanup,
    handle_message: ROption::RNone,
};

#[no_mangle]
pub extern "C" fn plugin_entry() -> *const PluginVTable {
    &PLUGIN_VTABLE
}

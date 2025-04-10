use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use crate::{ResourceConverter, ResourceGenerator, ResourceLibError, WoaVersion};
use glacier_codegen::{hm2016_bindings, hm2_bindings, hm3_bindings};

pub trait ResourceLibResource : serde::Serialize + for<'a> serde::Deserialize<'a> + Debug{
    fn get_version() -> WoaVersion;
    fn get_resource_type() -> String;
}

macro_rules! register_resource {
    ($ty:ty, $version:path, $type_label:literal) => {
        impl ResourceLibResource for $ty {
            fn get_version() -> WoaVersion {
                $version
            }

            fn get_resource_type() -> String {
                format!("{}", $type_label)
            }
        }
    };
}

//hitman 2016
register_resource!(hm2016_bindings::properties::STemplateEntity, WoaVersion::HM2016, "TEMP");
register_resource!(hm2016_bindings::properties::STemplateEntityBlueprint, WoaVersion::HM2016, "TBLU");
register_resource!(hm2016_bindings::properties::SReasoningGrid, WoaVersion::HM2016, "AIRG");
register_resource!(hm2016_bindings::properties::ZamdTake, WoaVersion::HM2016, "ATMD");
register_resource!(hm2016_bindings::properties::SVideoDatabaseData, WoaVersion::HM2016, "VIDB");
register_resource!(hm2016_bindings::properties::SCppEntityBlueprint, WoaVersion::HM2016, "CBLU");
register_resource!(hm2016_bindings::properties::SCppEntity, WoaVersion::HM2016, "CPPT");
register_resource!(hm2016_bindings::properties::SCrowdMapData, WoaVersion::HM2016, "CRMD");

//hitman 2
register_resource!(hm2_bindings::properties::STemplateEntityFactory, WoaVersion::HM2, "TEMP");
register_resource!(hm2_bindings::properties::STemplateEntityBlueprint, WoaVersion::HM2, "TBLU");
register_resource!(hm2_bindings::properties::SReasoningGrid, WoaVersion::HM2, "AIRG");
register_resource!(hm2_bindings::properties::ZamdTake, WoaVersion::HM2, "ATMD");
register_resource!(hm2_bindings::properties::SVideoDatabaseData, WoaVersion::HM2, "VIDB");
register_resource!(hm2_bindings::properties::SCppEntityBlueprint, WoaVersion::HM2, "CBLU");
register_resource!(hm2_bindings::properties::SCppEntity, WoaVersion::HM2, "CPPT");
register_resource!(hm2_bindings::properties::SCrowdMapData, WoaVersion::HM2, "CRMD");

//hitman 3
register_resource!(hm3_bindings::properties::STemplateEntityFactory, WoaVersion::HM3, "TEMP");
register_resource!(hm3_bindings::properties::STemplateEntityBlueprint, WoaVersion::HM3, "TBLU");
register_resource!(hm3_bindings::properties::SReasoningGrid, WoaVersion::HM3, "AIRG");
register_resource!(hm3_bindings::properties::ZamdTake, WoaVersion::HM3, "ATMD");
register_resource!(hm3_bindings::properties::SVideoDatabaseData, WoaVersion::HM3, "VIDB");
register_resource!(hm3_bindings::properties::SCppEntityBlueprint, WoaVersion::HM3, "CBLU");
register_resource!(hm3_bindings::properties::SCppEntity, WoaVersion::HM3, "CPPT");
register_resource!(hm3_bindings::properties::SCrowdMapData, WoaVersion::HM3, "CRMD");

#[derive(Debug, Clone)]
pub struct ResourceParserTyped<T: ResourceLibResource> {
    _marker: PhantomData<T>,
    converter: ResourceConverter,
    generator: ResourceGenerator,
}

unsafe impl<T: ResourceLibResource> Send for ResourceParserTyped<T> {}
unsafe impl<T: ResourceLibResource> Sync for ResourceParserTyped<T> {}

impl<T: ResourceLibResource> ResourceParserTyped<T> {
    pub fn new() -> Result<Self, ResourceLibError> {
            Ok(ResourceParserTyped {
                _marker: Default::default(),
                converter: ResourceConverter::new(T::get_version(), &*T::get_resource_type())?,
                generator: ResourceGenerator::new(T::get_version(), &*T::get_resource_type())?,
            })
    }

    pub fn parse_from_memory(&self, resource_data: &[u8]) -> Result<T, ResourceLibError> {
        let json = self.converter.memory_to_json_string(resource_data)?;
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub fn parse_from_file<P: AsRef<Path>>(&self, resource_file_path: P,) -> Result<T, ResourceLibError> {
        let json = self.converter.resource_file_to_json_string(resource_file_path)?;
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub fn parse_to_memory(&self, object: T, generate_compatible: bool) -> Result<Vec<u8>, ResourceLibError> {
        Ok(self.generator.json_string_to_resource_mem(&*serde_json::to_string(&object)?, generate_compatible)?)
    }

    pub fn parse_to_file<P: AsRef<Path>>(&self, resource_file_path: P, object: T, generate_compatible: bool) -> Result<bool, ResourceLibError> {
        Ok(self.generator.json_string_to_resource_file(&*serde_json::to_string(&object)?, resource_file_path, generate_compatible)?)
    }
}


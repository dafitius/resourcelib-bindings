#[cfg(test)]
mod tests {
    use resourcelib_ffi::ResourceGenerator;
use resourcelib_ffi::ResourceConverter;
use resourcelib_ffi::ResourceLib;
use resourcelib_ffi::WoaVersion;

    #[test]
    fn test_supported_resource_types() {
        let versions = [WoaVersion::HM2016, WoaVersion::HM2, WoaVersion::HM3];

        for version in versions.into_iter() {
            let result = ResourceLib::supported_resource_types(version);
            assert!(
                result.is_ok(),
                "Failed to get supported resource types for {:?}",
                version
            );
            let types = result.unwrap();
            assert!(
                !types.is_empty(),
                "Supported resource types list is empty for {:?}",
                version
            );
            println!("Supported types for {:?}: {:?}", version, types);
        }
    }

    #[test]
    fn test_is_supported_resource_type() {
        let versions = [WoaVersion::HM2016, WoaVersion::HM2, WoaVersion::HM3];

        for &version in versions.iter() {
            let result = ResourceLib::supported_resource_types(version);
            assert!(
                result.is_ok(),
                "Failed to get supported resource types for {:?}",
                version
            );
            let types = result.unwrap();
            assert!(
                !types.is_empty(),
                "Supported resource types list is empty for {:?}",
                version
            );

            // Use the first resource type for testing
            let resource_type = &types[0];
            let is_supported = ResourceLib::is_supported_resource_type(version, resource_type);
            assert!(
                is_supported,
                "Resource type '{}' should be supported for {:?}",
                resource_type,
                version
            );

            // Test with an unsupported resource type
            let is_supported =
                ResourceLib::is_supported_resource_type(version, "nonexistent_type");
            assert!(
                !is_supported,
                "Resource type 'nonexistent_type' should not be supported for {:?}",
                version
            );
        }
    }

    #[test]
    fn test_resource_converter_new() {
        let versions = [WoaVersion::HM2016, WoaVersion::HM2, WoaVersion::HM3];

        for &version in versions.iter() {
            let result = ResourceLib::supported_resource_types(version);
            assert!(
                result.is_ok(),
                "Failed to get supported resource types for {:?}",
                version
            );
            let types = result.unwrap();
            assert!(
                !types.is_empty(),
                "Supported resource types list is empty for {:?}",
                version
            );

            let resource_type = &types[0];
            let converter = ResourceConverter::new(version, resource_type);
            assert!(
                converter.is_some(),
                "Failed to create ResourceConverter for type '{}' and version {:?}",
                resource_type,
                version
            );

            // Test with an unsupported resource type
            let converter = ResourceConverter::new(version, "nonexistent_type");
            assert!(
                converter.is_none(),
                "ResourceConverter should not be created for unsupported resource type 'nonexistent_type' and version {:?}",
                version
            );
        }
    }

    #[test]
    fn test_resource_generator_new() {
        let versions = [WoaVersion::HM2016, WoaVersion::HM2, WoaVersion::HM3];

        for &version in versions.iter() {
            let result = ResourceLib::supported_resource_types(version);
            assert!(
                result.is_ok(),
                "Failed to get supported resource types for {:?}",
                version
            );
            let types = result.unwrap();
            assert!(
                !types.is_empty(),
                "Supported resource types list is empty for {:?}",
                version
            );

            let resource_type = &types[0];
            let generator = ResourceGenerator::new(version, resource_type);
            assert!(
                generator.is_some(),
                "Failed to create ResourceGenerator for type '{}' and version {:?}",
                resource_type,
                version
            );

            // Test with an unsupported resource type
            let generator = ResourceGenerator::new(version, "nonexistent_type");
            assert!(
                generator.is_none(),
                "ResourceGenerator should not be created for unsupported resource type 'nonexistent_type' and version {:?}",
                version
            );
        }
    }

}
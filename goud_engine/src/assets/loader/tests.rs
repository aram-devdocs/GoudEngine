//! Unit tests for the asset loader infrastructure.

#[cfg(test)]
mod asset_load_error {
    use crate::assets::AssetLoadError;

    #[test]
    fn test_not_found() {
        let error = AssetLoadError::not_found("test.png");
        assert!(error.is_not_found());
        assert!(!error.is_io_error());
        assert!(error.to_string().contains("test.png"));
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = AssetLoadError::io_error("test.png", io_err);
        assert!(error.is_io_error());
        assert!(!error.is_not_found());
        assert!(error.to_string().contains("test.png"));
        assert!(error.to_string().contains("access denied"));
    }

    #[test]
    fn test_decode_failed() {
        let error = AssetLoadError::decode_failed("Invalid UTF-8");
        assert!(error.is_decode_failed());
        assert!(error.to_string().contains("Invalid UTF-8"));
    }

    #[test]
    fn test_unsupported_format() {
        let error = AssetLoadError::unsupported_format("xyz");
        assert!(error.is_unsupported_format());
        assert!(error.to_string().contains("xyz"));
    }

    #[test]
    fn test_dependency_failed() {
        let error =
            AssetLoadError::dependency_failed("main.asset", "dependency.asset", "not found");
        assert!(error.is_dependency_failed());
        let msg = error.to_string();
        assert!(msg.contains("main.asset"));
        assert!(msg.contains("dependency.asset"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_custom() {
        let error = AssetLoadError::custom("Something went wrong");
        assert!(error.is_custom());
        assert!(error.to_string().contains("Something went wrong"));
    }

    #[test]
    fn test_clone() {
        let error = AssetLoadError::decode_failed("test");
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }
}

#[cfg(test)]
mod load_context {
    use crate::assets::{AssetPath, LoadContext};

    #[test]
    fn test_new() {
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let context = LoadContext::new(path);
        assert_eq!(context.path_str(), "assets/test.txt");
    }

    #[test]
    fn test_path_str() {
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let context = LoadContext::new(path);
        assert_eq!(context.path_str(), "assets/test.txt");
    }

    #[test]
    fn test_extension() {
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let context = LoadContext::new(path);
        assert_eq!(context.extension(), Some("txt"));
    }

    #[test]
    fn test_file_name() {
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let context = LoadContext::new(path);
        assert_eq!(context.file_name(), Some("test.txt"));
    }

    #[test]
    fn test_debug() {
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let context = LoadContext::new(path);
        let debug_str = format!("{:?}", context);
        assert!(debug_str.contains("LoadContext"));
        assert!(debug_str.contains("assets/test.txt"));
    }
}

#[cfg(test)]
mod asset_loader {
    use crate::assets::loader::test_fixtures::{
        BinaryAssetLoader, LoaderSettings, SettingsLoader, TextAssetLoader,
    };
    use crate::assets::{AssetLoader, AssetPath, LoadContext};

    #[test]
    fn test_text_loader_extensions() {
        let loader = TextAssetLoader;
        assert_eq!(loader.extensions(), &["txt", "text"]);
    }

    #[test]
    fn test_text_loader_supports_extension() {
        let loader = TextAssetLoader;
        assert!(loader.supports_extension("txt"));
        assert!(loader.supports_extension("text"));
        assert!(loader.supports_extension("TXT")); // case-insensitive
        assert!(!loader.supports_extension("png"));
    }

    #[test]
    fn test_text_loader_load_success() {
        let loader = TextAssetLoader;
        let bytes = b"Hello, World!";
        let path = AssetPath::from_string("test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bytes, &(), &mut context);
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.content, "Hello, World!");
    }

    #[test]
    fn test_text_loader_load_invalid_utf8() {
        let loader = TextAssetLoader;
        let bytes = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let path = AssetPath::from_string("test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(bytes, &(), &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_decode_failed());
    }

    #[test]
    fn test_binary_loader() {
        let loader = BinaryAssetLoader;
        let bytes = vec![1, 2, 3, 4, 5];
        let path = AssetPath::from_string("test.bin".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(&bytes, &(), &mut context);
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.data, bytes);
    }

    #[test]
    fn test_loader_with_settings() {
        let loader = SettingsLoader;
        let settings = LoaderSettings { max_size: 10 };
        let bytes = vec![1, 2, 3, 4, 5]; // 5 bytes, under limit
        let path = AssetPath::from_string("test.custom".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loader_settings_limit_exceeded() {
        let loader = SettingsLoader;
        let settings = LoaderSettings { max_size: 3 };
        let bytes = vec![1, 2, 3, 4, 5]; // 5 bytes, over limit
        let path = AssetPath::from_string("test.custom".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_custom());
        assert!(error.to_string().contains("too large"));
    }
}

#[cfg(test)]
mod typed_asset_loader {
    use crate::assets::loader::test_fixtures::{LoaderSettings, SettingsLoader, TextAssetLoader};
    use crate::assets::{AssetLoader, AssetPath, ErasedAssetLoader, LoadContext, TypedAssetLoader};

    #[test]
    fn test_new() {
        let loader = TypedAssetLoader::new(TextAssetLoader);
        assert_eq!(loader.extensions(), &["txt", "text"]);
    }

    #[test]
    fn test_with_settings() {
        let settings = LoaderSettings { max_size: 100 };
        let loader = TypedAssetLoader::with_settings(SettingsLoader, settings);
        assert_eq!(loader.settings().max_size, 100);
    }

    #[test]
    fn test_loader_accessor() {
        let typed = TypedAssetLoader::new(TextAssetLoader);
        let loader = typed.loader();
        assert_eq!(loader.extensions(), &["txt", "text"]);
    }

    #[test]
    fn test_settings_accessor() {
        let settings = LoaderSettings { max_size: 50 };
        let typed = TypedAssetLoader::with_settings(SettingsLoader, settings);
        assert_eq!(typed.settings().max_size, 50);
    }

    #[test]
    fn test_settings_mut_accessor() {
        let mut typed = TypedAssetLoader::new(SettingsLoader);
        typed.settings_mut().max_size = 200;
        assert_eq!(typed.settings().max_size, 200);
    }

    #[test]
    fn test_erased_extensions() {
        let typed = TypedAssetLoader::new(TextAssetLoader);
        let erased: &dyn ErasedAssetLoader = &typed;
        assert_eq!(erased.extensions(), &["txt", "text"]);
    }

    #[test]
    fn test_erased_supports_extension() {
        let typed = TypedAssetLoader::new(TextAssetLoader);
        let erased: &dyn ErasedAssetLoader = &typed;
        assert!(erased.supports_extension("txt"));
        assert!(!erased.supports_extension("png"));
    }

    #[test]
    fn test_erased_load_success() {
        use crate::assets::loader::test_fixtures::TextAsset;
        let typed = TypedAssetLoader::new(TextAssetLoader);
        let erased: &dyn ErasedAssetLoader = &typed;
        let bytes = b"Hello, World!";
        let path = AssetPath::from_string("test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = erased.load_erased(bytes, &mut context);
        assert!(result.is_ok());

        let boxed = result.unwrap();
        let asset = boxed.downcast::<TextAsset>().unwrap();
        assert_eq!(asset.content, "Hello, World!");
    }

    #[test]
    fn test_erased_load_failure() {
        let typed = TypedAssetLoader::new(TextAssetLoader);
        let erased: &dyn ErasedAssetLoader = &typed;
        let bytes = &[0xFF, 0xFE]; // Invalid UTF-8
        let path = AssetPath::from_string("test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = erased.load_erased(bytes, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_loaders() {
        use crate::assets::loader::test_fixtures::BinaryAssetLoader;
        let text_loader: Box<dyn ErasedAssetLoader> =
            Box::new(TypedAssetLoader::new(TextAssetLoader));
        let binary_loader: Box<dyn ErasedAssetLoader> =
            Box::new(TypedAssetLoader::new(BinaryAssetLoader));

        assert!(text_loader.supports_extension("txt"));
        assert!(!text_loader.supports_extension("bin"));

        assert!(binary_loader.supports_extension("bin"));
        assert!(!binary_loader.supports_extension("txt"));
    }

    #[test]
    fn test_boxed_collection() {
        use crate::assets::loader::test_fixtures::BinaryAssetLoader;
        let loaders: Vec<Box<dyn ErasedAssetLoader>> = vec![
            Box::new(TypedAssetLoader::new(TextAssetLoader)),
            Box::new(TypedAssetLoader::new(BinaryAssetLoader)),
        ];

        assert_eq!(loaders.len(), 2);
        assert!(loaders[0].supports_extension("txt"));
        assert!(loaders[1].supports_extension("bin"));
    }
}

#[cfg(test)]
mod thread_safety {
    use crate::assets::loader::test_fixtures::TextAssetLoader;
    use crate::assets::{AssetLoadError, TypedAssetLoader};

    #[test]
    fn test_asset_load_error_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AssetLoadError>();
    }

    #[test]
    fn test_asset_load_error_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<AssetLoadError>();
    }

    #[test]
    fn test_text_loader_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TextAssetLoader>();
    }

    #[test]
    fn test_text_loader_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TextAssetLoader>();
    }

    #[test]
    fn test_typed_loader_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TypedAssetLoader<TextAssetLoader>>();
    }

    #[test]
    fn test_typed_loader_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TypedAssetLoader<TextAssetLoader>>();
    }
}

#[cfg(test)]
mod integration {
    use crate::assets::loader::test_fixtures::{BinaryAssetLoader, TextAsset, TextAssetLoader};
    use crate::assets::{AssetPath, ErasedAssetLoader, LoadContext, TypedAssetLoader};

    #[test]
    fn test_full_workflow() {
        let loader = TypedAssetLoader::new(TextAssetLoader);

        let bytes = b"Test content";
        let path = AssetPath::from_string("assets/test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load_erased(bytes, &mut context);
        assert!(result.is_ok());

        let boxed = result.unwrap();
        let asset = boxed.downcast::<TextAsset>().unwrap();
        assert_eq!(asset.content, "Test content");
    }

    #[test]
    fn test_loader_registry_pattern() {
        use std::collections::HashMap;

        let mut registry: HashMap<&str, Box<dyn ErasedAssetLoader>> = HashMap::new();
        registry.insert("txt", Box::new(TypedAssetLoader::new(TextAssetLoader)));
        registry.insert("bin", Box::new(TypedAssetLoader::new(BinaryAssetLoader)));

        let loader = registry.get("txt").unwrap();
        assert!(loader.supports_extension("txt"));

        let loader = registry.get("bin").unwrap();
        assert!(loader.supports_extension("bin"));
    }

    #[test]
    fn test_error_propagation() {
        let loader = TypedAssetLoader::new(TextAssetLoader);
        let bytes = &[0xFF, 0xFE]; // Invalid UTF-8
        let path = AssetPath::from_string("test.txt".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load_erased(bytes, &mut context);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.is_decode_failed());
    }
}

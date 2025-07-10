// Integration test for Social login trait object

#[cfg(feature = "social")]
use starberry_oauth::social::provider::ExternalLoginProvider;

#[cfg(feature = "social")]
#[test]
fn test_social_provider_trait_object_safety() {
    // Ensure trait is object-safe
    let providers: Vec<Box<dyn ExternalLoginProvider>> = Vec::new();
    let _ = providers;
} 
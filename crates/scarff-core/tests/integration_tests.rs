// //! Integration tests for scarff-core.

// use scarff_adapters::{InMemoryStore, LocalFilesystem, MemoryFilesystem, SimpleRenderer};
// use scarff_core::{
//     application::{ScaffoldService, TemplateService, ports::*},
//     domain::{Architecture, Language, ProjectKind, Target},
//     prelude::*,
// };

// #[test]
// fn test_full_scaffold_workflow() {
//     // Setup adapters
//     let store = Box::new(InMemoryStore::with_builtin().unwrap());
//     let renderer = Box::new(SimpleRenderer::new());
//     let filesystem = Box::new(MemoryFilesystem::new());

//     // Create service
//     let service = ScaffoldService::new(store, renderer, filesystem.clone());

//     // Create target
//     let target = Target::builder()
//         .language(Language::Rust)
//         .kind(ProjectKind::Cli)
//         .unwrap()
//         .build()
//         .unwrap();

//     // Scaffold
//     service.scaffold(target, "test-project", "/output").unwrap();

//     // Verify
//     assert!(filesystem.exists("/output/test-project".as_ref()));
//     assert!(filesystem.exists("/output/test-project/src/main.rs".as_ref()));

//     let main_content = filesystem
//         .read_file("/output/test-project/src/main.rs".as_ref())
//         .unwrap();
//     assert!(main_content.contains("test-project"));
// }

// #[test]
// fn test_template_resolution_specificity() {
//     let store = Box::new(InMemoryStore::new());
//     let renderer = Box::new(SimpleRenderer::new());
//     let filesystem = Box::new(MemoryFilesystem::new());

//     let service = ScaffoldService::new(store, renderer, filesystem);

//     // Add two templates - one specific, one general
//     let general = Template::builder()
//         .id(TemplateId::new("general", "1.0.0"))
//         .matcher(TargetMatcher::builder().language(Language::Rust).build())
//         .metadata(TemplateMetadata::new("General"))
//         .add_node(TemplateNode::File(FileSpec::new(
//             "general.txt",
//             TemplateContent::Literal(TemplateSource::Static("general")),
//         )))
//         .build()
//         .unwrap();

//     let specific = Template::builder()
//         .id(TemplateId::new("specific", "1.0.0"))
//         .matcher(
//             TargetMatcher::builder()
//                 .language(Language::Rust)
//                 .kind(ProjectKind::Cli)
//                 .build(),
//         )
//         .metadata(TemplateMetadata::new("Specific"))
//         .add_node(TemplateNode::File(FileSpec::new(
//             "specific.txt",
//             TemplateContent::Literal(TemplateSource::Static("specific")),
//         )))
//         .build()
//         .unwrap();

//     // Use TemplateService to add templates
//     let template_service = TemplateService::new(service.store);
//     template_service.save(general).unwrap();
//     template_service.save(specific).unwrap();

//     // Resolve should pick specific
//     let target = Target::builder()
//         .language(Language::Rust)
//         .kind(ProjectKind::Cli)
//         .unwrap()
//         .build()
//         .unwrap();

//     let templates = service.find_templates(&target).unwrap();
//     assert_eq!(templates.len(), 2); // Both match

//     // But scaffold picks most specific
//     service.scaffold(target, "test", "/out").unwrap();
//     assert!(filesystem.exists("/out/test/specific.txt".as_ref()));
//     assert!(!filesystem.exists("/out/test/general.txt".as_ref()));
// }

// #[test]
// fn test_rollback_on_failure() {
//     let store = Box::new(InMemoryStore::with_builtin().unwrap());
//     let renderer = Box::new(SimpleRenderer::new());
//     let filesystem = Box::new(MemoryFilesystem::new());

//     let service = ScaffoldService::new(store, renderer, filesystem.clone());

//     // First scaffold succeeds
//     let target = Target::builder()
//         .language(Language::Rust)
//         .kind(ProjectKind::Cli)
//         .unwrap()
//         .build()
//         .unwrap();

//     service.scaffold(target.clone(), "project", "/out").unwrap();
//     assert!(filesystem.exists("/out/project".as_ref()));

//     // Second scaffold to same location should fail and rollback
//     let result = service.scaffold(target, "project", "/out");
//     assert!(result.is_err());

//     // Original should still exist (rollback doesn't delete existing)
//     assert!(filesystem.exists("/out/project".as_ref()));
// }

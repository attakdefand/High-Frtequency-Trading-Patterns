//! Final integration test for Phase 6 features

use std::process::Command;

#[test]
fn test_phase6_components_compile() {
    // Test that all new crates compile successfully
    let crates = vec!["sniper-market", "svc-market", "svc-gateway"];
    
    for crate_name in crates {
        let output = Command::new("cargo")
            .args(&["check", "-p", crate_name])
            .output()
            .expect("Failed to execute cargo check");
            
        assert!(output.status.success(), 
                "Failed to compile {}: {}", crate_name, String::from_utf8_lossy(&output.stderr));
    }
    
    println!("All Phase 6 components compile successfully!");
}

#[test]
fn test_phase6_services_run() {
    // Test that services can be built (not actually run to avoid port conflicts)
    let services = vec!["svc-market", "svc-gateway"];
    
    for service in services {
        let output = Command::new("cargo")
            .args(&["build", "-p", service])
            .output()
            .expect("Failed to execute cargo build");
            
        assert!(output.status.success(), 
                "Failed to build service {}: {}", service, String::from_utf8_lossy(&output.stderr));
    }
    
    println!("All Phase 6 services build successfully!");
}

#[test]
fn test_phase6_documentation_exists() {
    // Test that documentation files exist
    let doc_files = vec![
        "COMMUNITY_GUIDE.MD",
        "TUTORIALS/first_strategy.MD",
        "TUTORIALS/signal_processors.MD",
        "MOBILE_APP/README.MD",
        "DESKTOP_APP/README.MD",
        "PROFESSIONAL_SERVICES.MD",
        "PHASE6_SUMMARY.MD"
    ];
    
    for doc_file in doc_files {
        let path = format!("{}", doc_file);
        assert!(std::path::Path::new(&path).exists(), 
                "Documentation file {} does not exist", doc_file);
    }
    
    println!("All Phase 6 documentation files exist!");
}
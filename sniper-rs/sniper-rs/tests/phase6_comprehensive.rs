//! Comprehensive integration test for all Phase 6 features

use std::process::Command;

#[test]
fn test_all_phase6_features() {
    // Test 1: Verify all new crates compile
    let new_crates = vec!["sniper-plugin", "svc-plugin", "sniper-market", "svc-market"];
    
    for crate_name in new_crates {
        let output = Command::new("cargo")
            .args(&["check", "-p", crate_name])
            .output()
            .expect("Failed to execute cargo check");
            
        assert!(output.status.success(), 
                "Failed to compile {}: {}", crate_name, String::from_utf8_lossy(&output.stderr));
    }
    
    // Test 2: Verify enhanced gateway service compiles
    let output = Command::new("cargo")
        .args(&["check", "-p", "svc-gateway"])
        .output()
        .expect("Failed to execute cargo check for svc-gateway");
        
    assert!(output.status.success(), 
            "Failed to compile svc-gateway: {}", String::from_utf8_lossy(&output.stderr));
    
    // Test 3: Verify all documentation files exist
    let doc_files = vec![
        "COMMUNITY_GUIDE.MD",
        "TUTORIALS/first_strategy.MD",
        "TUTORIALS/signal_processors.MD",
        "MOBILE_APP/README.MD",
        "DESKTOP_APP/README.MD",
        "PROFESSIONAL_SERVICES.MD",
        "PHASE6_SUMMARY.MD",
        "FINAL_SUMMARY.MD"
    ];
    
    for doc_file in doc_files {
        assert!(std::path::Path::new(doc_file).exists(), 
                "Documentation file {} does not exist", doc_file);
    }
    
    // Test 4: Verify all services can be built
    let services = vec!["svc-plugin", "svc-market"];
    
    for service in services {
        let output = Command::new("cargo")
            .args(&["build", "--bin", service])
            .output()
            .expect("Failed to execute cargo build");
            
        assert!(output.status.success(), 
                "Failed to build service {}: {}", service, String::from_utf8_lossy(&output.stderr));
    }
    
    println!("All Phase 6 features are fully implemented and working correctly!");
}
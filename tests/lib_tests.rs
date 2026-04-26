use std::ops::Deref;

use you_shall_not_pass::password_generator::{
    calc_entropy, generate_password, Defaults, DEFAULTS, PasswordError,
};

// ---------------------------------------------------------------------------
// Task 1: Happy-path tests
// ---------------------------------------------------------------------------

#[test]
fn test_generate_password_default_length() {
    let password = generate_password(&DEFAULTS, rand::rng()).unwrap();
    assert_eq!(password.len(), DEFAULTS.length);
}

#[test]
fn test_generate_password_custom_length() {
    let config = Defaults {
        length: 24,
        ..DEFAULTS
    };
    let password = generate_password(&config, rand::rng()).unwrap();
    assert_eq!(password.len(), 24);
}

#[test]
fn test_generate_password_no_symbols() {
    let config = Defaults {
        length: 16,
        symbol_flag: false,
        ..DEFAULTS
    };
    let password = generate_password(&config, rand::rng()).unwrap();
    assert_eq!(password.len(), 16);
    for ch in password.chars() {
        assert!(
            !"!@#$%^&*".contains(ch),
            "found disallowed symbol character: {ch}"
        );
    }
}

#[test]
fn test_calc_entropy_classical() {
    let config = Defaults {
        quantum_check: false,
        ..DEFAULTS
    };
    let entropy = calc_entropy(&config, rand::rng()).unwrap();
    assert!(
        entropy > 60.0,
        "classical entropy {entropy} should be > 60.0"
    );
}

#[test]
fn test_calc_entropy_quantum() {
    let classical_config = Defaults {
        quantum_check: false,
        ..DEFAULTS
    };
    let quantum_config = Defaults {
        quantum_check: true,
        ..DEFAULTS
    };
    let classical = calc_entropy(&classical_config, rand::rng()).unwrap();
    let quantum = calc_entropy(&quantum_config, rand::rng()).unwrap();
    let expected = classical / 2.0;
    let diff = (quantum - expected).abs();
    assert!(
        diff < 1.0,
        "quantum entropy {quantum} should be ~half of classical {classical} (expected {expected}, diff {diff})"
    );
}

#[test]
fn test_password_is_zeroizing() {
    let password = generate_password(&DEFAULTS, rand::rng()).unwrap();
    // Verify Zeroizing<String> implements Deref<Target = String>
    let s: &String = password.deref();
    assert_eq!(s.len(), DEFAULTS.length);
    // Also accessible as &str via auto-deref coercion
    let s2: &str = &password;
    assert_eq!(s2.len(), DEFAULTS.length);
}
